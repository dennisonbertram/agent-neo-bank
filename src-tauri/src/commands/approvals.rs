use tauri::State;

use crate::core::approval_manager::ApprovalManager;
use crate::db::models::{ApprovalRequest, ApprovalRequestType, ApprovalStatus};
use crate::db::queries;
use crate::error::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn list_approvals(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<ApprovalRequest>, AppError> {
    let db = state.db.clone();
    let status_filter = status.as_deref().map(|s| match s {
        "pending" => ApprovalStatus::Pending,
        "approved" => ApprovalStatus::Approved,
        "denied" => ApprovalStatus::Denied,
        "expired" => ApprovalStatus::Expired,
        _ => ApprovalStatus::Pending,
    });
    tokio::task::spawn_blocking(move || {
        let manager = ApprovalManager::new(db);
        manager.list_all(status_filter.as_ref())
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn resolve_approval(
    state: State<'_, AppState>,
    approval_id: String,
    decision: String,
) -> Result<ApprovalRequest, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let status = match decision.as_str() {
            "approved" => ApprovalStatus::Approved,
            "denied" => ApprovalStatus::Denied,
            _ => {
                return Err(AppError::InvalidInput(
                    "Decision must be 'approved' or 'denied'".to_string(),
                ))
            }
        };

        let manager = ApprovalManager::new(db.clone());
        let resolved = manager.resolve(&approval_id, status.clone(), "user")?;

        // Side effect: if limit_increase was approved, update spending policy
        if status == ApprovalStatus::Approved
            && resolved.request_type == ApprovalRequestType::LimitIncrease
        {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&resolved.payload) {
                let proposed = payload.get("proposed").cloned().unwrap_or(payload.clone());
                if let Ok(mut policy) = queries::get_spending_policy(&db, &resolved.agent_id) {
                    if let Some(v) = proposed.get("new_per_tx_max").and_then(|v| v.as_str()) {
                        policy.per_tx_max = v.to_string();
                    }
                    if let Some(v) = proposed.get("new_daily_cap").and_then(|v| v.as_str()) {
                        policy.daily_cap = v.to_string();
                    }
                    if let Some(v) = proposed.get("new_weekly_cap").and_then(|v| v.as_str()) {
                        policy.weekly_cap = v.to_string();
                    }
                    if let Some(v) = proposed.get("new_monthly_cap").and_then(|v| v.as_str()) {
                        policy.monthly_cap = v.to_string();
                    }
                    policy.updated_at = chrono::Utc::now().timestamp();
                    let _ = queries::update_spending_policy(&db, &policy);
                }
            }
        }

        // Side effect: if transaction was approved, execute it
        if status == ApprovalStatus::Approved
            && resolved.request_type == ApprovalRequestType::Transaction
        {
            if let Some(ref tx_id) = resolved.tx_id {
                let _ = queries::update_transaction_status(
                    &db,
                    tx_id,
                    &crate::db::models::TxStatus::Executing,
                    None,
                    None,
                    chrono::Utc::now().timestamp(),
                );
                // Note: ledger was already reserved by check_policy_and_reserve_atomic.
                // execute_send will confirm the tx or rollback on failure.
            }
        }

        // Side effect: if transaction was denied, rollback the reservation
        if status == ApprovalStatus::Denied
            && resolved.request_type == ApprovalRequestType::Transaction
        {
            if let Some(ref tx_id) = resolved.tx_id {
                // Get the transaction to retrieve amount and period keys
                if let Ok(tx) = queries::get_transaction(&db, tx_id) {
                    let _ = queries::rollback_reservation(
                        &db,
                        &resolved.agent_id,
                        &tx.amount,
                        &tx.period_daily,
                        &tx.period_weekly,
                        &tx.period_monthly,
                        chrono::Utc::now().timestamp(),
                    );
                    let _ = queries::update_transaction_status(
                        &db,
                        tx_id,
                        &crate::db::models::TxStatus::Denied,
                        None,
                        Some("Approval denied by user"),
                        chrono::Utc::now().timestamp(),
                    );
                }
            }
        }

        Ok(resolved)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn get_approval(
    state: State<'_, AppState>,
    approval_id: String,
) -> Result<ApprovalRequest, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let manager = ApprovalManager::new(db);
        manager.get(&approval_id)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{AgentStatus, ApprovalRequestType};
    use crate::db::queries::{insert_agent, insert_approval_request};
    use crate::db::schema::Database;

    fn create_test_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn create_test_agent(id: &str, name: &str) -> crate::db::models::Agent {
        crate::db::models::Agent {
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

    fn make_approval(agent_id: &str) -> ApprovalRequest {
        ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            request_type: ApprovalRequestType::Transaction,
            payload: "{}".to_string(),
            status: ApprovalStatus::Pending,
            tx_id: None,
            expires_at: chrono::Utc::now().timestamp() + 86400,
            created_at: chrono::Utc::now().timestamp(),
            resolved_at: None,
            resolved_by: None,
        }
    }

    #[test]
    fn test_list_approvals_command() {
        let db = create_test_db();
        let agent = create_test_agent("agent-1", "TestBot");
        insert_agent(&db, &agent).unwrap();

        let a1 = make_approval("agent-1");
        let a2 = make_approval("agent-1");
        insert_approval_request(&db, &a1).unwrap();
        insert_approval_request(&db, &a2).unwrap();

        let db = std::sync::Arc::new(db);
        let manager = ApprovalManager::new(db);
        let all = manager.list_all(None).unwrap();
        assert_eq!(all.len(), 2);

        let pending = manager.list_all(Some(&ApprovalStatus::Pending)).unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_resolve_approval_command() {
        let db = create_test_db();
        let agent = create_test_agent("agent-1", "TestBot");
        insert_agent(&db, &agent).unwrap();

        let approval = make_approval("agent-1");
        insert_approval_request(&db, &approval).unwrap();

        let db = std::sync::Arc::new(db);
        let manager = ApprovalManager::new(db);
        let resolved = manager
            .resolve(&approval.id, ApprovalStatus::Approved, "user")
            .unwrap();
        assert_eq!(resolved.status, ApprovalStatus::Approved);
    }

    #[test]
    fn test_resolve_limit_increase_approved_updates_policy() {
        let db = create_test_db();
        let agent = create_test_agent("agent-li", "LimitBot");
        insert_agent(&db, &agent).unwrap();

        // Insert spending policy
        let policy = crate::db::models::SpendingPolicy {
            agent_id: agent.id.clone(),
            per_tx_max: "100".to_string(),
            daily_cap: "1000".to_string(),
            weekly_cap: "5000".to_string(),
            monthly_cap: "20000".to_string(),
            auto_approve_max: "50".to_string(),
            allowlist: vec![],
            updated_at: 1740700800,
        };
        crate::db::queries::insert_spending_policy(&db, &policy).unwrap();

        // Create limit_increase approval
        let approval = ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent.id.clone(),
            request_type: ApprovalRequestType::LimitIncrease,
            payload: serde_json::json!({
                "proposed": {
                    "new_daily_cap": "3000",
                    "new_per_tx_max": "200",
                },
                "reason": "Need more"
            })
            .to_string(),
            status: ApprovalStatus::Pending,
            tx_id: None,
            expires_at: chrono::Utc::now().timestamp() + 86400,
            created_at: chrono::Utc::now().timestamp(),
            resolved_at: None,
            resolved_by: None,
        };
        insert_approval_request(&db, &approval).unwrap();

        let db = std::sync::Arc::new(db);
        let manager = ApprovalManager::new(db.clone());
        let resolved = manager
            .resolve(&approval.id, ApprovalStatus::Approved, "user")
            .unwrap();

        // Apply the side effect
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&resolved.payload) {
            let proposed = payload.get("proposed").cloned().unwrap_or(payload.clone());
            if let Ok(mut policy) = queries::get_spending_policy(&db, &resolved.agent_id) {
                if let Some(v) = proposed.get("new_per_tx_max").and_then(|v| v.as_str()) {
                    policy.per_tx_max = v.to_string();
                }
                if let Some(v) = proposed.get("new_daily_cap").and_then(|v| v.as_str()) {
                    policy.daily_cap = v.to_string();
                }
                if let Some(v) = proposed.get("new_weekly_cap").and_then(|v| v.as_str()) {
                    policy.weekly_cap = v.to_string();
                }
                if let Some(v) = proposed.get("new_monthly_cap").and_then(|v| v.as_str()) {
                    policy.monthly_cap = v.to_string();
                }
                policy.updated_at = chrono::Utc::now().timestamp();
                queries::update_spending_policy(&db, &policy).unwrap();
            }
        }

        let updated = queries::get_spending_policy(&db, &agent.id).unwrap();
        assert_eq!(updated.daily_cap, "3000");
        assert_eq!(updated.per_tx_max, "200");
        assert_eq!(updated.weekly_cap, "5000"); // unchanged
        assert_eq!(updated.monthly_cap, "20000"); // unchanged
    }

    #[test]
    fn test_resolve_limit_increase_denied_preserves_policy() {
        let db = create_test_db();
        let agent = create_test_agent("agent-lid", "LimitDenyBot");
        insert_agent(&db, &agent).unwrap();

        let policy = crate::db::models::SpendingPolicy {
            agent_id: agent.id.clone(),
            per_tx_max: "100".to_string(),
            daily_cap: "1000".to_string(),
            weekly_cap: "5000".to_string(),
            monthly_cap: "20000".to_string(),
            auto_approve_max: "50".to_string(),
            allowlist: vec![],
            updated_at: 1740700800,
        };
        crate::db::queries::insert_spending_policy(&db, &policy).unwrap();

        let approval = ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent.id.clone(),
            request_type: ApprovalRequestType::LimitIncrease,
            payload: serde_json::json!({
                "proposed": { "new_daily_cap": "999999" },
                "reason": "Want more"
            })
            .to_string(),
            status: ApprovalStatus::Pending,
            tx_id: None,
            expires_at: chrono::Utc::now().timestamp() + 86400,
            created_at: chrono::Utc::now().timestamp(),
            resolved_at: None,
            resolved_by: None,
        };
        insert_approval_request(&db, &approval).unwrap();

        let db = std::sync::Arc::new(db);
        let manager = ApprovalManager::new(db.clone());
        let _resolved = manager
            .resolve(&approval.id, ApprovalStatus::Denied, "admin")
            .unwrap();

        // Denied: no side effect, policy unchanged
        let current = queries::get_spending_policy(&db, &agent.id).unwrap();
        assert_eq!(current.daily_cap, "1000");
        assert_eq!(current.per_tx_max, "100");
    }

    #[test]
    fn test_get_approval_command() {
        let db = create_test_db();
        let agent = create_test_agent("agent-1", "TestBot");
        insert_agent(&db, &agent).unwrap();

        let approval = make_approval("agent-1");
        insert_approval_request(&db, &approval).unwrap();

        let db = std::sync::Arc::new(db);
        let manager = ApprovalManager::new(db);
        let fetched = manager.get(&approval.id).unwrap();
        assert_eq!(fetched.id, approval.id);
    }
}
