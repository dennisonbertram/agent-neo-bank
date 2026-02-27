use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;

use crate::db::models::GlobalPolicy;
use crate::db::queries::{
    get_global_policy, get_global_spending_for_period, upsert_global_policy,
    upsert_global_spending_ledger,
};
use crate::db::schema::Database;
use crate::error::AppError;

// -------------------------------------------------------------------------
// Decision enum
// -------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalPolicyDecision {
    Allowed,
    Denied { reason: String },
}

// -------------------------------------------------------------------------
// Period key helpers
// -------------------------------------------------------------------------

pub fn daily_period_key() -> String {
    let now = Utc::now();
    format!("daily:{}", now.format("%Y-%m-%d"))
}

pub fn weekly_period_key() -> String {
    let now = Utc::now();
    format!("weekly:{}", now.format("%G-W%V"))
}

pub fn monthly_period_key() -> String {
    let now = Utc::now();
    format!("monthly:{}", now.format("%Y-%m"))
}

// -------------------------------------------------------------------------
// GlobalPolicyEngine
// -------------------------------------------------------------------------

pub struct GlobalPolicyEngine {
    db: Arc<Database>,
}

impl GlobalPolicyEngine {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Evaluate a transaction against the global policy.
    ///
    /// Evaluation order:
    /// 1. Load global policy (if none, default to permissive = Allowed)
    /// 2. Check kill switch (overrides everything)
    /// 3. Check minimum reserve balance
    /// 4. Check daily cap (if > 0)
    /// 5. Check weekly cap (if > 0)
    /// 6. Check monthly cap (if > 0)
    /// 7. Otherwise -> Allowed
    pub fn evaluate(
        &self,
        amount: Decimal,
        current_balance: Decimal,
    ) -> Result<GlobalPolicyDecision, AppError> {
        // 1. Load global policy
        let policy = match get_global_policy(&self.db)? {
            Some(p) => p,
            None => return Ok(GlobalPolicyDecision::Allowed),
        };

        // 2. Check kill switch
        if policy.kill_switch_active {
            return Ok(GlobalPolicyDecision::Denied {
                reason: format!(
                    "Emergency kill switch active: {}",
                    policy.kill_switch_reason
                ),
            });
        }

        // 3. Check minimum reserve balance
        let min_reserve: Decimal = policy
            .min_reserve_balance
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid min_reserve_balance: {}", e)))?;
        if min_reserve > Decimal::ZERO && current_balance - amount < min_reserve {
            return Ok(GlobalPolicyDecision::Denied {
                reason: format!(
                    "Would drop balance below minimum reserve of {}",
                    min_reserve
                ),
            });
        }

        // 4. Check daily cap
        let daily_cap: Decimal = policy
            .daily_cap
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid daily_cap: {}", e)))?;
        if daily_cap > Decimal::ZERO {
            let period = daily_period_key();
            let current_total = self.get_period_total(&period)?;
            if current_total + amount > daily_cap {
                return Ok(GlobalPolicyDecision::Denied {
                    reason: format!(
                        "Global daily spending cap of {} would be exceeded",
                        daily_cap
                    ),
                });
            }
        }

        // 5. Check weekly cap
        let weekly_cap: Decimal = policy
            .weekly_cap
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid weekly_cap: {}", e)))?;
        if weekly_cap > Decimal::ZERO {
            let period = weekly_period_key();
            let current_total = self.get_period_total(&period)?;
            if current_total + amount > weekly_cap {
                return Ok(GlobalPolicyDecision::Denied {
                    reason: format!(
                        "Global weekly spending cap of {} would be exceeded",
                        weekly_cap
                    ),
                });
            }
        }

        // 6. Check monthly cap
        let monthly_cap: Decimal = policy
            .monthly_cap
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid monthly_cap: {}", e)))?;
        if monthly_cap > Decimal::ZERO {
            let period = monthly_period_key();
            let current_total = self.get_period_total(&period)?;
            if current_total + amount > monthly_cap {
                return Ok(GlobalPolicyDecision::Denied {
                    reason: format!(
                        "Global monthly spending cap of {} would be exceeded",
                        monthly_cap
                    ),
                });
            }
        }

        // 7. Allowed
        Ok(GlobalPolicyDecision::Allowed)
    }

    /// Toggle the kill switch on/off with a reason.
    pub fn toggle_kill_switch(&self, active: bool, reason: &str) -> Result<(), AppError> {
        let now = Utc::now().timestamp();

        // Load existing policy or create default
        let mut policy = get_global_policy(&self.db)?.unwrap_or(GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "0".to_string(),
            weekly_cap: "0".to_string(),
            monthly_cap: "0".to_string(),
            min_reserve_balance: "0".to_string(),
            kill_switch_active: false,
            kill_switch_reason: String::new(),
            updated_at: now,
        });

        policy.kill_switch_active = active;
        policy.kill_switch_reason = reason.to_string();
        policy.updated_at = now;

        upsert_global_policy(&self.db, &policy)
    }

    /// Helper: get the total spending for a given period key.
    fn get_period_total(&self, period: &str) -> Result<Decimal, AppError> {
        match get_global_spending_for_period(&self.db, period)? {
            Some(ledger) => ledger
                .total
                .parse()
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
    use std::str::FromStr;

    use super::*;
    use crate::db::models::GlobalPolicy;
    use crate::db::queries::upsert_global_policy;
    use crate::test_helpers::setup_test_db;

    /// Helper: insert a global policy with the given caps/settings.
    fn insert_policy(
        db: &Database,
        daily_cap: &str,
        weekly_cap: &str,
        monthly_cap: &str,
        min_reserve: &str,
        kill_switch_active: bool,
        kill_switch_reason: &str,
    ) {
        let policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: daily_cap.to_string(),
            weekly_cap: weekly_cap.to_string(),
            monthly_cap: monthly_cap.to_string(),
            min_reserve_balance: min_reserve.to_string(),
            kill_switch_active,
            kill_switch_reason: kill_switch_reason.to_string(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        upsert_global_policy(db, &policy).unwrap();
    }

    /// Helper: seed global spending ledger for a period.
    fn seed_global_spending(db: &Database, period: &str, amount: &str) {
        upsert_global_spending_ledger(db, period, amount, chrono::Utc::now().timestamp()).unwrap();
    }

    /// Helper: parse a Decimal from a string literal.
    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    // 1. test_global_policy_allows_within_daily_cap
    #[test]
    fn test_global_policy_allows_within_daily_cap() {
        let db = setup_test_db();
        insert_policy(&db, "500", "0", "0", "0", false, "");

        // Seed today's spending to 200
        let today = daily_period_key();
        seed_global_spending(&db, &today, "200");

        let engine = GlobalPolicyEngine::new(db);
        let result = engine.evaluate(d("50"), d("10000")).unwrap();
        assert_eq!(result, GlobalPolicyDecision::Allowed);
    }

    // 2. test_global_policy_denies_when_daily_cap_exceeded
    #[test]
    fn test_global_policy_denies_when_daily_cap_exceeded() {
        let db = setup_test_db();
        insert_policy(&db, "500", "0", "0", "0", false, "");

        let today = daily_period_key();
        seed_global_spending(&db, &today, "480");

        let engine = GlobalPolicyEngine::new(db);
        let result = engine.evaluate(d("25"), d("10000")).unwrap();
        match result {
            GlobalPolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("daily"),
                    "Expected 'daily' in reason: {}",
                    reason
                );
            }
            _ => panic!("Expected Denied, got Allowed"),
        }
    }

    // 3. test_global_policy_denies_when_weekly_cap_exceeded
    #[test]
    fn test_global_policy_denies_when_weekly_cap_exceeded() {
        let db = setup_test_db();
        insert_policy(&db, "0", "2000", "0", "0", false, "");

        let week = weekly_period_key();
        seed_global_spending(&db, &week, "1990");

        let engine = GlobalPolicyEngine::new(db);
        let result = engine.evaluate(d("15"), d("10000")).unwrap();
        match result {
            GlobalPolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("weekly"),
                    "Expected 'weekly' in reason: {}",
                    reason
                );
            }
            _ => panic!("Expected Denied, got Allowed"),
        }
    }

    // 4. test_global_policy_denies_when_monthly_cap_exceeded
    #[test]
    fn test_global_policy_denies_when_monthly_cap_exceeded() {
        let db = setup_test_db();
        insert_policy(&db, "0", "0", "5000", "0", false, "");

        let month = monthly_period_key();
        seed_global_spending(&db, &month, "4995");

        let engine = GlobalPolicyEngine::new(db);
        let result = engine.evaluate(d("10"), d("10000")).unwrap();
        match result {
            GlobalPolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("monthly"),
                    "Expected 'monthly' in reason: {}",
                    reason
                );
            }
            _ => panic!("Expected Denied, got Allowed"),
        }
    }

    // 5. test_global_policy_minimum_reserve_prevents_overdraw
    #[test]
    fn test_global_policy_minimum_reserve_prevents_overdraw() {
        let db = setup_test_db();
        insert_policy(&db, "0", "0", "0", "100", false, "");

        let engine = GlobalPolicyEngine::new(db);
        // balance=150, amount=60 -> remaining=90 < min_reserve=100
        let result = engine.evaluate(d("60"), d("150")).unwrap();
        match result {
            GlobalPolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("minimum reserve of 100"),
                    "Expected reserve message, got: {}",
                    reason
                );
            }
            _ => panic!("Expected Denied, got Allowed"),
        }
    }

    // 6. test_global_policy_minimum_reserve_allows_safe_tx
    #[test]
    fn test_global_policy_minimum_reserve_allows_safe_tx() {
        let db = setup_test_db();
        insert_policy(&db, "0", "0", "0", "100", false, "");

        let engine = GlobalPolicyEngine::new(db);
        // balance=500, amount=50 -> remaining=450 >= min_reserve=100
        let result = engine.evaluate(d("50"), d("500")).unwrap();
        assert_eq!(result, GlobalPolicyDecision::Allowed);
    }

    // 7. test_global_policy_kill_switch_denies_all
    #[test]
    fn test_global_policy_kill_switch_denies_all() {
        let db = setup_test_db();
        insert_policy(&db, "0", "0", "0", "0", true, "Suspicious activity");

        let engine = GlobalPolicyEngine::new(db);
        let result = engine.evaluate(d("1"), d("10000")).unwrap();
        match result {
            GlobalPolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("Emergency kill switch active: Suspicious activity"),
                    "Expected kill switch reason, got: {}",
                    reason
                );
            }
            _ => panic!("Expected Denied, got Allowed"),
        }
    }

    // 8. test_global_policy_kill_switch_denies_even_with_remaining_limits
    #[test]
    fn test_global_policy_kill_switch_denies_even_with_remaining_limits() {
        let db = setup_test_db();
        // Large caps but kill switch is active
        insert_policy(&db, "999999", "999999", "999999", "0", true, "Maintenance");

        let engine = GlobalPolicyEngine::new(db);
        let result = engine.evaluate(d("1"), d("10000")).unwrap();
        match result {
            GlobalPolicyDecision::Denied { reason } => {
                assert!(
                    reason.contains("kill switch"),
                    "Expected kill switch in reason: {}",
                    reason
                );
            }
            _ => panic!("Expected Denied, got Allowed"),
        }
    }

    // 9. test_global_policy_zero_cap_means_unlimited
    #[test]
    fn test_global_policy_zero_cap_means_unlimited() {
        let db = setup_test_db();
        // daily_cap=0 means unlimited
        insert_policy(&db, "0", "0", "0", "0", false, "");

        let engine = GlobalPolicyEngine::new(db);
        let result = engine.evaluate(d("999999"), d("9999999")).unwrap();
        assert_eq!(result, GlobalPolicyDecision::Allowed);
    }

    // 10. test_global_policy_reserve_edge_exact_balance
    #[test]
    fn test_global_policy_reserve_edge_exact_balance() {
        let db = setup_test_db();
        insert_policy(&db, "0", "0", "0", "100", false, "");

        let engine = GlobalPolicyEngine::new(db);
        // balance=200, amount=100 -> remaining=100 == min_reserve=100 -> Allowed
        let result = engine.evaluate(d("100"), d("200")).unwrap();
        assert_eq!(result, GlobalPolicyDecision::Allowed);
    }

    // Additional: test toggle_kill_switch
    #[test]
    fn test_toggle_kill_switch() {
        let db = setup_test_db();
        let engine = GlobalPolicyEngine::new(db.clone());

        // No policy yet, toggle on
        engine
            .toggle_kill_switch(true, "Security incident")
            .unwrap();

        let policy = get_global_policy(&db).unwrap().unwrap();
        assert!(policy.kill_switch_active);
        assert_eq!(policy.kill_switch_reason, "Security incident");

        // Toggle off
        engine.toggle_kill_switch(false, "").unwrap();

        let policy = get_global_policy(&db).unwrap().unwrap();
        assert!(!policy.kill_switch_active);
    }

    // Additional: test no policy defaults to allowed
    #[test]
    fn test_no_policy_defaults_to_allowed() {
        let db = setup_test_db();
        let engine = GlobalPolicyEngine::new(db);

        let result = engine.evaluate(d("1000"), d("5000")).unwrap();
        assert_eq!(result, GlobalPolicyDecision::Allowed);
    }
}
