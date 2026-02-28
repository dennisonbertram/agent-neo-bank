//! Streamable HTTP MCP server (protocol version 2025-11-25).
//!
//! Implements the MCP Streamable HTTP transport as a single `/mcp` endpoint
//! supporting POST (JSON-RPC), GET (SSE — currently 405), and DELETE (session
//! termination). Uses `McpRouter` for transport-agnostic tool dispatch.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::Router;
use dashmap::DashMap;
use serde_json::Value;

use crate::api::mcp_router::{error_code, McpRouter};
use crate::api::mcp_server::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::api::mcp_tools::get_tool_definitions;
use crate::db::queries;
use crate::db::schema::Database;
use crate::state::app_state::AppState;

// -------------------------------------------------------------------------
// Constants
// -------------------------------------------------------------------------

const PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "tally-agentic-wallet-mcp";
const SERVER_VERSION: &str = "0.1.0";

// -------------------------------------------------------------------------
// Session types
// -------------------------------------------------------------------------

struct McpSession {
    agent_id: Option<String>,
    #[allow(dead_code)]
    created_at: std::time::Instant,
    protocol_version: String,
}

// -------------------------------------------------------------------------
// Shared server state (passed into Axum handlers via State)
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct McpHttpState {
    pub db: Arc<Database>,
    pub cli: Option<Arc<dyn crate::cli::executor::CliExecutable>>,
    sessions: Arc<DashMap<String, McpSession>>,
}

impl McpHttpState {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            cli: None,
            sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn new_with_cli(db: Arc<Database>, cli: Arc<dyn crate::cli::executor::CliExecutable>) -> Self {
        Self {
            db,
            cli: Some(cli),
            sessions: Arc::new(DashMap::new()),
        }
    }
}

// -------------------------------------------------------------------------
// Public API
// -------------------------------------------------------------------------

/// Build the Axum `Router` for the MCP HTTP server.
///
/// Callers can use this to either run a real server (`axum::serve`) or
/// build a test client (`axum::Router::into_service`).
pub fn build_router(state: McpHttpState) -> Router {
    Router::new()
        .route("/mcp", post(handle_post))
        .route("/mcp", get(handle_get))
        .route("/mcp", delete(handle_delete))
        .with_state(state)
}

/// Start the MCP HTTP server, listening on `127.0.0.1:{port}`.
pub async fn start(app_state: Arc<AppState>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = McpHttpState::new_with_cli(app_state.db.clone(), app_state.cli.clone());

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    tracing::info!("MCP HTTP server listening on 127.0.0.1:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

// -------------------------------------------------------------------------
// Origin validation
// -------------------------------------------------------------------------

fn validate_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
    if let Some(origin) = headers.get("origin") {
        let origin_str = origin.to_str().unwrap_or("");
        let allowed = origin_str.starts_with("http://localhost")
            || origin_str.starts_with("http://127.0.0.1")
            || origin_str.starts_with("https://localhost")
            || origin_str.starts_with("https://127.0.0.1");
        if !allowed {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    // No Origin header (e.g. curl, CLI tools) is allowed
    Ok(())
}

// -------------------------------------------------------------------------
// POST /mcp handler
// -------------------------------------------------------------------------

async fn handle_post(
    State(state): State<McpHttpState>,
    headers: HeaderMap,
    body: String,
) -> Response {
    // 1. Validate Origin
    if let Err(status) = validate_origin(&headers) {
        return (status, "Forbidden: invalid origin").into_response();
    }

    // 2. Validate Accept header
    let accept = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !accept.contains("application/json") {
        return (
            StatusCode::BAD_REQUEST,
            "Missing or invalid Accept header — must include application/json",
        )
            .into_response();
    }

    // 3. Parse JSON-RPC
    let request: JsonRpcRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(e) => {
            let err_resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                }),
            };
            return (StatusCode::OK, axum::Json(err_resp)).into_response();
        }
    };

    // 4. Handle notifications (no id field) -> 202 Accepted
    if request.id.is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    // 5. Route by method
    match request.method.as_str() {
        "initialize" => handle_initialize(&state, &request, &headers).into_response(),
        method => {
            // All methods after initialize require MCP-Session-Id
            let session_id = match headers.get("mcp-session-id").and_then(|v| v.to_str().ok()) {
                Some(id) => id.to_string(),
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "Missing MCP-Session-Id header",
                    )
                        .into_response();
                }
            };

            // Verify session exists
            if !state.sessions.contains_key(&session_id) {
                return (
                    StatusCode::NOT_FOUND,
                    "Session not found or expired — re-initialize",
                )
                    .into_response();
            }

            match method {
                "tools/list" => handle_tools_list(&state, &request),
                "tools/call" => handle_tools_call(&state, &request, &headers, &session_id),
                _ => {
                    let err_resp = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id.clone(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: format!("Method not found: {}", method),
                        }),
                    };
                    (StatusCode::OK, axum::Json(err_resp)).into_response()
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Method handlers
// -------------------------------------------------------------------------

fn handle_initialize(
    state: &McpHttpState,
    request: &JsonRpcRequest,
    _headers: &HeaderMap,
) -> Response {
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = McpSession {
        agent_id: None,
        created_at: std::time::Instant::now(),
        protocol_version: PROTOCOL_VERSION.to_string(),
    };
    state.sessions.insert(session_id.clone(), session);

    let result = serde_json::json!({
        "protocolVersion": PROTOCOL_VERSION,
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION,
        },
        "capabilities": {
            "tools": {}
        }
    });

    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(result),
        error: None,
    };

    let mut response = (StatusCode::OK, axum::Json(resp)).into_response();
    response.headers_mut().insert(
        "mcp-session-id",
        HeaderValue::from_str(&session_id).unwrap(),
    );
    response
}

fn handle_tools_list(_state: &McpHttpState, request: &JsonRpcRequest) -> Response {
    let tools = get_tool_definitions();
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id.clone(),
        result: Some(serde_json::json!({ "tools": tools })),
        error: None,
    };
    (StatusCode::OK, axum::Json(resp)).into_response()
}

fn handle_tools_call(
    state: &McpHttpState,
    request: &JsonRpcRequest,
    headers: &HeaderMap,
    session_id: &str,
) -> Response {
    let params = request
        .params
        .as_ref()
        .unwrap_or(&serde_json::Value::Null);
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // register_agent does NOT require auth
    if tool_name == "register_agent" {
        return handle_register_agent_call(state, request, &arguments, session_id);
    }

    // All other tools require Bearer token
    let bearer_token = match extract_bearer_token(headers) {
        Some(t) => t,
        None => {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: "Authentication required — provide Authorization: Bearer <token>"
                        .to_string(),
                }),
            };
            return (StatusCode::OK, axum::Json(resp)).into_response();
        }
    };

    // Validate token -> agent_id
    let agent_id = match validate_bearer_token(&state.db, &bearer_token) {
        Some(id) => id,
        None => {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: "Invalid token".to_string(),
                }),
            };
            return (StatusCode::OK, axum::Json(resp)).into_response();
        }
    };

    // Dispatch via McpRouter
    let router = if let Some(ref cli) = state.cli {
        McpRouter::new_with_cli(state.db.clone(), agent_id, cli.clone())
    } else {
        McpRouter::new(state.db.clone(), agent_id)
    };
    match router.handle_tool_call(tool_name, arguments) {
        Ok(content) => {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": content.to_string() }]
                })),
                error: None,
            };
            (StatusCode::OK, axum::Json(resp)).into_response()
        }
        Err(e) => {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: error_code(&e),
                    message: e.to_string(),
                }),
            };
            (StatusCode::OK, axum::Json(resp)).into_response()
        }
    }
}

fn handle_register_agent_call(
    state: &McpHttpState,
    request: &JsonRpcRequest,
    arguments: &Value,
    session_id: &str,
) -> Response {
    // Use a temporary router (agent_id doesn't matter for register_agent since
    // it creates a new agent)
    let router = if let Some(ref cli) = state.cli {
        McpRouter::new_with_cli(state.db.clone(), "registering".to_string(), cli.clone())
    } else {
        McpRouter::new(state.db.clone(), "registering".to_string())
    };
    match router.handle_tool_call("register_agent", arguments.clone()) {
        Ok(content) => {
            // Update session with the new agent_id if returned
            if let Some(agent_id) = content.get("agent_id").and_then(|v| v.as_str()) {
                if let Some(mut session) = state.sessions.get_mut(session_id) {
                    session.agent_id = Some(agent_id.to_string());
                }
            }
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(serde_json::json!({
                    "content": [{ "type": "text", "text": content.to_string() }]
                })),
                error: None,
            };
            (StatusCode::OK, axum::Json(resp)).into_response()
        }
        Err(e) => {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: error_code(&e),
                    message: e.to_string(),
                }),
            };
            (StatusCode::OK, axum::Json(resp)).into_response()
        }
    }
}

// -------------------------------------------------------------------------
// GET /mcp handler (SSE — not yet implemented)
// -------------------------------------------------------------------------

async fn handle_get(
    State(_state): State<McpHttpState>,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = validate_origin(&headers) {
        return (status, "Forbidden: invalid origin").into_response();
    }

    // SSE not implemented yet — return 405
    StatusCode::METHOD_NOT_ALLOWED.into_response()
}

// -------------------------------------------------------------------------
// DELETE /mcp handler
// -------------------------------------------------------------------------

async fn handle_delete(
    State(state): State<McpHttpState>,
    headers: HeaderMap,
) -> Response {
    if let Err(status) = validate_origin(&headers) {
        return (status, "Forbidden: invalid origin").into_response();
    }

    let session_id = match headers.get("mcp-session-id").and_then(|v| v.to_str().ok()) {
        Some(id) => id.to_string(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing MCP-Session-Id header").into_response();
        }
    };

    if state.sessions.remove(&session_id).is_some() {
        StatusCode::OK.into_response()
    } else {
        (StatusCode::NOT_FOUND, "Session not found").into_response()
    }
}

// -------------------------------------------------------------------------
// Auth helpers
// -------------------------------------------------------------------------

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

fn validate_bearer_token(db: &Database, token: &str) -> Option<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    queries::get_agent_by_token_hash(db, &token_hash).map(|agent| agent.id)
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    use crate::db::models::*;
    use crate::db::queries::{insert_agent, insert_spending_policy};
    use crate::test_helpers::{
        create_test_agent, create_test_invitation, create_test_spending_policy, setup_test_db,
    };

    fn test_state() -> McpHttpState {
        let db = setup_test_db();
        McpHttpState::new(db)
    }

    fn test_state_with_token_agent() -> (McpHttpState, String, String) {
        let state = test_state();
        let raw_token = "test_bearer_token_abc123";
        // Hash the token with SHA-256 (same as validate_bearer_token)
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        let mut agent = create_test_agent("AuthBot", AgentStatus::Active);
        let agent_id = agent.id.clone();
        agent.api_token_hash = Some(token_hash);
        insert_agent(&state.db, &agent).unwrap();
        let policy =
            create_test_spending_policy(&agent_id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&state.db, &policy).unwrap();
        (state, agent_id, raw_token.to_string())
    }

    /// Send a POST request and return the response.
    async fn post_mcp(
        app: &Router,
        body: &str,
        extra_headers: Vec<(&str, &str)>,
    ) -> http::Response<Body> {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream");

        for (k, v) in extra_headers {
            builder = builder.header(k, v);
        }

        let req = builder.body(Body::from(body.to_string())).unwrap();
        app.clone().oneshot(req).await.unwrap()
    }

    async fn body_json(resp: http::Response<Body>) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// Helper: send initialize and return (session_id, response_json).
    async fn do_initialize(app: &Router) -> (String, Value) {
        let resp = post_mcp(
            app,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{}}}"#,
            vec![],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let session_id = resp
            .headers()
            .get("mcp-session-id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let json = body_json(resp).await;
        (session_id, json)
    }

    // ------------------------------------------------------------------
    // 1. POST initialize -> 200 + MCP-Session-Id header
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_initialize_returns_session_id() {
        let state = test_state();
        let app = build_router(state);

        let (session_id, json) = do_initialize(&app).await;

        assert!(!session_id.is_empty(), "MCP-Session-Id should be set");
        assert_eq!(json["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(json["result"]["serverInfo"]["name"], SERVER_NAME);
        assert!(json["result"]["capabilities"]["tools"].is_object());
        assert!(json.get("error").is_none());
    }

    // ------------------------------------------------------------------
    // 2. POST notification -> 202 Accepted
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_notification_returns_202() {
        let state = test_state();
        let app = build_router(state);

        // Notification: no "id" field
        let resp = post_mcp(
            &app,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            vec![],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    // ------------------------------------------------------------------
    // 3. POST without Accept header -> 400
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_without_accept_header_returns_400() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", "application/json")
            // No Accept header
            .body(Body::from(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ------------------------------------------------------------------
    // 4. POST with invalid JSON -> JSON-RPC parse error (-32700)
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_invalid_json_returns_parse_error() {
        let state = test_state();
        let app = build_router(state);

        let resp = post_mcp(&app, "not valid json{{{", vec![]).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["error"]["code"], -32700);
        assert!(json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Parse error"));
    }

    // ------------------------------------------------------------------
    // 5. POST tools/list -> returns tools array
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_tools_list_returns_tools() {
        let state = test_state();
        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        let resp = post_mcp(
            &app,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            vec![("mcp-session-id", &session_id)],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        let tools = json["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 13);
    }

    // ------------------------------------------------------------------
    // 6. POST tools/call register_agent without auth -> succeeds
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_register_agent_without_auth_succeeds() {
        let state = test_state();
        // Create an invitation code for registration
        let invitation = create_test_invitation("INV-HTTP-001", "HTTP test");
        crate::db::queries::insert_invitation_code(&state.db, &invitation).unwrap();

        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "register_agent",
                "arguments": {
                    "name": "NewHTTPAgent",
                    "purpose": "Testing HTTP transport",
                    "invitation_code": "INV-HTTP-001"
                }
            }
        });

        let resp = post_mcp(
            &app,
            &body.to_string(),
            vec![("mcp-session-id", &session_id)],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert!(json.get("error").is_none(), "register_agent should not error without auth");
        assert!(json["result"]["content"].is_array());
    }

    // ------------------------------------------------------------------
    // 7. POST tools/call send_payment without auth -> error
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_send_payment_without_auth_returns_error() {
        let state = test_state();
        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "send_payment",
                "arguments": {
                    "to": "0x1234",
                    "amount": "10.00",
                    "asset": "USDC"
                }
            }
        });

        let resp = post_mcp(
            &app,
            &body.to_string(),
            vec![("mcp-session-id", &session_id)],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["error"]["code"], -32000);
        assert!(json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Authentication required"));
    }

    // ------------------------------------------------------------------
    // 8. POST tools/call send_payment with valid Bearer -> succeeds
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_send_payment_with_valid_bearer_succeeds() {
        let (state, _agent_id, token) = test_state_with_token_agent();
        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "send_payment",
                "arguments": {
                    "to": "0xRecipient123",
                    "amount": "25.00",
                    "asset": "USDC"
                }
            }
        });

        let auth_header = format!("Bearer {}", token);
        let resp = post_mcp(
            &app,
            &body.to_string(),
            vec![
                ("mcp-session-id", &session_id),
                ("authorization", &auth_header),
            ],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert!(
            json.get("error").is_none(),
            "Authenticated send_payment should succeed, got: {:?}",
            json
        );
        assert!(json["result"]["content"].is_array());
    }

    // ------------------------------------------------------------------
    // 9. POST without MCP-Session-Id after init -> 400
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_post_without_session_id_after_init_returns_400() {
        let state = test_state();
        let app = build_router(state);

        // First initialize to prove sessions work
        let _ = do_initialize(&app).await;

        // Now call tools/list WITHOUT the session ID
        let resp = post_mcp(
            &app,
            r#"{"jsonrpc":"2.0","id":10,"method":"tools/list"}"#,
            vec![], // no mcp-session-id
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ------------------------------------------------------------------
    // 10. DELETE /mcp -> terminates session
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_delete_terminates_session() {
        let state = test_state();
        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        // DELETE session
        let req = Request::builder()
            .method("DELETE")
            .uri("/mcp")
            .header("mcp-session-id", &session_id)
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Subsequent request with same session_id should fail with 404
        let resp = post_mcp(
            &app,
            r#"{"jsonrpc":"2.0","id":11,"method":"tools/list"}"#,
            vec![("mcp-session-id", &session_id)],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ------------------------------------------------------------------
    // 11. Origin header validation
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_origin_validation_rejects_bad_origin() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .header("origin", "https://evil.com")
            .body(Body::from(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_origin_validation_allows_localhost() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .header("origin", "http://localhost:3000")
            .body(Body::from(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_origin_validation_allows_no_origin() {
        let state = test_state();
        let app = build_router(state);

        // No origin header (CLI tools, curl)
        let (_, json) = do_initialize(&app).await;
        assert!(json.get("error").is_none());
    }

    // ------------------------------------------------------------------
    // 12. DELETE with unknown session -> 404
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_delete_unknown_session_returns_404() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("DELETE")
            .uri("/mcp")
            .header("mcp-session-id", "nonexistent-session-id")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ------------------------------------------------------------------
    // 13. DELETE without MCP-Session-Id -> 400
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_delete_without_session_id_returns_400() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("DELETE")
            .uri("/mcp")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ------------------------------------------------------------------
    // 14. GET /mcp -> 405 (SSE not implemented yet)
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_returns_405() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/mcp")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // ------------------------------------------------------------------
    // 15. Unknown method after init -> method not found (-32601)
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_unknown_method_returns_32601() {
        let state = test_state();
        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        let resp = post_mcp(
            &app,
            r#"{"jsonrpc":"2.0","id":99,"method":"unknown/method"}"#,
            vec![("mcp-session-id", &session_id)],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["error"]["code"], -32601);
    }

    // ------------------------------------------------------------------
    // 16. POST with invalid Bearer token -> error
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_invalid_bearer_token_returns_error() {
        let state = test_state();
        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "tools/call",
            "params": {
                "name": "check_balance",
                "arguments": {}
            }
        });

        let resp = post_mcp(
            &app,
            &body.to_string(),
            vec![
                ("mcp-session-id", &session_id),
                ("authorization", "Bearer invalid_token_xyz"),
            ],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["error"]["code"], -32000);
        assert!(json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid token"));
    }

    // ------------------------------------------------------------------
    // 17. Session uniqueness — each initialize gets a unique session ID
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_each_initialize_creates_unique_session() {
        let state = test_state();
        let app = build_router(state);

        let (id1, _) = do_initialize(&app).await;
        let (id2, _) = do_initialize(&app).await;
        assert_ne!(id1, id2, "Each initialize should create a unique session");
    }

    // ------------------------------------------------------------------
    // 18. POST with expired/deleted session -> 404
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_request_with_expired_session_returns_404() {
        let state = test_state();
        let app = build_router(state);
        let (session_id, _) = do_initialize(&app).await;

        // Delete the session
        let req = Request::builder()
            .method("DELETE")
            .uri("/mcp")
            .header("mcp-session-id", &session_id)
            .body(Body::empty())
            .unwrap();
        let _ = app.clone().oneshot(req).await.unwrap();

        // Try to use the deleted session
        let resp = post_mcp(
            &app,
            r#"{"jsonrpc":"2.0","id":30,"method":"tools/list"}"#,
            vec![("mcp-session-id", &session_id)],
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ------------------------------------------------------------------
    // 19. Origin validation on DELETE
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_delete_with_bad_origin_returns_403() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("DELETE")
            .uri("/mcp")
            .header("mcp-session-id", "some-session")
            .header("origin", "https://evil.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // ------------------------------------------------------------------
    // 20. Origin validation on GET
    // ------------------------------------------------------------------
    #[tokio::test]
    async fn test_get_with_bad_origin_returns_403() {
        let state = test_state();
        let app = build_router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/mcp")
            .header("origin", "https://attacker.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
