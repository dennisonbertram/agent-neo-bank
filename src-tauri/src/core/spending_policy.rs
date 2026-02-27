use std::str::FromStr;
use std::sync::Arc;

use chrono::{Datelike, IsoWeek, Utc};
use rust_decimal::Decimal;

use crate::db::queries::{get_spending_for_period, get_spending_policy};
use crate::db::schema::Database;
use crate::error::AppError;

// -------------------------------------------------------------------------
// PolicyDecision
// -------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecision {
    AutoApproved,
    RequiresApproval { reason: String },
    Denied { reason: String },
}

// -------------------------------------------------------------------------
// Period key helpers
// -------------------------------------------------------------------------

pub fn daily_period_key(dt: &chrono::DateTime<Utc>) -> String {
    format!("daily:{}", dt.format("%Y-%m-%d"))
}

pub fn weekly_period_key(dt: &chrono::DateTime<Utc>) -> String {
    let week: IsoWeek = dt.iso_week();
    format!("weekly:{}-W{:02}", week.year(), week.week())
}

pub fn monthly_period_key(dt: &chrono::DateTime<Utc>) -> String {
    format!("monthly:{}", dt.format("%Y-%m"))
}

// -------------------------------------------------------------------------
// SpendingPolicyEngine
// -------------------------------------------------------------------------

pub struct SpendingPolicyEngine {
    db: Arc<Database>,
}

impl SpendingPolicyEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Evaluate a spending request against the agent's spending policy.
    ///
    /// Evaluation order:
    /// 1. Get spending policy from DB
    /// 2. Check per_tx_max (deny if exceeded; 0 means zero-capped = denied)
    /// 3. Check allowlist (deny if recipient not in non-empty allowlist)
    /// 4. Check daily_cap against spending_ledger for today
    /// 5. Check weekly_cap against spending_ledger for this week
    /// 6. Check monthly_cap against spending_ledger for this month
    /// 7. If amount <= auto_approve_max -> AutoApproved
    /// 8. Else -> RequiresApproval
    pub fn evaluate(
        &self,
        agent_id: &str,
        amount: Decimal,
        recipient: &str,
    ) -> Result<PolicyDecision, AppError> {
        let policy = get_spending_policy(&self.db, agent_id)?;

        let per_tx_max = Decimal::from_str(&policy.per_tx_max)
            .map_err(|e| AppError::Internal(format!("Invalid per_tx_max: {}", e)))?;
        let daily_cap = Decimal::from_str(&policy.daily_cap)
            .map_err(|e| AppError::Internal(format!("Invalid daily_cap: {}", e)))?;
        let weekly_cap = Decimal::from_str(&policy.weekly_cap)
            .map_err(|e| AppError::Internal(format!("Invalid weekly_cap: {}", e)))?;
        let monthly_cap = Decimal::from_str(&policy.monthly_cap)
            .map_err(|e| AppError::Internal(format!("Invalid monthly_cap: {}", e)))?;
        let auto_approve_max = Decimal::from_str(&policy.auto_approve_max)
            .map_err(|e| AppError::Internal(format!("Invalid auto_approve_max: {}", e)))?;

        // 2. Check per_tx_max
        if amount > per_tx_max {
            return Ok(PolicyDecision::Denied {
                reason: format!(
                    "Amount {} exceeds per-tx limit of {}",
                    amount, per_tx_max
                ),
            });
        }

        // 3. Check allowlist
        if !policy.allowlist.is_empty()
            && !policy.allowlist.iter().any(|a| a == recipient)
        {
            return Ok(PolicyDecision::Denied {
                reason: "Recipient not in allowlist".to_string(),
            });
        }

        // 4-6. Check period caps
        let now = Utc::now();

        // Daily cap
        let daily_key = daily_period_key(&now);
        let daily_spent = self.get_period_total(agent_id, &daily_key)?;
        if daily_spent + amount > daily_cap {
            return Ok(PolicyDecision::Denied {
                reason: format!(
                    "Amount {} would exceed daily cap of {} (already spent {})",
                    amount, daily_cap, daily_spent
                ),
            });
        }

        // Weekly cap
        let weekly_key = weekly_period_key(&now);
        let weekly_spent = self.get_period_total(agent_id, &weekly_key)?;
        if weekly_spent + amount > weekly_cap {
            return Ok(PolicyDecision::Denied {
                reason: format!(
                    "Amount {} would exceed weekly cap of {} (already spent {})",
                    amount, weekly_cap, weekly_spent
                ),
            });
        }

        // Monthly cap
        let monthly_key = monthly_period_key(&now);
        let monthly_spent = self.get_period_total(agent_id, &monthly_key)?;
        if monthly_spent + amount > monthly_cap {
            return Ok(PolicyDecision::Denied {
                reason: format!(
                    "Amount {} would exceed monthly cap of {} (already spent {})",
                    amount, monthly_cap, monthly_spent
                ),
            });
        }

        // 7-8. Auto-approve vs requires approval
        if amount <= auto_approve_max {
            Ok(PolicyDecision::AutoApproved)
        } else {
            Ok(PolicyDecision::RequiresApproval {
                reason: format!(
                    "Amount {} exceeds auto-approve threshold of {}",
                    amount, auto_approve_max
                ),
            })
        }
    }

    fn get_period_total(&self, agent_id: &str, period: &str) -> Result<Decimal, AppError> {
        match get_spending_for_period(&self.db, agent_id, period)? {
            Some(ledger) => Decimal::from_str(&ledger.total)
                .map_err(|e| AppError::Internal(format!("Invalid ledger total: {}", e))),
            None => Ok(Decimal::ZERO),
        }
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::AgentStatus;
    use crate::db::queries::{insert_agent, insert_spending_policy, upsert_spending_ledger};
    use crate::test_helpers::{create_test_agent, create_test_spending_policy, setup_test_db};

    /// Helper: set up engine + agent + policy, return (engine, agent_id).
    fn setup_with_policy(
        per_tx_max: &str,
        daily_cap: &str,
        weekly_cap: &str,
        monthly_cap: &str,
        auto_approve_max: &str,
        allowlist: Vec<String>,
    ) -> (SpendingPolicyEngine, String) {
        let db = setup_test_db();
        let agent = create_test_agent("PolicyTestBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let mut policy =
            create_test_spending_policy(&agent.id, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max);
        policy.allowlist = allowlist;
        insert_spending_policy(&db, &policy).unwrap();

        let engine = SpendingPolicyEngine::new(db);
        (engine, agent.id)
    }

    /// Helper: set up engine with pre-existing spending in ledger.
    fn setup_with_spending(
        per_tx_max: &str,
        daily_cap: &str,
        weekly_cap: &str,
        monthly_cap: &str,
        auto_approve_max: &str,
        allowlist: Vec<String>,
        daily_spent: Option<&str>,
        weekly_spent: Option<&str>,
        monthly_spent: Option<&str>,
    ) -> (SpendingPolicyEngine, String) {
        let db = setup_test_db();
        let agent = create_test_agent("SpendTestBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let mut policy =
            create_test_spending_policy(&agent.id, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max);
        policy.allowlist = allowlist;
        insert_spending_policy(&db, &policy).unwrap();

        let now = chrono::Utc::now().timestamp();

        if let Some(amount) = daily_spent {
            let key = daily_period_key(&Utc::now());
            upsert_spending_ledger(&db, &agent.id, &key, amount, now).unwrap();
        }
        if let Some(amount) = weekly_spent {
            let key = weekly_period_key(&Utc::now());
            upsert_spending_ledger(&db, &agent.id, &key, amount, now).unwrap();
        }
        if let Some(amount) = monthly_spent {
            let key = monthly_period_key(&Utc::now());
            upsert_spending_ledger(&db, &agent.id, &key, amount, now).unwrap();
        }

        let engine = SpendingPolicyEngine::new(db);
        (engine, agent.id)
    }

    // 1. Auto-approves below threshold
    #[test]
    fn test_spending_policy_auto_approves_below_threshold() {
        let (engine, agent_id) =
            setup_with_policy("25", "100", "500", "1500", "10", vec![]);

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("5.00").unwrap(), "0xAnyone")
            .unwrap();

        assert_eq!(decision, PolicyDecision::AutoApproved);
    }

    // 2. Requires approval above auto_approve_max but within per_tx_max
    #[test]
    fn test_spending_policy_requires_approval_above_threshold() {
        let (engine, agent_id) =
            setup_with_policy("25", "100", "500", "1500", "10", vec![]);

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("15.00").unwrap(), "0xAnyone")
            .unwrap();

        match decision {
            PolicyDecision::RequiresApproval { .. } => {} // expected
            other => panic!("Expected RequiresApproval, got {:?}", other),
        }
    }

    // 3. Denied when per_tx_max exceeded
    #[test]
    fn test_spending_policy_denies_when_per_tx_limit_exceeded() {
        let (engine, agent_id) =
            setup_with_policy("25", "100", "500", "1500", "10", vec![]);

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("26.00").unwrap(), "0xAnyone")
            .unwrap();

        match decision {
            PolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("exceeds per-tx limit"),
                    "Expected per-tx limit message, got: {}",
                    reason
                );
                assert!(reason.contains("26"), "Should mention amount 26");
                assert!(reason.contains("25"), "Should mention limit 25");
            }
            other => panic!("Expected Denied, got {:?}", other),
        }
    }

    // 4. Denied when daily cap exceeded
    #[test]
    fn test_spending_policy_denies_when_daily_cap_exceeded() {
        let (engine, agent_id) = setup_with_spending(
            "100", "100", "500", "1500", "10",
            vec![],
            Some("95.00"), Some("95.00"), Some("95.00"),
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("6.00").unwrap(), "0xAnyone")
            .unwrap();

        match decision {
            PolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("daily cap"),
                    "Expected daily cap message, got: {}",
                    reason
                );
            }
            other => panic!("Expected Denied for daily cap, got {:?}", other),
        }
    }

    // 5. Denied when weekly cap exceeded
    #[test]
    fn test_spending_policy_denies_when_weekly_cap_exceeded() {
        let (engine, agent_id) = setup_with_spending(
            "100", "1000", "500", "1500", "10",
            vec![],
            Some("0"), Some("498.00"), Some("498.00"),
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("3.00").unwrap(), "0xAnyone")
            .unwrap();

        match decision {
            PolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("weekly cap"),
                    "Expected weekly cap message, got: {}",
                    reason
                );
            }
            other => panic!("Expected Denied for weekly cap, got {:?}", other),
        }
    }

    // 6. Denied when monthly cap exceeded
    #[test]
    fn test_spending_policy_denies_when_monthly_cap_exceeded() {
        let (engine, agent_id) = setup_with_spending(
            "100", "1000", "5000", "1500", "10",
            vec![],
            Some("0"), Some("0"), Some("1490.00"),
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("11.00").unwrap(), "0xAnyone")
            .unwrap();

        match decision {
            PolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("monthly cap"),
                    "Expected monthly cap message, got: {}",
                    reason
                );
            }
            other => panic!("Expected Denied for monthly cap, got {:?}", other),
        }
    }

    // 7. Exact amount equals limit -- allowed (not denied)
    #[test]
    fn test_spending_policy_exact_amount_equals_limit_allowed() {
        let (engine, agent_id) = setup_with_spending(
            "25", "100", "500", "1500", "10",
            vec![],
            Some("75.00"), Some("75.00"), Some("75.00"),
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("25.00").unwrap(), "0xAnyone")
            .unwrap();

        // Should NOT be Denied -- exact boundary is allowed
        match decision {
            PolicyDecision::Denied { reason } => {
                panic!("Should not be denied at exact boundary, got: {}", reason)
            }
            _ => {} // AutoApproved or RequiresApproval are both fine
        }
    }

    // 8. Just over limit -- denied
    #[test]
    fn test_spending_policy_just_over_limit_denied() {
        let (engine, agent_id) = setup_with_spending(
            "100", "100", "500", "1500", "10",
            vec![],
            Some("75.00"), Some("75.00"), Some("75.00"),
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("25.01").unwrap(), "0xAnyone")
            .unwrap();

        match decision {
            PolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("daily cap"),
                    "Expected daily cap denial, got: {}",
                    reason
                );
            }
            other => panic!("Expected Denied, got {:?}", other),
        }
    }

    // 9. Allowlist enforced -- recipient allowed
    #[test]
    fn test_spending_policy_allowlist_enforced_recipient_allowed() {
        let (engine, agent_id) = setup_with_policy(
            "25", "100", "500", "1500", "10",
            vec!["0xAllowed1".to_string(), "0xAllowed2".to_string()],
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("5.00").unwrap(), "0xAllowed1")
            .unwrap();

        // Should not be Denied for allowlist
        match &decision {
            PolicyDecision::Denied { reason } if reason.contains("allowlist") => {
                panic!("Should not be denied by allowlist for allowed recipient")
            }
            _ => {} // OK
        }
    }

    // 10. Allowlist enforced -- recipient blocked
    #[test]
    fn test_spending_policy_allowlist_enforced_recipient_blocked() {
        let (engine, agent_id) = setup_with_policy(
            "25", "100", "500", "1500", "10",
            vec!["0xAllowed1".to_string()],
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("5.00").unwrap(), "0xNotAllowed")
            .unwrap();

        match decision {
            PolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("Recipient not in allowlist"),
                    "Expected allowlist denial, got: {}",
                    reason
                );
            }
            other => panic!("Expected Denied for blocked recipient, got {:?}", other),
        }
    }

    // 11. Empty allowlist allows any recipient
    #[test]
    fn test_spending_policy_empty_allowlist_allows_any_recipient() {
        let (engine, agent_id) =
            setup_with_policy("25", "100", "500", "1500", "10", vec![]);

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("5.00").unwrap(), "0xAnyAddress")
            .unwrap();

        match &decision {
            PolicyDecision::Denied { reason } if reason.contains("allowlist") => {
                panic!("Empty allowlist should not deny any recipient")
            }
            _ => {} // OK
        }
    }

    // 12. Multiple rules combined
    #[test]
    fn test_spending_policy_multiple_rules_combined() {
        let (engine, agent_id) = setup_with_spending(
            "50", "100", "500", "1500", "10",
            vec!["0xValid".to_string()],
            Some("40.00"), Some("40.00"), Some("40.00"),
        );

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("45.00").unwrap(), "0xValid")
            .unwrap();

        // amount 45 > auto_approve_max 10, but within per_tx_max 50, daily 40+45=85 <= 100,
        // weekly 40+45=85 <= 500, monthly 40+45=85 <= 1500, recipient in allowlist
        match decision {
            PolicyDecision::RequiresApproval { .. } => {} // expected
            other => panic!("Expected RequiresApproval, got {:?}", other),
        }
    }

    // 13. Zero caps mean denied
    #[test]
    fn test_spending_policy_zero_caps_mean_denied() {
        let (engine, agent_id) =
            setup_with_policy("0", "0", "0", "0", "0", vec![]);

        let decision = engine
            .evaluate(&agent_id, Decimal::from_str("1.00").unwrap(), "0xAnyone")
            .unwrap();

        match decision {
            PolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("per-tx limit"),
                    "Expected per-tx denial for zero cap, got: {}",
                    reason
                );
            }
            other => panic!("Expected Denied for zero caps, got {:?}", other),
        }
    }
}
