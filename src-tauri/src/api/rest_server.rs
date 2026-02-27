use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::api::auth_middleware::auth_middleware;
use crate::api::rate_limiter::RateLimiter;
use crate::api::rest_routes;
use crate::config::AppConfig;
use crate::core::agent_registry::AgentRegistry;
use crate::core::auth_service::AuthService;
use crate::core::tx_processor::TransactionProcessor;
use crate::core::wallet_service::WalletService;
use crate::db::schema::Database;
use crate::error::AppError;

// -------------------------------------------------------------------------
// AppStateAxum
// -------------------------------------------------------------------------

pub struct AppStateAxum {
    pub db: Arc<Database>,
    pub auth_service: Arc<AuthService>,
    pub agent_registry: Arc<AgentRegistry>,
    pub tx_processor: Arc<TransactionProcessor>,
    pub wallet_service: Arc<WalletService>,
    pub rate_limiter: Arc<RateLimiter>,
    pub config: AppConfig,
}

// -------------------------------------------------------------------------
// ApiServer
// -------------------------------------------------------------------------

pub struct ApiServer {
    config: AppConfig,
}

impl ApiServer {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Start the REST API server. Blocks until shutdown.
    pub async fn start(state: Arc<AppStateAxum>) -> Result<(), AppError> {
        let addr = format!("{}:{}", state.config.rest_host, state.config.rest_port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

        tracing::info!("REST API listening on {}", addr);

        axum::serve(listener, Self::router(state))
            .await
            .map_err(|e| AppError::Internal(format!("Server error: {}", e)))
    }

    /// Build the router. Exposed publicly for integration testing.
    pub fn router(state: Arc<AppStateAxum>) -> Router {
        let auth_routes = Router::new()
            .route("/v1/send", post(rest_routes::send_transaction))
            .route("/v1/balance", get(rest_routes::get_balance))
            .route("/v1/transactions", get(rest_routes::list_transactions))
            .route(
                "/v1/transactions/{tx_id}",
                get(rest_routes::get_transaction_handler),
            )
            .route(
                "/v1/limits/request-increase",
                post(rest_routes::request_limit_increase),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        let public_routes = Router::new()
            .route("/v1/health", get(rest_routes::health))
            .route("/v1/agents/register", post(rest_routes::register_agent))
            .route(
                "/v1/agents/register/{id}/status",
                get(rest_routes::registration_status),
            );

        Router::new()
            .merge(auth_routes)
            .merge(public_routes)
            .with_state(state)
    }
}
