use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use super::rest_server::AppStateAxum;

/// Bearer token authentication middleware.
/// Extracts `Authorization: Bearer <token>`, validates via auth_service,
/// inserts the agent_id as an Extension, or returns 401.
pub async fn auth_middleware(
    State(state): State<Arc<AppStateAxum>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let token = match auth_header {
        Some(ref header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "missing_or_invalid_auth" })),
            )
                .into_response();
        }
    };

    // Rate limit by token prefix (first 12 chars or full token if shorter)
    let rate_key = if token.len() >= 12 {
        &token[..12]
    } else {
        token
    };
    if let Err(_) = state.rate_limiter.check(rate_key) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({ "error": "rate_limited" })),
        )
            .into_response();
    }

    match state.auth_service.validate_agent_token(token).await {
        Ok(agent_id) => {
            req.extensions_mut().insert(agent_id);
            next.run(req).await
        }
        Err(crate::error::AppError::AgentSuspended(msg)) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "agent_suspended", "message": msg })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid_token" })),
        )
            .into_response(),
    }
}
