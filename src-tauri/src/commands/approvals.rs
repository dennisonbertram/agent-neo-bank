use tauri::State;

use crate::core::approval_manager::ApprovalManager;
use crate::db::models::{ApprovalRequest, ApprovalStatus};
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
        let manager = ApprovalManager::new(db);
        let status = match decision.as_str() {
            "approved" => ApprovalStatus::Approved,
            "denied" => ApprovalStatus::Denied,
            _ => {
                return Err(AppError::InvalidInput(
                    "Decision must be 'approved' or 'denied'".to_string(),
                ))
            }
        };
        manager.resolve(&approval_id, status, "user")
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
