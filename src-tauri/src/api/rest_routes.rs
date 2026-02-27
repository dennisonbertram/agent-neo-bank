use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::api::rest_server::AppStateAxum;
use crate::core::agent_registry::AgentRegistrationRequest;
use crate::core::approval_manager::ApprovalManager;
use crate::core::tx_processor::{SendRequest, TransactionResult};
use crate::db::models::ApprovalRequestType;
use crate::db::queries;
use crate::error::AppError;

// -------------------------------------------------------------------------
// Request / Response types
// -------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub purpose: Option<String>,
    pub agent_type: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub invitation_code: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct ApiSendRequest {
    pub to: String,
    pub amount: serde_json::Value,
    pub asset: Option<String>,
    pub description: Option<String>,
    pub memo: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Deserialize)]
pub struct LimitIncreaseRequest {
    pub new_per_tx_max: Option<String>,
    pub new_daily_cap: Option<String>,
    pub new_weekly_cap: Option<String>,
    pub new_monthly_cap: Option<String>,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct ListTxParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// -------------------------------------------------------------------------
// Error → HTTP status mapping
// -------------------------------------------------------------------------

fn error_to_response(err: AppError) -> impl IntoResponse {
    let (status, error_key) = match &err {
        AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
        AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
        AppError::PolicyViolation(_) => (StatusCode::FORBIDDEN, "policy_denied"),
        AppError::KillSwitchActive(_) => (StatusCode::FORBIDDEN, "kill_switch_active"),
        AppError::AgentSuspended(_) => (StatusCode::FORBIDDEN, "agent_suspended"),
        AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
        AppError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "invalid_input"),
        AppError::InvalidInvitationCode => (StatusCode::BAD_REQUEST, "invalid_invitation_code"),
        AppError::InvitationCodeExpired => (StatusCode::BAD_REQUEST, "invitation_code_expired"),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
    };

    (
        status,
        Json(serde_json::json!({
            "error": error_key,
            "message": err.to_string(),
        })),
    )
}

// -------------------------------------------------------------------------
// Handlers
// -------------------------------------------------------------------------

/// GET /v1/health — no auth required
pub async fn health(State(state): State<Arc<AppStateAxum>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "network": state.config.network,
        "mock_mode": state.config.mock_mode,
    }))
}

/// POST /v1/agents/register — no auth, rate-limited by invitation code
pub async fn register_agent(
    State(state): State<Arc<AppStateAxum>>,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Rate limit by invitation code
    if let Err(_) = state.rate_limiter.check(&body.invitation_code) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({ "error": "rate_limited" })),
        )
            .into_response();
    }

    let request = AgentRegistrationRequest {
        name: body.name,
        purpose: body.purpose.unwrap_or_else(|| "Not specified".to_string()),
        agent_type: body.agent_type.unwrap_or_else(|| "automated".to_string()),
        capabilities: body.capabilities.unwrap_or_default(),
        invitation_code: body.invitation_code,
        description: body.description,
        webhook_url: None,
    };

    let db = state.db.clone();
    let registry = state.agent_registry.clone();
    let result = tokio::task::spawn_blocking(move || registry.register(request))
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)));

    match result {
        Ok(Ok(reg_result)) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "agent_id": reg_result.agent_id,
                "status": reg_result.status,
            })),
        )
            .into_response(),
        Ok(Err(err)) => error_to_response(err).into_response(),
        Err(err) => error_to_response(err).into_response(),
    }
}

/// GET /v1/agents/register/{id}/status — no auth
pub async fn registration_status(
    State(state): State<Arc<AppStateAxum>>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    let registry = state.agent_registry.clone();
    let aid = agent_id.clone();
    let result = tokio::task::spawn_blocking(move || registry.get_status(&aid))
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)));

    match result {
        Ok(Ok(status)) => {
            // Also check if there's a token to deliver
            let registry2 = state.agent_registry.clone();
            let aid2 = agent_id.clone();
            let token = tokio::task::spawn_blocking(move || registry2.retrieve_token(&aid2))
                .await
                .ok()
                .and_then(|r| r.ok())
                .flatten();

            let mut response = serde_json::json!({
                "agent_id": status.agent_id,
                "status": status.status,
            });

            if let Some(t) = token {
                response["token"] = serde_json::Value::String(t);
            }

            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(Err(err)) => error_to_response(err).into_response(),
        Err(err) => error_to_response(err).into_response(),
    }
}

/// POST /v1/send — requires bearer auth
pub async fn send_transaction(
    State(state): State<Arc<AppStateAxum>>,
    Extension(agent_id): Extension<String>,
    Json(body): Json<ApiSendRequest>,
) -> impl IntoResponse {
    // Parse amount from flexible JSON value
    let amount_result: Result<Decimal, AppError> = match &body.amount {
        serde_json::Value::String(s) => s.parse().map_err(|_| {
            AppError::InvalidInput(format!("Invalid amount: {}", s))
        }),
        serde_json::Value::Number(n) => n.to_string().parse().map_err(|_| {
            AppError::InvalidInput(format!("Invalid amount: {}", n))
        }),
        other => Err(AppError::InvalidInput(format!(
            "Amount must be a string or number, got: {}",
            other
        ))),
    };

    let amount = match amount_result {
        Ok(a) => a,
        Err(e) => return error_to_response(e).into_response(),
    };

    if amount <= Decimal::ZERO {
        return error_to_response(AppError::InvalidInput("Amount must be positive".to_string()))
            .into_response();
    }

    let request = SendRequest {
        to: body.to,
        amount,
        asset: body.asset,
        description: body.description,
        memo: body.memo,
        webhook_url: body.webhook_url,
    };

    match state.tx_processor.process_send(&agent_id, request).await {
        Ok(TransactionResult::Accepted { tx_id, status }) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "tx_id": tx_id,
                "status": status,
            })),
        )
            .into_response(),
        Ok(TransactionResult::Denied { tx_id, reason }) => {
            // Map deny reasons to appropriate status codes
            let status_code = if reason.contains("kill switch") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::FORBIDDEN
            };
            let error_key = if reason.contains("kill switch") {
                "kill_switch_active"
            } else {
                "policy_denied"
            };
            (
                status_code,
                Json(serde_json::json!({
                    "tx_id": tx_id,
                    "error": error_key,
                    "reason": reason,
                })),
            )
                .into_response()
        }
        Err(err) => error_to_response(err).into_response(),
    }
}

/// GET /v1/balance — requires bearer auth
pub async fn get_balance(
    State(state): State<Arc<AppStateAxum>>,
    Extension(agent_id): Extension<String>,
) -> impl IntoResponse {
    match state.wallet_service.get_balance_for_agent(&agent_id).await {
        Ok(resp) => (StatusCode::OK, Json(serde_json::json!(resp))).into_response(),
        Err(err) => error_to_response(err).into_response(),
    }
}

/// GET /v1/transactions — requires bearer auth, paginated
pub async fn list_transactions(
    State(state): State<Arc<AppStateAxum>>,
    Extension(agent_id): Extension<String>,
    Query(params): Query<ListTxParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100).max(1);
    let offset = params.offset.unwrap_or(0).max(0);
    let status = params.status.clone();

    let db = state.db.clone();
    let aid = agent_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        queries::list_transactions_paginated(&db, Some(&aid), status.as_deref(), limit, offset)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)));

    match result {
        Ok(Ok((txs, total))) => {
            let response = PaginatedResponse {
                data: txs,
                total,
                limit,
                offset,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(Err(err)) => error_to_response(err).into_response(),
        Err(err) => error_to_response(err).into_response(),
    }
}

/// GET /v1/transactions/{tx_id} — requires bearer auth
pub async fn get_transaction_handler(
    State(state): State<Arc<AppStateAxum>>,
    Extension(agent_id): Extension<String>,
    Path(tx_id): Path<String>,
) -> impl IntoResponse {
    let db = state.db.clone();
    let tid = tx_id.clone();
    let result = tokio::task::spawn_blocking(move || queries::get_transaction(&db, &tid))
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)));

    match result {
        Ok(Ok(tx)) => {
            // Verify the transaction belongs to this agent
            if tx.agent_id.as_deref() != Some(&agent_id) {
                return error_to_response(AppError::NotFound(format!(
                    "Transaction not found: {}",
                    tx_id
                )))
                .into_response();
            }
            (StatusCode::OK, Json(tx)).into_response()
        }
        Ok(Err(err)) => error_to_response(err).into_response(),
        Err(err) => error_to_response(err).into_response(),
    }
}

/// POST /v1/limits/request-increase — requires bearer auth
pub async fn request_limit_increase(
    State(state): State<Arc<AppStateAxum>>,
    Extension(agent_id): Extension<String>,
    Json(body): Json<LimitIncreaseRequest>,
) -> impl IntoResponse {
    if body.reason.trim().is_empty() {
        return error_to_response(AppError::InvalidInput(
            "Reason is required and cannot be empty".to_string(),
        ))
        .into_response();
    }

    // At least one new limit must be proposed
    if body.new_per_tx_max.is_none()
        && body.new_daily_cap.is_none()
        && body.new_weekly_cap.is_none()
        && body.new_monthly_cap.is_none()
    {
        return error_to_response(AppError::InvalidInput(
            "At least one new limit must be proposed".to_string(),
        ))
        .into_response();
    }

    let db = state.db.clone();
    let aid = agent_id.clone();

    let result = tokio::task::spawn_blocking(move || {
        // Get current spending policy
        let current_policy = queries::get_spending_policy(&db, &aid)?;

        // Build payload with current vs proposed limits
        let payload = serde_json::json!({
            "current": {
                "per_tx_max": current_policy.per_tx_max,
                "daily_cap": current_policy.daily_cap,
                "weekly_cap": current_policy.weekly_cap,
                "monthly_cap": current_policy.monthly_cap,
            },
            "proposed": {
                "new_per_tx_max": body.new_per_tx_max,
                "new_daily_cap": body.new_daily_cap,
                "new_weekly_cap": body.new_weekly_cap,
                "new_monthly_cap": body.new_monthly_cap,
            },
            "reason": body.reason,
        });

        // Create approval request of type LimitIncrease
        let manager = ApprovalManager::new(db);
        manager.create_request(
            &aid,
            ApprovalRequestType::LimitIncrease,
            payload,
            None,
            None,
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)));

    match result {
        Ok(Ok(approval)) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "approval_id": approval.id,
                "status": "pending",
                "message": "Limit increase request submitted for approval",
            })),
        )
            .into_response(),
        Ok(Err(err)) => error_to_response(err).into_response(),
        Err(err) => error_to_response(err).into_response(),
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::rate_limiter::RateLimiter;
    use crate::api::rest_server::{ApiServer, AppStateAxum};
    use crate::cli::executor::MockCliExecutor;
    use crate::config::AppConfig;
    use crate::core::agent_registry::AgentRegistry;
    use crate::core::auth_service::AuthService;
    use crate::core::tx_processor::TransactionProcessor;
    use crate::core::wallet_service::WalletService;
    use crate::db::models::*;
    use crate::db::queries::{insert_agent, insert_spending_policy, insert_transaction};
    use crate::db::schema::Database;
    use crate::test_helpers::{
        create_test_agent, create_test_invitation, create_test_spending_policy, setup_test_db,
    };

    use axum::body::Body;
    use axum::http::Request;
    use rust_decimal_macros::dec;
    use std::time::Duration;
    use tower::util::ServiceExt;

    /// Helper: create a full test AppStateAxum with in-memory DB and mock CLI.
    fn create_test_state() -> Arc<AppStateAxum> {
        create_test_state_with_config(AppConfig::default_test())
    }

    fn create_test_state_with_config(config: AppConfig) -> Arc<AppStateAxum> {
        let db = setup_test_db();
        let cli: Arc<dyn crate::cli::executor::CliExecutable> =
            Arc::new(MockCliExecutor::with_defaults());
        let auth_service = Arc::new(AuthService::new(
            cli.clone(),
            db.clone(),
            Duration::from_secs(300),
        ));
        let agent_registry = Arc::new(AgentRegistry::new(db.clone(), config.clone()));
        let (tx_processor, _rx) =
            TransactionProcessor::new(db.clone(), cli.clone(), dec!(10000), 16);
        let tx_processor = Arc::new(tx_processor);
        let wallet_service = Arc::new(WalletService::new(
            cli.clone(),
            db.clone(),
            Duration::from_secs(0),
        ));
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_requests_per_minute));

        Arc::new(AppStateAxum {
            db,
            auth_service,
            agent_registry,
            tx_processor,
            wallet_service,
            rate_limiter,
            config,
        })
    }

    /// Helper: create an active agent with a known argon2 token hash.
    /// Returns (state, agent_id, raw_token).
    fn create_state_with_active_agent() -> (Arc<AppStateAxum>, String, String) {
        let state = create_test_state();
        let (agent_id, raw_token) = create_active_agent_in_state(&state);
        (state, agent_id, raw_token)
    }

    /// Helper: insert an active agent with real argon2 hash into the given state's DB.
    fn create_active_agent_in_state(state: &Arc<AppStateAxum>) -> (String, String) {
        let raw_token = "anb_test_token_for_api_tests_1234";
        let argon2_hash = hash_token_argon2(raw_token);
        let mut agent = create_test_agent("ApiTestBot", AgentStatus::Active);
        agent.api_token_hash = Some(argon2_hash);
        agent.token_prefix = Some(raw_token[..12].to_string());
        insert_agent(&state.db, &agent).unwrap();

        // Also insert a spending policy so sends can work
        let policy = create_test_spending_policy(
            &agent.id, "100", "1000", "5000", "20000", "50",
        );
        insert_spending_policy(&state.db, &policy).unwrap();

        (agent.id, raw_token.to_string())
    }

    fn hash_token_argon2(token: &str) -> String {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::SaltString;
        use argon2::{Argon2, PasswordHasher};
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(token.as_bytes(), &salt)
            .expect("Failed to hash token")
            .to_string()
    }

    /// Helper: make a request with bearer token.
    fn bearer_request(method: &str, uri: &str, token: &str, body: Body) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(body)
            .unwrap()
    }

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    // =================================================================
    // Test 1: test_api_health_check_no_auth
    // =================================================================
    #[tokio::test]
    async fn test_api_health_check_no_auth() {
        let state = create_test_state();
        let app = ApiServer::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body = body_json(response).await;
        assert_eq!(body["status"], "ok");
        assert!(body["version"].is_string());
        assert!(body["network"].is_string());
        assert!(body.get("mock_mode").is_some());
    }

    // =================================================================
    // Test 2: test_api_send_valid_request_returns_202
    // =================================================================
    #[tokio::test]
    async fn test_api_send_valid_request_returns_202() {
        let (state, _agent_id, token) = create_state_with_active_agent();
        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "to": "0xRecipient123",
            "amount": "10.5",
            "asset": "USDC"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/send",
                &token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 202);
        let body = body_json(response).await;
        assert!(body["tx_id"].is_string());
        assert_eq!(body["status"], "executing");
    }

    // =================================================================
    // Test 3: test_api_send_missing_auth_returns_401
    // =================================================================
    #[tokio::test]
    async fn test_api_send_missing_auth_returns_401() {
        let state = create_test_state();
        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "to": "0xRecipient123",
            "amount": "10.5"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/send")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }

    // =================================================================
    // Test 4: test_api_send_expired_token_returns_401
    // =================================================================
    #[tokio::test]
    async fn test_api_send_expired_token_returns_401() {
        let state = create_test_state();
        // Insert a revoked agent with token
        let raw_token = "anb_revoked_agent_token_test1234";
        let argon2_hash = hash_token_argon2(raw_token);
        let mut agent = create_test_agent("RevokedBot", AgentStatus::Revoked);
        agent.api_token_hash = Some(argon2_hash);
        insert_agent(&state.db, &agent).unwrap();

        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "to": "0xRecipient123",
            "amount": "10"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/send",
                raw_token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }

    // =================================================================
    // Test 5: test_api_send_invalid_amount_returns_400
    // =================================================================
    #[tokio::test]
    async fn test_api_send_invalid_amount_returns_400() {
        let (state, _agent_id, token) = create_state_with_active_agent();
        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "to": "0xRecipient123",
            "amount": "not-a-number"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/send",
                &token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 400);
    }

    // =================================================================
    // Test 6: test_api_send_missing_to_field_returns_400
    // =================================================================
    #[tokio::test]
    async fn test_api_send_missing_to_field_returns_400() {
        let (state, _agent_id, token) = create_state_with_active_agent();
        let app = ApiServer::router(state);

        // Missing "to" field entirely
        let body = serde_json::json!({
            "amount": "10"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/send",
                &token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        // Axum will reject this with 422 for deserialization failure
        assert!(
            response.status() == 400 || response.status() == 422,
            "Expected 400 or 422, got {}",
            response.status()
        );
    }

    // =================================================================
    // Test 7: test_api_send_policy_denied_returns_403
    // =================================================================
    #[tokio::test]
    async fn test_api_send_policy_denied_returns_403() {
        let state = create_test_state();

        // Create agent with very low per_tx_max
        let raw_token = "anb_policy_denied_token_test1234";
        let argon2_hash = hash_token_argon2(raw_token);
        let mut agent = create_test_agent("PolicyDeniedBot", AgentStatus::Active);
        agent.api_token_hash = Some(argon2_hash);
        agent.token_prefix = Some(raw_token[..12].to_string());
        insert_agent(&state.db, &agent).unwrap();

        // Very low per_tx_max of 5
        let policy = create_test_spending_policy(&agent.id, "5", "1000", "5000", "20000", "50");
        insert_spending_policy(&state.db, &policy).unwrap();

        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "to": "0xRecipient123",
            "amount": "50"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/send",
                raw_token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 403);
        let resp_body = body_json(response).await;
        assert_eq!(resp_body["error"], "policy_denied");
    }

    // =================================================================
    // Test 8: test_api_send_kill_switch_returns_403
    // =================================================================
    #[tokio::test]
    async fn test_api_send_kill_switch_returns_403() {
        let state = create_test_state();
        let (agent_id, raw_token) = create_active_agent_in_state(&state);

        // Activate kill switch via global policy
        use crate::db::models::GlobalPolicy;
        use crate::db::queries::upsert_global_policy;
        let policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "0".to_string(),
            weekly_cap: "0".to_string(),
            monthly_cap: "0".to_string(),
            min_reserve_balance: "0".to_string(),
            kill_switch_active: true,
            kill_switch_reason: "Emergency maintenance".to_string(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        upsert_global_policy(&state.db, &policy).unwrap();

        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "to": "0xRecipient123",
            "amount": "10"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/send",
                &raw_token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 403);
        let resp_body = body_json(response).await;
        assert_eq!(resp_body["error"], "kill_switch_active");
    }

    // =================================================================
    // Test 9: test_api_register_with_valid_code_returns_201
    // =================================================================
    #[tokio::test]
    async fn test_api_register_with_valid_code_returns_201() {
        let state = create_test_state();

        // Insert a valid invitation code
        let invitation = create_test_invitation("INV-api-test-001", "API test");
        crate::db::queries::insert_invitation_code(&state.db, &invitation).unwrap();

        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "name": "TestApiBot",
            "invitation_code": "INV-api-test-001",
            "purpose": "Testing the API",
            "agent_type": "automated"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/agents/register")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 201);
        let resp_body = body_json(response).await;
        assert!(resp_body["agent_id"].is_string());
        assert_eq!(resp_body["status"], "pending");
    }

    // =================================================================
    // Test 10: test_api_register_invalid_code_returns_400
    // =================================================================
    #[tokio::test]
    async fn test_api_register_invalid_code_returns_400() {
        let state = create_test_state();
        let app = ApiServer::router(state);

        let body = serde_json::json!({
            "name": "TestBadCodeBot",
            "invitation_code": "INV-bogus-code"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/agents/register")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 400);
    }

    // =================================================================
    // Test 11: test_api_list_transactions_pagination
    // =================================================================
    #[tokio::test]
    async fn test_api_list_transactions_pagination() {
        let (state, agent_id, token) = create_state_with_active_agent();

        // Insert 50 transactions for this agent
        for i in 0..50 {
            let tx = Transaction {
                id: format!("tx-pagination-{}", i),
                agent_id: Some(agent_id.clone()),
                tx_type: TxType::Send,
                amount: "10".to_string(),
                asset: "USDC".to_string(),
                recipient: Some("0xRecipient".to_string()),
                sender: None,
                chain_tx_hash: Some(format!("0xhash{}", i)),
                status: TxStatus::Confirmed,
                category: "test".to_string(),
                memo: String::new(),
                description: String::new(),
                service_name: String::new(),
                service_url: String::new(),
                reason: String::new(),
                webhook_url: None,
                error_message: None,
                period_daily: "daily:2026-02-27".to_string(),
                period_weekly: "weekly:2026-W09".to_string(),
                period_monthly: "monthly:2026-02".to_string(),
                created_at: 1000000 + i,
                updated_at: 1000000 + i,
            };
            insert_transaction(&state.db, &tx).unwrap();
        }

        let app = ApiServer::router(state);

        let response = app
            .oneshot(bearer_request(
                "GET",
                "/v1/transactions?limit=10&offset=20",
                &token,
                Body::empty(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let resp_body = body_json(response).await;
        assert_eq!(resp_body["total"], 50);
        assert_eq!(resp_body["limit"], 10);
        assert_eq!(resp_body["offset"], 20);
        let data = resp_body["data"].as_array().unwrap();
        assert_eq!(data.len(), 10);
    }

    // =================================================================
    // Test 12: test_api_list_transactions_filter_by_status
    // =================================================================
    #[tokio::test]
    async fn test_api_list_transactions_filter_by_status() {
        let (state, agent_id, token) = create_state_with_active_agent();

        // Insert 5 confirmed and 3 failed transactions
        for i in 0..5 {
            let tx = Transaction {
                id: format!("tx-confirmed-{}", i),
                agent_id: Some(agent_id.clone()),
                tx_type: TxType::Send,
                amount: "10".to_string(),
                asset: "USDC".to_string(),
                recipient: Some("0xRecipient".to_string()),
                sender: None,
                chain_tx_hash: Some(format!("0xhash{}", i)),
                status: TxStatus::Confirmed,
                category: "test".to_string(),
                memo: String::new(),
                description: String::new(),
                service_name: String::new(),
                service_url: String::new(),
                reason: String::new(),
                webhook_url: None,
                error_message: None,
                period_daily: "daily:2026-02-27".to_string(),
                period_weekly: "weekly:2026-W09".to_string(),
                period_monthly: "monthly:2026-02".to_string(),
                created_at: 1000000 + i,
                updated_at: 1000000 + i,
            };
            insert_transaction(&state.db, &tx).unwrap();
        }
        for i in 0..3 {
            let tx = Transaction {
                id: format!("tx-failed-{}", i),
                agent_id: Some(agent_id.clone()),
                tx_type: TxType::Send,
                amount: "10".to_string(),
                asset: "USDC".to_string(),
                recipient: Some("0xRecipient".to_string()),
                sender: None,
                chain_tx_hash: None,
                status: TxStatus::Failed,
                category: "test".to_string(),
                memo: String::new(),
                description: String::new(),
                service_name: String::new(),
                service_url: String::new(),
                reason: String::new(),
                webhook_url: None,
                error_message: Some("CLI error".to_string()),
                period_daily: "daily:2026-02-27".to_string(),
                period_weekly: "weekly:2026-W09".to_string(),
                period_monthly: "monthly:2026-02".to_string(),
                created_at: 2000000 + i,
                updated_at: 2000000 + i,
            };
            insert_transaction(&state.db, &tx).unwrap();
        }

        let app = ApiServer::router(state);

        let response = app
            .oneshot(bearer_request(
                "GET",
                "/v1/transactions?status=confirmed",
                &token,
                Body::empty(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let resp_body = body_json(response).await;
        assert_eq!(resp_body["total"], 5);
        let data = resp_body["data"].as_array().unwrap();
        assert_eq!(data.len(), 5);
        for tx in data {
            assert_eq!(tx["status"], "confirmed");
        }
    }

    // =================================================================
    // Test 13: test_api_rate_limiter_blocks_excess_requests
    // =================================================================
    #[tokio::test]
    async fn test_api_rate_limiter_blocks_excess_requests() {
        let mut config = AppConfig::default_test();
        config.rate_limit_requests_per_minute = 60;
        let state = create_test_state_with_config(config);
        let (agent_id, raw_token) = create_active_agent_in_state(&state);

        // Make 60 requests (should all pass via auth middleware rate limiting)
        for i in 0..60 {
            let app = ApiServer::router(state.clone());
            let response = app
                .oneshot(bearer_request(
                    "GET",
                    "/v1/balance",
                    &raw_token,
                    Body::empty(),
                ))
                .await
                .unwrap();

            assert!(
                response.status() == 200 || response.status() == 429,
                "Request {} got unexpected status: {}",
                i + 1,
                response.status()
            );
        }

        // 61st request should be rate limited
        let app = ApiServer::router(state.clone());
        let response = app
            .oneshot(bearer_request(
                "GET",
                "/v1/balance",
                &raw_token,
                Body::empty(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 429, "61st request should be rate limited");
    }

    // =================================================================
    // Test 14: test_limit_increase_request_creates_approval
    // =================================================================
    #[tokio::test]
    async fn test_limit_increase_request_creates_approval() {
        let (state, agent_id, token) = create_state_with_active_agent();
        let app = ApiServer::router(state.clone());

        let body = serde_json::json!({
            "new_daily_cap": "2000",
            "reason": "Need higher limits for operations"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/limits/request-increase",
                &token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 202);
        let resp_body = body_json(response).await;
        assert!(resp_body["approval_id"].is_string());
        assert_eq!(resp_body["status"], "pending");

        // Verify the approval request was created in the DB
        let approval_id = resp_body["approval_id"].as_str().unwrap();
        let approval =
            crate::db::queries::get_approval_request(&state.db, approval_id).unwrap();
        assert_eq!(approval.agent_id, agent_id);
        assert_eq!(
            approval.request_type,
            crate::db::models::ApprovalRequestType::LimitIncrease
        );
        assert_eq!(approval.status, crate::db::models::ApprovalStatus::Pending);

        // Verify the payload contains current and proposed limits
        let payload: serde_json::Value =
            serde_json::from_str(&approval.payload).unwrap();
        assert!(payload.get("current").is_some());
        assert!(payload.get("proposed").is_some());
        assert_eq!(payload["reason"], "Need higher limits for operations");
        assert_eq!(payload["proposed"]["new_daily_cap"], "2000");
    }

    // =================================================================
    // Test 15: test_limit_increase_approval_updates_policy
    // =================================================================
    #[tokio::test]
    async fn test_limit_increase_approval_updates_policy() {
        let (state, agent_id, _token) = create_state_with_active_agent();

        // Get current policy to verify initial state
        let old_policy =
            crate::db::queries::get_spending_policy(&state.db, &agent_id).unwrap();
        assert_eq!(old_policy.daily_cap, "1000");

        // Create a limit_increase approval request
        let manager =
            crate::core::approval_manager::ApprovalManager::new(state.db.clone());
        let payload = serde_json::json!({
            "current": {
                "per_tx_max": old_policy.per_tx_max,
                "daily_cap": old_policy.daily_cap,
                "weekly_cap": old_policy.weekly_cap,
                "monthly_cap": old_policy.monthly_cap,
            },
            "proposed": {
                "new_daily_cap": "5000",
                "new_per_tx_max": "500",
            },
            "reason": "Need more capacity"
        });
        let approval = manager
            .create_request(
                &agent_id,
                crate::db::models::ApprovalRequestType::LimitIncrease,
                payload,
                None,
                None,
            )
            .unwrap();

        // Resolve as approved -- use the same logic as the command
        let db = state.db.clone();
        let resolved = manager
            .resolve(&approval.id, crate::db::models::ApprovalStatus::Approved, "user")
            .unwrap();

        // Apply side effect (simulating what resolve_approval command does)
        if let Ok(payload) =
            serde_json::from_str::<serde_json::Value>(&resolved.payload)
        {
            let proposed = payload.get("proposed").cloned().unwrap_or(payload.clone());
            if let Ok(mut policy) =
                crate::db::queries::get_spending_policy(&db, &resolved.agent_id)
            {
                if let Some(v) = proposed.get("new_per_tx_max").and_then(|v| v.as_str())
                {
                    policy.per_tx_max = v.to_string();
                }
                if let Some(v) = proposed.get("new_daily_cap").and_then(|v| v.as_str()) {
                    policy.daily_cap = v.to_string();
                }
                if let Some(v) = proposed.get("new_weekly_cap").and_then(|v| v.as_str())
                {
                    policy.weekly_cap = v.to_string();
                }
                if let Some(v) =
                    proposed.get("new_monthly_cap").and_then(|v| v.as_str())
                {
                    policy.monthly_cap = v.to_string();
                }
                policy.updated_at = chrono::Utc::now().timestamp();
                crate::db::queries::update_spending_policy(&db, &policy).unwrap();
            }
        }

        // Verify the spending policy was updated
        let updated_policy =
            crate::db::queries::get_spending_policy(&state.db, &agent_id).unwrap();
        assert_eq!(updated_policy.daily_cap, "5000");
        assert_eq!(updated_policy.per_tx_max, "500");
        // weekly_cap and monthly_cap should remain unchanged
        assert_eq!(updated_policy.weekly_cap, old_policy.weekly_cap);
        assert_eq!(updated_policy.monthly_cap, old_policy.monthly_cap);
    }

    // =================================================================
    // Test 16: test_limit_increase_denial_preserves_old_limits
    // =================================================================
    #[tokio::test]
    async fn test_limit_increase_denial_preserves_old_limits() {
        let (state, agent_id, _token) = create_state_with_active_agent();

        let old_policy =
            crate::db::queries::get_spending_policy(&state.db, &agent_id).unwrap();

        // Create and deny a limit_increase approval
        let manager =
            crate::core::approval_manager::ApprovalManager::new(state.db.clone());
        let payload = serde_json::json!({
            "proposed": {
                "new_daily_cap": "999999",
            },
            "reason": "Want more"
        });
        let approval = manager
            .create_request(
                &agent_id,
                crate::db::models::ApprovalRequestType::LimitIncrease,
                payload,
                None,
                None,
            )
            .unwrap();

        // Resolve as denied
        manager
            .resolve(&approval.id, crate::db::models::ApprovalStatus::Denied, "admin")
            .unwrap();

        // Verify spending policy is unchanged
        let current_policy =
            crate::db::queries::get_spending_policy(&state.db, &agent_id).unwrap();
        assert_eq!(current_policy.daily_cap, old_policy.daily_cap);
        assert_eq!(current_policy.per_tx_max, old_policy.per_tx_max);
        assert_eq!(current_policy.weekly_cap, old_policy.weekly_cap);
        assert_eq!(current_policy.monthly_cap, old_policy.monthly_cap);
    }

    // =================================================================
    // Test 17: test_limit_increase_request_requires_reason
    // =================================================================
    #[tokio::test]
    async fn test_limit_increase_request_requires_reason() {
        let (state, _agent_id, token) = create_state_with_active_agent();
        let app = ApiServer::router(state);

        // Empty reason
        let body = serde_json::json!({
            "new_daily_cap": "2000",
            "reason": "   "
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/limits/request-increase",
                &token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 400);
    }

    // =================================================================
    // Test 18: test_limit_increase_request_requires_at_least_one_limit
    // =================================================================
    #[tokio::test]
    async fn test_limit_increase_request_requires_at_least_one_limit() {
        let (state, _agent_id, token) = create_state_with_active_agent();
        let app = ApiServer::router(state);

        // No new limits proposed
        let body = serde_json::json!({
            "reason": "I want more"
        });

        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/limits/request-increase",
                &token,
                Body::from(serde_json::to_string(&body).unwrap()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 400);
    }
}
