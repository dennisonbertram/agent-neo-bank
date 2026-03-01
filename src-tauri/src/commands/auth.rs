use tauri::State;

use crate::core::auth_service::{AuthResult, AuthStatus};
use crate::error::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn auth_login(
    email: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    let result = state.auth_service.login(&email).await?;
    match result {
        AuthResult::OtpSent { flow_id } => {
            Ok(serde_json::json!({ "status": "otp_sent", "flow_id": flow_id }))
        }
        AuthResult::Verified => Ok(serde_json::json!({ "status": "verified" })),
        AuthResult::AlreadyAuthenticated => {
            Ok(serde_json::json!({ "status": "already_authenticated" }))
        }
    }
}

#[tauri::command]
pub async fn auth_verify(
    otp: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    let result = state.auth_service.verify(&otp).await?;
    match result {
        AuthResult::Verified => Ok(serde_json::json!({ "status": "verified" })),
        AuthResult::OtpSent { flow_id } => {
            Ok(serde_json::json!({ "status": "otp_sent", "flow_id": flow_id }))
        }
        AuthResult::AlreadyAuthenticated => {
            Ok(serde_json::json!({ "status": "already_authenticated" }))
        }
    }
}

#[tauri::command]
pub async fn auth_status(state: State<'_, AppState>) -> Result<AuthStatus, AppError> {
    state.auth_service.check_status().await
}

#[tauri::command]
pub async fn auth_logout(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    state.auth_service.logout().await?;
    Ok(serde_json::json!({ "status": "logged_out" }))
}
