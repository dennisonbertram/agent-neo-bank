use crate::error::AppError;

// TODO: implement in Phase 1a

#[tauri::command]
pub async fn auth_login(email: String) -> Result<serde_json::Value, AppError> {
    let _ = email;
    Ok(serde_json::json!({ "status": "not_implemented" }))
}

#[tauri::command]
pub async fn auth_verify(email: String, otp: String) -> Result<serde_json::Value, AppError> {
    let _ = (email, otp);
    Ok(serde_json::json!({ "status": "not_implemented" }))
}

#[tauri::command]
pub async fn auth_status() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({ "status": "not_implemented" }))
}

#[tauri::command]
pub async fn auth_logout() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({ "status": "not_implemented" }))
}
