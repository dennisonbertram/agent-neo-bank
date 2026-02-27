use tauri::State;

use crate::db::models::GlobalPolicy;
use crate::db::queries;
use crate::error::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_global_policy(
    state: State<'_, AppState>,
) -> Result<GlobalPolicy, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let policy = queries::get_global_policy(&db)?;
        Ok(policy.unwrap_or(GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "0".to_string(),
            weekly_cap: "0".to_string(),
            monthly_cap: "0".to_string(),
            min_reserve_balance: "0".to_string(),
            kill_switch_active: false,
            kill_switch_reason: String::new(),
            updated_at: chrono::Utc::now().timestamp(),
        }))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn update_global_policy(
    state: State<'_, AppState>,
    policy: GlobalPolicy,
) -> Result<(), AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        queries::upsert_global_policy(&db, &policy)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn toggle_kill_switch(
    state: State<'_, AppState>,
    active: bool,
    reason: Option<String>,
) -> Result<(), AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let mut policy = queries::get_global_policy(&db)?.unwrap_or(GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "0".to_string(),
            weekly_cap: "0".to_string(),
            monthly_cap: "0".to_string(),
            min_reserve_balance: "0".to_string(),
            kill_switch_active: false,
            kill_switch_reason: String::new(),
            updated_at: chrono::Utc::now().timestamp(),
        });
        policy.kill_switch_active = active;
        policy.kill_switch_reason = reason.unwrap_or_default();
        policy.updated_at = chrono::Utc::now().timestamp();
        queries::upsert_global_policy(&db, &policy)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::db::models::GlobalPolicy;
    use crate::db::queries;
    use crate::db::schema::Database;

    fn create_test_db() -> Arc<Database> {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        Arc::new(db)
    }

    #[test]
    fn test_get_global_policy_returns_default_when_none() {
        let db = create_test_db();
        let result = queries::get_global_policy(&db).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_upsert_and_get_global_policy() {
        let db = create_test_db();
        let policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "1000".to_string(),
            weekly_cap: "5000".to_string(),
            monthly_cap: "20000".to_string(),
            min_reserve_balance: "100".to_string(),
            kill_switch_active: false,
            kill_switch_reason: String::new(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        queries::upsert_global_policy(&db, &policy).unwrap();

        let fetched = queries::get_global_policy(&db).unwrap().unwrap();
        assert_eq!(fetched.daily_cap, "1000");
        assert_eq!(fetched.weekly_cap, "5000");
        assert_eq!(fetched.monthly_cap, "20000");
        assert_eq!(fetched.min_reserve_balance, "100");
        assert!(!fetched.kill_switch_active);
    }

    #[test]
    fn test_toggle_kill_switch_on_and_off() {
        let db = create_test_db();

        // Insert initial policy
        let policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "1000".to_string(),
            weekly_cap: "5000".to_string(),
            monthly_cap: "20000".to_string(),
            min_reserve_balance: "100".to_string(),
            kill_switch_active: false,
            kill_switch_reason: String::new(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        queries::upsert_global_policy(&db, &policy).unwrap();

        // Toggle on
        let mut updated = queries::get_global_policy(&db).unwrap().unwrap();
        updated.kill_switch_active = true;
        updated.kill_switch_reason = "Emergency".to_string();
        updated.updated_at = chrono::Utc::now().timestamp();
        queries::upsert_global_policy(&db, &updated).unwrap();

        let fetched = queries::get_global_policy(&db).unwrap().unwrap();
        assert!(fetched.kill_switch_active);
        assert_eq!(fetched.kill_switch_reason, "Emergency");

        // Toggle off
        let mut deactivated = fetched;
        deactivated.kill_switch_active = false;
        deactivated.kill_switch_reason = String::new();
        deactivated.updated_at = chrono::Utc::now().timestamp();
        queries::upsert_global_policy(&db, &deactivated).unwrap();

        let fetched = queries::get_global_policy(&db).unwrap().unwrap();
        assert!(!fetched.kill_switch_active);
    }
}
