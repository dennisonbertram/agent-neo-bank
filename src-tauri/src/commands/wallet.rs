use tauri::State;

use crate::error::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn get_balance(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let cached = state.wallet_service.get_balance().await?;
    let balance_usd = cached.balances.get("USDC").map(|b| b.formatted.clone());

    Ok(serde_json::json!({
        "balance": balance_usd,
        "asset": "USDC",
        "balances": cached.balances,
        "balance_visible": true,
        "cached": true,
    }))
}

#[tauri::command]
pub async fn get_address(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let address = state.wallet_service.get_address().await?;
    Ok(serde_json::json!({ "address": address }))
}
