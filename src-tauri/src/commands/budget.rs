use tauri::State;

use crate::db::models::{GlobalPolicy, SpendingPolicy};
use crate::db::queries;
use crate::error::AppError;
use crate::state::app_state::AppState;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AgentBudgetSummary {
    pub agent_id: String,
    pub agent_name: String,
    pub daily_spent: String,
    pub daily_cap: String,
    pub weekly_spent: String,
    pub weekly_cap: String,
    pub monthly_spent: String,
    pub monthly_cap: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlobalBudgetSummary {
    pub daily_spent: String,
    pub daily_cap: String,
    pub weekly_spent: String,
    pub weekly_cap: String,
    pub monthly_spent: String,
    pub monthly_cap: String,
    pub kill_switch_active: bool,
}

#[tauri::command]
pub async fn get_agent_budget_summaries(
    state: State<'_, AppState>,
) -> Result<Vec<AgentBudgetSummary>, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let agents = queries::list_all_agents(&db)?;
        let now = chrono::Utc::now();
        let daily_key = format!("daily:{}", now.format("%Y-%m-%d"));
        let weekly_key = format!("weekly:{}", now.format("%Y-W%V"));
        let monthly_key = format!("monthly:{}", now.format("%Y-%m"));

        let mut summaries = Vec::new();
        for agent in agents {
            let policy = queries::get_spending_policy(&db, &agent.id)
                .unwrap_or_else(|_| SpendingPolicy {
                    agent_id: agent.id.clone(),
                    per_tx_max: "0".into(),
                    daily_cap: "0".into(),
                    weekly_cap: "0".into(),
                    monthly_cap: "0".into(),
                    auto_approve_max: "0".into(),
                    allowlist: vec![],
                    updated_at: 0,
                });

            let daily_spent = queries::get_spending_for_period(&db, &agent.id, &daily_key)
                .ok()
                .flatten()
                .map(|l| l.total)
                .unwrap_or_else(|| "0".into());
            let weekly_spent = queries::get_spending_for_period(&db, &agent.id, &weekly_key)
                .ok()
                .flatten()
                .map(|l| l.total)
                .unwrap_or_else(|| "0".into());
            let monthly_spent = queries::get_spending_for_period(&db, &agent.id, &monthly_key)
                .ok()
                .flatten()
                .map(|l| l.total)
                .unwrap_or_else(|| "0".into());

            summaries.push(AgentBudgetSummary {
                agent_id: agent.id,
                agent_name: agent.name,
                daily_spent,
                daily_cap: policy.daily_cap,
                weekly_spent,
                weekly_cap: policy.weekly_cap,
                monthly_spent,
                monthly_cap: policy.monthly_cap,
            });
        }
        Ok(summaries)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn get_global_budget_summary(
    state: State<'_, AppState>,
) -> Result<GlobalBudgetSummary, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let policy = queries::get_global_policy(&db)?.unwrap_or(GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "0".to_string(),
            weekly_cap: "0".to_string(),
            monthly_cap: "0".to_string(),
            min_reserve_balance: "0".to_string(),
            kill_switch_active: false,
            kill_switch_reason: String::new(),
            updated_at: chrono::Utc::now().timestamp(),
        });

        let now = chrono::Utc::now();
        let daily_key = format!("daily:{}", now.format("%Y-%m-%d"));
        let weekly_key = format!("weekly:{}", now.format("%Y-W%V"));
        let monthly_key = format!("monthly:{}", now.format("%Y-%m"));

        let daily_spent = queries::get_global_spending_for_period(&db, &daily_key)
            .ok()
            .flatten()
            .map(|l| l.total)
            .unwrap_or_else(|| "0".into());
        let weekly_spent = queries::get_global_spending_for_period(&db, &weekly_key)
            .ok()
            .flatten()
            .map(|l| l.total)
            .unwrap_or_else(|| "0".into());
        let monthly_spent = queries::get_global_spending_for_period(&db, &monthly_key)
            .ok()
            .flatten()
            .map(|l| l.total)
            .unwrap_or_else(|| "0".into());

        Ok(GlobalBudgetSummary {
            daily_spent,
            daily_cap: policy.daily_cap,
            weekly_spent,
            weekly_cap: policy.weekly_cap,
            monthly_spent,
            monthly_cap: policy.monthly_cap,
            kill_switch_active: policy.kill_switch_active,
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::db::models::{Agent, AgentStatus, GlobalPolicy, SpendingPolicy};
    use crate::db::queries;
    use crate::db::schema::Database;

    fn create_test_db() -> Arc<Database> {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        Arc::new(db)
    }

    fn create_test_agent(id: &str, name: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test agent".to_string(),
            purpose: "Testing".to_string(),
            agent_type: "test".to_string(),
            capabilities: vec!["send".to_string()],
            status: AgentStatus::Active,
            api_token_hash: None,
            token_prefix: None,
            balance_visible: true,
            invitation_code: None,
            created_at: 1740700800,
            updated_at: 1740700800,
            last_active_at: None,
            metadata: "{}".to_string(),
        }
    }

    #[test]
    fn test_agent_budget_summaries_empty() {
        let db = create_test_db();
        let agents = queries::list_all_agents(&db).unwrap();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_agent_budget_summaries_with_agents() {
        let db = create_test_db();

        let agent = create_test_agent("agent-1", "Claude");
        queries::insert_agent(&db, &agent).unwrap();

        let policy = SpendingPolicy {
            agent_id: "agent-1".to_string(),
            per_tx_max: "50".to_string(),
            daily_cap: "1000".to_string(),
            weekly_cap: "5000".to_string(),
            monthly_cap: "20000".to_string(),
            auto_approve_max: "10".to_string(),
            allowlist: vec![],
            updated_at: 1740700800,
        };
        queries::insert_spending_policy(&db, &policy).unwrap();

        let now = chrono::Utc::now();
        let daily_key = format!("daily:{}", now.format("%Y-%m-%d"));

        queries::upsert_spending_ledger(&db, "agent-1", &daily_key, "250.00", 1740700800).unwrap();

        // Verify the data
        let fetched_policy = queries::get_spending_policy(&db, "agent-1").unwrap();
        assert_eq!(fetched_policy.daily_cap, "1000");

        let fetched_ledger =
            queries::get_spending_for_period(&db, "agent-1", &daily_key).unwrap();
        assert!(fetched_ledger.is_some());
        assert_eq!(fetched_ledger.unwrap().total, "250.00");
    }

    #[test]
    fn test_global_budget_summary_defaults() {
        let db = create_test_db();
        let result = queries::get_global_policy(&db).unwrap();
        // No policy set yet, should return None
        assert!(result.is_none());
    }

    #[test]
    fn test_global_budget_summary_with_policy() {
        let db = create_test_db();

        let policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "10000".to_string(),
            weekly_cap: "50000".to_string(),
            monthly_cap: "200000".to_string(),
            min_reserve_balance: "1000".to_string(),
            kill_switch_active: false,
            kill_switch_reason: String::new(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        queries::upsert_global_policy(&db, &policy).unwrap();

        let fetched = queries::get_global_policy(&db).unwrap().unwrap();
        assert_eq!(fetched.daily_cap, "10000");
        assert!(!fetched.kill_switch_active);
    }

    #[test]
    fn test_global_budget_summary_with_kill_switch() {
        let db = create_test_db();

        let policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "10000".to_string(),
            weekly_cap: "50000".to_string(),
            monthly_cap: "200000".to_string(),
            min_reserve_balance: "1000".to_string(),
            kill_switch_active: true,
            kill_switch_reason: "Emergency".to_string(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        queries::upsert_global_policy(&db, &policy).unwrap();

        let fetched = queries::get_global_policy(&db).unwrap().unwrap();
        assert!(fetched.kill_switch_active);
        assert_eq!(fetched.kill_switch_reason, "Emergency");
    }
}
