use std::sync::Arc;
use std::time::Duration;

use agent_neo_bank_lib::api::rest_server::{ApiServer, AppStateAxum};
use agent_neo_bank_lib::api::rate_limiter::RateLimiter;
use agent_neo_bank_lib::cli::executor::MockCliExecutor;
use agent_neo_bank_lib::config::AppConfig;
use agent_neo_bank_lib::core::agent_registry::AgentRegistry;
use agent_neo_bank_lib::core::auth_service::AuthService;
use agent_neo_bank_lib::core::tx_processor::TransactionProcessor;
use agent_neo_bank_lib::core::wallet_service::WalletService;
use agent_neo_bank_lib::db::schema::Database;
use agent_neo_bank_lib::test_helpers::setup_test_db;

use axum::body::Body;
use axum::Router;
use http::Request;
use rust_decimal_macros::dec;
pub use tower::ServiceExt;

/// Build a full test app with default test config.
/// Returns (Router, Arc<AppStateAxum>) so tests can also access state directly.
pub fn create_test_app() -> (Router, Arc<AppStateAxum>) {
    create_test_app_with_config(AppConfig::default_test())
}

/// Build a full test app with a custom config.
pub fn create_test_app_with_config(config: AppConfig) -> (Router, Arc<AppStateAxum>) {
    let db = setup_test_db();
    create_test_app_with_db_and_config(db, config)
}

/// Build a full test app with an existing DB and config.
pub fn create_test_app_with_db_and_config(
    db: Arc<Database>,
    config: AppConfig,
) -> (Router, Arc<AppStateAxum>) {
    let cli: Arc<dyn agent_neo_bank_lib::cli::executor::CliExecutable> =
        Arc::new(MockCliExecutor::with_defaults());
    create_test_app_with_db_config_and_cli(db, config, cli)
}

/// Build a full test app with an existing DB, config, and custom CLI executor.
pub fn create_test_app_with_db_config_and_cli(
    db: Arc<Database>,
    config: AppConfig,
    cli: Arc<dyn agent_neo_bank_lib::cli::executor::CliExecutable>,
) -> (Router, Arc<AppStateAxum>) {
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

    let state = Arc::new(AppStateAxum {
        db,
        auth_service,
        agent_registry,
        tx_processor,
        wallet_service,
        rate_limiter,
        config,
    });

    let router = ApiServer::router(state.clone());
    (router, state)
}

/// Build an HTTP request with Bearer token authentication.
pub fn bearer_request(method: &str, uri: &str, token: &str, body: Body) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(body)
        .unwrap()
}

/// Parse a response body into a serde_json::Value.
pub async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Register an agent through the API, approve it, and retrieve the token.
/// Returns (agent_id, raw_token).
pub async fn register_approve_and_get_token(
    state: &Arc<AppStateAxum>,
    invitation_code: &str,
    agent_name: &str,
) -> (String, String) {
    use agent_neo_bank_lib::test_helpers::create_test_invitation;
    use agent_neo_bank_lib::db::queries::insert_invitation_code;

    // 1. Insert invitation code
    let invitation = create_test_invitation(invitation_code, "Integration test");
    insert_invitation_code(&state.db, &invitation).unwrap();

    // 2. Register via API
    let app = ApiServer::router(state.clone());
    let body = serde_json::json!({
        "name": agent_name,
        "invitation_code": invitation_code,
        "purpose": "Integration testing",
        "agent_type": "automated",
        "capabilities": ["send"],
        "description": "Integration test agent"
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

    assert_eq!(response.status(), 201, "Registration should succeed");
    let resp_body = body_json(response).await;
    let agent_id = resp_body["agent_id"].as_str().unwrap().to_string();

    // 3. Approve the agent (direct access to registry)
    let raw_token = state.agent_registry.approve(&agent_id).unwrap();

    (agent_id, raw_token)
}

/// Register, approve, get token, and set a custom spending policy.
/// Returns (agent_id, raw_token).
pub async fn register_agent_with_policy(
    state: &Arc<AppStateAxum>,
    invitation_code: &str,
    agent_name: &str,
    per_tx_max: &str,
    daily_cap: &str,
    weekly_cap: &str,
    monthly_cap: &str,
    auto_approve_max: &str,
) -> (String, String) {
    let (agent_id, token) =
        register_approve_and_get_token(state, invitation_code, agent_name).await;

    // Update spending policy using raw SQL since the registry already created a zero policy
    // and there's no update function available.
    // Scope the connection so it's dropped before returning (pool max_size=1 for in-memory DB).
    {
        let conn = state.db.get_connection().unwrap();
        conn.execute(
            "UPDATE spending_policies SET per_tx_max = ?1, daily_cap = ?2, weekly_cap = ?3, monthly_cap = ?4, auto_approve_max = ?5, updated_at = ?6 WHERE agent_id = ?7",
            rusqlite::params![
                per_tx_max,
                daily_cap,
                weekly_cap,
                monthly_cap,
                auto_approve_max,
                chrono::Utc::now().timestamp(),
                agent_id,
            ],
        ).unwrap();
    }

    (agent_id, token)
}
