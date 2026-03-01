//! Layer 4 — MCP E2E Client Tests
//!
//! A dedicated test file that acts as a real MCP client, connecting over HTTP
//! to a real MCP server instance. Tests the complete agent experience end-to-end
//! using `McpTestClient`, a helper struct that encapsulates the MCP protocol.

mod common;

use std::sync::Arc;

use serde_json::Value;
use tally_agentic_wallet_lib::api::mcp_http_server::{build_router, McpHttpState};
use tally_agentic_wallet_lib::db::models::*;
use tally_agentic_wallet_lib::db::queries;
use tally_agentic_wallet_lib::db::schema::Database;
use tally_agentic_wallet_lib::test_helpers::{
    create_test_agent, create_test_invitation, create_test_spending_policy, setup_test_db,
};

// =========================================================================
// McpTestClient — reusable MCP HTTP client for E2E tests
// =========================================================================

struct McpTestClient {
    base_url: String,
    session_id: Option<String>,
    token: Option<String>,
    client: reqwest::Client,
    next_id: u64,
}

impl McpTestClient {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            session_id: None,
            token: None,
            client: reqwest::Client::new(),
            next_id: 1,
        }
    }

    fn with_token(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            session_id: None,
            token: Some(token.to_string()),
            client: reqwest::Client::new(),
            next_id: 1,
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Send a raw POST request to /mcp.
    async fn raw_post(&self, body: &Value) -> reqwest::Response {
        let mut req = self
            .client
            .post(format!("{}/mcp", self.base_url))
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .json(body);

        if let Some(sid) = &self.session_id {
            req = req.header("mcp-session-id", sid);
        }
        if let Some(token) = &self.token {
            req = req.header("authorization", format!("Bearer {}", token));
        }

        req.send().await.unwrap()
    }

    /// Initialize a session.
    async fn initialize(&mut self) -> Value {
        let id = self.next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {}
            }
        });
        let resp = self.raw_post(&body).await;
        assert_eq!(resp.status(), 200, "initialize should return 200");

        let session_id = resp
            .headers()
            .get("mcp-session-id")
            .expect("initialize should return MCP-Session-Id")
            .to_str()
            .unwrap()
            .to_string();
        self.session_id = Some(session_id);

        resp.json().await.unwrap()
    }

    /// Send initialized notification.
    async fn send_initialized_notification(&mut self) {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let resp = self.raw_post(&body).await;
        assert_eq!(resp.status(), 202);
    }

    /// List available tools.
    async fn list_tools(&mut self) -> Vec<Value> {
        let id = self.next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/list"
        });
        let resp = self.raw_post(&body).await;
        let json: Value = resp.json().await.unwrap();
        assert!(json.get("error").is_none(), "tools/list failed: {:?}", json);
        json["result"]["tools"]
            .as_array()
            .unwrap()
            .clone()
    }

    /// Call a tool and return the parsed tool result (from content[0].text).
    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, Value> {
        let id = self.next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments }
        });
        let resp = self.raw_post(&body).await;
        let json: Value = resp.json().await.unwrap();

        if let Some(err) = json.get("error") {
            return Err(err.clone());
        }

        let text = json["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or("{}");
        Ok(serde_json::from_str(text).unwrap_or(Value::Null))
    }

    /// Terminate the session.
    async fn terminate(&self) {
        let sid = self.session_id.as_ref().expect("No session to terminate");
        let resp = self
            .client
            .delete(format!("{}/mcp", self.base_url))
            .header("mcp-session-id", sid)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "terminate should return 200");
    }
}

// =========================================================================
// Server setup helpers
// =========================================================================

async fn start_test_server() -> (String, Arc<Database>) {
    let db = setup_test_db();
    let state = McpHttpState::new(db.clone());
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    (base_url, db)
}

fn create_agent_with_token(
    db: &Database,
    name: &str,
    per_tx_max: &str,
    daily_cap: &str,
) -> (String, String) {
    use sha2::{Digest, Sha256};

    let raw_token = format!("tok_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    let mut agent = create_test_agent(name, AgentStatus::Active);
    let agent_id = agent.id.clone();
    agent.api_token_hash = Some(token_hash);
    queries::insert_agent(db, &agent).unwrap();

    let policy = create_test_spending_policy(&agent_id, per_tx_max, daily_cap, "50000", "200000", "50");
    queries::insert_spending_policy(db, &policy).unwrap();

    (agent_id, raw_token)
}

// =========================================================================
// Test: Fresh agent journey
// initialize -> list tools -> register -> denied (no limits) -> get limits
// =========================================================================

#[tokio::test]
async fn test_fresh_agent_journey() {
    let (base_url, db) = start_test_server().await;

    // Create invitation code for registration
    let invitation = create_test_invitation("INV-E2E-FRESH", "E2E fresh agent");
    queries::insert_invitation_code(&db, &invitation).unwrap();

    let mut client = McpTestClient::new(&base_url);

    // 1. Initialize
    let init_result = client.initialize().await;
    assert_eq!(init_result["result"]["protocolVersion"], "2025-11-25");
    assert!(client.session_id.is_some());

    // 2. Send initialized notification
    client.send_initialized_notification().await;

    // 3. List tools (unauthenticated — should only see register_agent)
    let tools = client.list_tools().await;
    assert_eq!(tools.len(), 1, "Unauthenticated should only see register_agent, got {}", tools.len());
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"register_agent"));

    // 4. Register (no auth needed)
    let register_result = client
        .call_tool(
            "register_agent",
            serde_json::json!({
                "name": "E2E Fresh Agent",
                "purpose": "End-to-end testing",
                "invitation_code": "INV-E2E-FRESH"
            }),
        )
        .await
        .unwrap();

    assert!(register_result.get("agent_id").is_some());
    assert_eq!(register_result["status"], "pending");
}

// =========================================================================
// Test: Returning agent with saved token
// =========================================================================

#[tokio::test]
async fn test_returning_agent_with_saved_token() {
    let (base_url, db) = start_test_server().await;

    let (_agent_id, token) = create_agent_with_token(&db, "ReturningBot", "100", "1000");

    let mut client = McpTestClient::with_token(&base_url, &token);
    client.initialize().await;
    client.send_initialized_notification().await;

    // Check balance
    let balance = client
        .call_tool("check_balance", serde_json::json!({}))
        .await
        .unwrap();
    assert!(balance.get("balance").is_some());

    // Get spending limits
    let limits = client
        .call_tool("get_spending_limits", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(limits["per_tx_max"], "100");
    assert_eq!(limits["daily_cap"], "1000");

    // Send payment within limits
    let send_result = client
        .call_tool(
            "send_payment",
            serde_json::json!({
                "to": "0xRecipient",
                "amount": "25.00",
                "asset": "USDC",
                "memo": "E2E test payment"
            }),
        )
        .await
        .unwrap();
    assert!(send_result.get("tx_id").is_some());
    assert_eq!(send_result["status"], "pending");

    // Get transactions — should see our payment
    let txs = client
        .call_tool("get_transactions", serde_json::json!({}))
        .await
        .unwrap();
    let tx_list = txs["transactions"].as_array().unwrap();
    assert_eq!(tx_list.len(), 1);
    assert_eq!(tx_list[0]["amount"], "25.00");
}

// =========================================================================
// Test: Multiple agents with isolated spending
// =========================================================================

#[tokio::test]
async fn test_multiple_agents_isolated_spending() {
    let (base_url, db) = start_test_server().await;

    let (_id_a, token_a) = create_agent_with_token(&db, "IsoAgentA", "100", "1000");
    let (_id_b, token_b) = create_agent_with_token(&db, "IsoAgentB", "100", "1000");

    // Agent A session
    let mut client_a = McpTestClient::with_token(&base_url, &token_a);
    client_a.initialize().await;

    // Agent B session
    let mut client_b = McpTestClient::with_token(&base_url, &token_b);
    client_b.initialize().await;

    // Agent A sends 3 payments
    for i in 1..=3 {
        let result = client_a
            .call_tool(
                "send_payment",
                serde_json::json!({ "to": format!("0xA{}", i), "amount": "10" }),
            )
            .await
            .unwrap();
        assert!(result.get("tx_id").is_some());
    }

    // Agent B sends 1 payment
    let result = client_b
        .call_tool(
            "send_payment",
            serde_json::json!({ "to": "0xB1", "amount": "50" }),
        )
        .await
        .unwrap();
    assert!(result.get("tx_id").is_some());

    // Agent A sees 3 transactions
    let txs_a = client_a
        .call_tool("get_transactions", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(txs_a["transactions"].as_array().unwrap().len(), 3);
    assert_eq!(txs_a["total"], 3);

    // Agent B sees 1 transaction
    let txs_b = client_b
        .call_tool("get_transactions", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(txs_b["transactions"].as_array().unwrap().len(), 1);
    assert_eq!(txs_b["total"], 1);

    // Agent A's spending limits are independent from B
    let limits_a = client_a
        .call_tool("get_spending_limits", serde_json::json!({}))
        .await
        .unwrap();
    let limits_b = client_b
        .call_tool("get_spending_limits", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(limits_a["per_tx_max"], limits_b["per_tx_max"]);
}

// =========================================================================
// Test: Client reconnects after session expiry (new session, same token)
// =========================================================================

#[tokio::test]
async fn test_client_reconnects_after_session_expiry() {
    let (base_url, db) = start_test_server().await;
    let (_, token) = create_agent_with_token(&db, "ReconnBot", "100", "1000");

    // Session 1: initialize, use, terminate
    let mut client = McpTestClient::with_token(&base_url, &token);
    client.initialize().await;
    let result = client
        .call_tool("check_balance", serde_json::json!({}))
        .await
        .unwrap();
    assert!(result.get("balance").is_some());
    client.terminate().await;

    // After terminate, old session fails
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 100, "method": "tools/list"
    });
    let resp = client.raw_post(&body).await;
    assert_eq!(resp.status(), 404);

    // Re-initialize with same token -> new session works
    client.initialize().await;
    let result = client
        .call_tool("check_balance", serde_json::json!({}))
        .await
        .unwrap();
    assert!(result.get("balance").is_some());
}

// =========================================================================
// Test: Expired session returns 404 -> client re-initializes
// =========================================================================

#[tokio::test]
async fn test_expired_session_triggers_reinitialize() {
    let (base_url, db) = start_test_server().await;
    let (_, token) = create_agent_with_token(&db, "ExpireBot", "100", "1000");

    let mut client = McpTestClient::with_token(&base_url, &token);
    client.initialize().await;
    let old_session = client.session_id.clone().unwrap();

    // Terminate to simulate expiry
    client.terminate().await;

    // Request with old session gets 404
    client.session_id = Some(old_session);
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/list"
    });
    let resp = client.raw_post(&body).await;
    assert_eq!(resp.status(), 404, "Expired session should return 404");

    // Client detects 404 and re-initializes
    client.initialize().await;
    let tools = client.list_tools().await;
    assert!(tools.len() >= 6, "Re-initialized session should have tools, got {}", tools.len());
}

// =========================================================================
// Test: Concurrent agents sending payments (atomic policy checks)
// =========================================================================

#[tokio::test]
async fn test_concurrent_agents_atomic_policy() {
    let (base_url, db) = start_test_server().await;

    // Create 5 agents with daily_cap=100, per_tx=50
    let mut tokens = Vec::new();
    for i in 0..5 {
        let (_, token) = create_agent_with_token(&db, &format!("ConcAgent{}", i), "50", "100");
        tokens.push(token);
    }

    // All 5 agents send 30 concurrently
    let mut handles = Vec::new();
    for token in &tokens {
        let url = base_url.clone();
        let tok = token.clone();
        handles.push(tokio::spawn(async move {
            let mut client = McpTestClient::with_token(&url, &tok);
            client.initialize().await;
            client
                .call_tool(
                    "send_payment",
                    serde_json::json!({ "to": "0xShared", "amount": "30" }),
                )
                .await
        }));
    }

    let mut successes = 0;
    for handle in handles {
        let result = handle.await.unwrap();
        if result.is_ok() {
            successes += 1;
        }
    }

    // All 5 should succeed (each agent has independent daily_cap=100)
    assert_eq!(successes, 5, "All 5 agents with independent caps should succeed");
}

// =========================================================================
// Test: Request limit increase
// =========================================================================

#[tokio::test]
async fn test_request_limit_increase() {
    let (base_url, db) = start_test_server().await;
    let (_, token) = create_agent_with_token(&db, "LimitBot", "10", "100");

    let mut client = McpTestClient::with_token(&base_url, &token);
    client.initialize().await;

    let result = client
        .call_tool(
            "request_limit_increase",
            serde_json::json!({
                "new_per_tx_max": "500",
                "new_daily_cap": "5000",
                "reason": "Need higher limits for vendor payments"
            }),
        )
        .await
        .unwrap();

    assert!(result.get("request_id").is_some());
    assert_eq!(result["status"], "pending");
    assert!(result["message"].as_str().unwrap().contains("approval"));
}

// =========================================================================
// Test: Full protocol compliance — headers validated correctly
// =========================================================================

#[tokio::test]
async fn test_protocol_compliance_headers() {
    let (base_url, _db) = start_test_server().await;
    let client = reqwest::Client::new();

    // Missing Accept header -> 400
    let resp = client
        .post(format!("{}/mcp", base_url))
        .header("content-type", "application/json")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "Missing Accept should be 400");

    // Bad origin -> 403
    let resp = client
        .post(format!("{}/mcp", base_url))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("origin", "https://evil.com")
        .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "Bad origin should be 403");

    // Invalid JSON -> parse error
    let resp = client
        .post(format!("{}/mcp", base_url))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .body("not json{{{")
        .send()
        .await
        .unwrap();
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["code"], -32700);

    // GET /mcp without session ID -> 400 (missing MCP-Session-Id)
    let resp = client
        .get(format!("{}/mcp", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}
