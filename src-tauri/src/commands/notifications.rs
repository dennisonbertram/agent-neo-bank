use tauri::State;

use crate::core::notification::NotificationService;
use crate::db::models::NotificationPreferences;
use crate::error::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_notification_preferences(
    state: State<'_, AppState>,
) -> Result<NotificationPreferences, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let service = NotificationService::new(db);
        service.get_preferences()
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn update_notification_preferences(
    state: State<'_, AppState>,
    prefs: NotificationPreferences,
) -> Result<(), AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let service = NotificationService::new(db);
        service.update_preferences(&prefs)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use crate::core::notification::NotificationService;
    use crate::db::models::NotificationPreferences;
    use crate::db::schema::Database;

    fn create_test_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    #[test]
    fn test_get_notification_preferences_returns_defaults() {
        let db = create_test_db();
        let service = NotificationService::new(std::sync::Arc::new(db));
        let prefs = service.get_preferences().unwrap();
        assert_eq!(prefs.id, "default");
        assert!(prefs.enabled);
        assert!(prefs.on_all_tx);
        assert!(!prefs.on_large_tx);
        assert_eq!(prefs.large_tx_threshold, "100");
        assert!(prefs.on_errors);
        assert!(prefs.on_limit_requests);
        assert!(prefs.on_agent_registration);
    }

    #[test]
    fn test_update_and_get_notification_preferences() {
        let db = create_test_db();
        let service = NotificationService::new(std::sync::Arc::new(db));

        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: false,
            on_large_tx: true,
            large_tx_threshold: "500".to_string(),
            on_errors: false,
            on_limit_requests: true,
            on_agent_registration: false,
        };

        service.update_preferences(&prefs).unwrap();
        let fetched = service.get_preferences().unwrap();

        assert_eq!(fetched.id, "default");
        assert!(fetched.enabled);
        assert!(!fetched.on_all_tx);
        assert!(fetched.on_large_tx);
        assert_eq!(fetched.large_tx_threshold, "500");
        assert!(!fetched.on_errors);
        assert!(fetched.on_limit_requests);
        assert!(!fetched.on_agent_registration);
    }
}
