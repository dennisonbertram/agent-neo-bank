use tauri::State;

use crate::db::models::InvitationCode;
use crate::db::queries;
use crate::error::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn list_invitation_codes(
    state: State<'_, AppState>,
) -> Result<Vec<InvitationCode>, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || queries::list_all_invitation_codes(&db))
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn generate_invitation_code(
    state: State<'_, AppState>,
    label: String,
    expires_at: Option<i64>,
    max_uses: Option<i32>,
) -> Result<InvitationCode, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_part: String = (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'A' + idx - 10) as char
                }
            })
            .collect();
        let code = format!("INV-{}", random_part);
        let now = chrono::Utc::now().timestamp();

        let invitation = InvitationCode {
            code,
            created_at: now,
            expires_at,
            used_by: None,
            used_at: None,
            max_uses: max_uses.unwrap_or(1),
            use_count: 0,
            label,
        };

        queries::insert_invitation_code(&db, &invitation)?;
        Ok(invitation)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[tauri::command]
pub async fn revoke_invitation_code(
    state: State<'_, AppState>,
    code: String,
) -> Result<(), AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || queries::delete_invitation_code(&db, &code))
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use crate::db::models::InvitationCode;
    use crate::db::queries;
    use crate::db::schema::Database;

    fn create_test_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    #[test]
    fn test_list_invitation_codes_command() {
        let db = create_test_db();

        let inv1 = InvitationCode {
            code: "INV-TEST0001".to_string(),
            created_at: 1000000,
            expires_at: None,
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "Test 1".to_string(),
        };
        let inv2 = InvitationCode {
            code: "INV-TEST0002".to_string(),
            created_at: 1000001,
            expires_at: None,
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "Test 2".to_string(),
        };

        queries::insert_invitation_code(&db, &inv1).unwrap();
        queries::insert_invitation_code(&db, &inv2).unwrap();

        let all = queries::list_all_invitation_codes(&db).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_generate_invitation_code_format() {
        // Test the code format INV- + 8 alphanumeric chars
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_part: String = (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'A' + idx - 10) as char
                }
            })
            .collect();
        let code = format!("INV-{}", random_part);

        assert!(code.starts_with("INV-"));
        assert_eq!(code.len(), 12); // "INV-" (4) + 8 chars
        assert!(code[4..].chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_delete_invitation_code_command() {
        let db = create_test_db();

        let inv = InvitationCode {
            code: "INV-DELETE01".to_string(),
            created_at: 1000000,
            expires_at: None,
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "To delete".to_string(),
        };

        queries::insert_invitation_code(&db, &inv).unwrap();

        // Delete it
        queries::delete_invitation_code(&db, "INV-DELETE01").unwrap();

        // Verify it's gone
        let result = queries::get_invitation_code(&db, "INV-DELETE01");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_invitation_code() {
        let db = create_test_db();
        let result = queries::delete_invitation_code(&db, "INV-NOPE0000");
        assert!(result.is_err());
    }
}
