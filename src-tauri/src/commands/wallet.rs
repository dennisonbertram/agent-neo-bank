use crate::error::AppError;

// TODO: implement in Phase 1a

#[tauri::command]
pub async fn get_balance() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({ "status": "not_implemented" }))
}

#[tauri::command]
pub async fn get_address() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({ "status": "not_implemented" }))
}
