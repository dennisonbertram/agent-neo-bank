//! Layer 3 — MCP HTTP Integration Tests
//!
//! Tests the full request path through the Axum HTTP server with a real SQLite
//! database. Covers session lifecycle, auth flow, policy enforcement, and
//! multi-agent isolation over real HTTP using reqwest.

mod common;

use std::sync::Arc;

use serde_json::Value;
use tally_agentic_wallet_lib::api::mcp_http_server::{build_router, McpHttpState};
use tally_agentic_wallet_lib::db::models::*;
use tally_agentic_wallet_lib::db::queries;
use tally_agentic_wallet_lib::test_helpers::{
    create_test_agent, create_test_invitation, create_test_spending_policy, setup_test_db,
};

// =========================================================================
// Test helpers
// =========================================================================

/// Start a real MCP HTTP server on port 0 (OS-assigned) and return
/// (base_url, db) so tests can interact over HTTP and inspect the DB.
async fn start_test_server() -> (String, Arc<tally_agentic_wallet_lib::db::schema::Database>) {
    let db = setup_test_db();
    let state = McpHttpState::new(db.clone());
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to bind
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    (base_url, db)
}

/// Create agent with SHA-256 token hash in the DB. Returns (agent_id, raw_token).
fn create_agent_with_token(
    db: &tally_agentic_wallet_lib::db::schema::Database,
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

/// Send a POST /mcp request.
async fn post_mcp(
    client: &reqwest::Client,
    base_url: &str,
    body: &Value,
    session_id: Option<&str>,
    bearer: Option<&str>,
) -> reqwest::Response {
    let mut req = client
        .post(format!("{}/mcp", base_url))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("origin", "http://localhost:1420")
        .json(body);

    if let Some(sid) = session_id {
        req = req.header("mcp-session-id", sid);
    }
    if let Some(token) = bearer {
        req = req.header("authorization", format!("Bearer {}", token));
    }

    req.send().await.unwrap()
}

/// Initialize a session and return the MCP-Session-Id.
async fn initialize(client: &reqwest::Client, base_url: &str) -> String {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {}
        }
    });
    let resp = post_mcp(client, base_url, &body, None, None).await;
    assert_eq!(resp.status(), 200);
    let session_id = resp
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    session_id
}

// =========================================================================
// Full session lifecycle:
// initialize -> notification -> tools/list -> tools/call -> delete
// =========================================================================

#[tokio::test]
async fn test_full_session_lifecycle() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();

    // Create an invitation code for register_agent
    let invitation = create_test_invitation("INV-HTTP-LIFE-001", "lifecycle test");
    queries::insert_invitation_code(&db, &invitation).unwrap();

    // 1. Initialize
    let session_id = initialize(&client, &base_url).await;
    assert!(!session_id.is_empty());

    // 2. Send initialized notification (no id -> 202)
    let notif = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let resp = post_mcp(&client, &base_url, &notif, Some(&session_id), None).await;
    assert_eq!(resp.status(), 202);

    // 3. tools/list
    let list_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    let resp = post_mcp(&client, &base_url, &list_body, Some(&session_id), None).await;
    assert_eq!(resp.status(), 200);
    let json: Value = resp.json().await.unwrap();
    let tools = json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1, "Unauthenticated should only see register_agent, got {}", tools.len());
    assert_eq!(tools[0]["name"], "register_agent");

    // 4. tools/call register_agent (no auth needed)
    let register_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "register_agent",
            "arguments": {
                "name": "IntegrationAgent",
                "purpose": "Integration testing",
                "invitation_code": "INV-HTTP-LIFE-001"
            }
        }
    });
    let resp = post_mcp(&client, &base_url, &register_body, Some(&session_id), None).await;
    assert_eq!(resp.status(), 200);
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none(), "register_agent failed: {:?}", json);

    // 5. DELETE session
    let resp = client
        .delete(format!("{}/mcp", base_url))
        .header("mcp-session-id", &session_id)
        .header("origin", "http://localhost:1420")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 6. Verify session is gone
    let resp = post_mcp(&client, &base_url, &list_body, Some(&session_id), None).await;
    assert_eq!(resp.status(), 404);
}

// =========================================================================
// Auth flow: unauth can only call register_agent + tools/list + initialize
// =========================================================================

#[tokio::test]
async fn test_unauth_session_restricted_to_public_methods() {
    let (base_url, _db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    // tools/list works without auth
    let list_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/list"
    });
    let resp = post_mcp(&client, &base_url, &list_body, Some(&session_id), None).await;
    assert_eq!(resp.status(), 200);
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none());

    // send_payment without auth -> auth error
    let send_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": { "name": "send_payment", "arguments": { "to": "0xdddddddddddddddddddddddddddddddddddddddd", "amount": "1" } }
    });
    let resp = post_mcp(&client, &base_url, &send_body, Some(&session_id), None).await;
    assert_eq!(resp.status(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["code"], -32000);

    // check_balance without auth -> auth error
    let balance_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": { "name": "check_balance", "arguments": {} }
    });
    let resp = post_mcp(&client, &base_url, &balance_body, Some(&session_id), None).await;
    assert_eq!(resp.status(), 200);
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["code"], -32000);
}

// =========================================================================
// After register, token works for all other tools
// =========================================================================

#[tokio::test]
async fn test_valid_token_grants_access_to_all_tools() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    let (agent_id, token) = create_agent_with_token(&db, "AuthTestBot", "100", "1000");

    // check_balance with valid token
    let balance_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "check_balance", "arguments": {} }
    });
    let resp = post_mcp(&client, &base_url, &balance_body, Some(&session_id), Some(&token)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none(), "check_balance with token should work: {:?}", json);

    // get_spending_limits with valid token
    let limits_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": { "name": "get_spending_limits", "arguments": {} }
    });
    let resp = post_mcp(&client, &base_url, &limits_body, Some(&session_id), Some(&token)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none(), "get_spending_limits with token should work: {:?}", json);

    // send_payment with valid token
    let send_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": { "name": "send_payment", "arguments": { "to": "0xdddddddddddddddddddddddddddddddddddddddd", "amount": "10", "asset": "USDC" } }
    });
    let resp = post_mcp(&client, &base_url, &send_body, Some(&session_id), Some(&token)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none(), "send_payment with token should work: {:?}", json);

    // Verify transaction persisted
    let (txs, _) = queries::list_transactions_paginated(&db, Some(&agent_id), None, 10, 0).unwrap();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].amount, "10");
}

// =========================================================================
// Token from Agent A cannot access Agent B's data
// =========================================================================

#[tokio::test]
async fn test_token_isolation_between_agents() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    let (_agent_a_id, token_a) = create_agent_with_token(&db, "AgentA", "100", "1000");
    let (_agent_b_id, token_b) = create_agent_with_token(&db, "AgentB", "100", "1000");

    // Agent A sends a payment
    let send_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "send_payment", "arguments": { "to": "0xdddddddddddddddddddddddddddddddddddddddd", "amount": "5" } }
    });
    let resp = post_mcp(&client, &base_url, &send_body, Some(&session_id), Some(&token_a)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none());

    // Agent A sees its transaction
    let txs_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": { "name": "get_transactions", "arguments": {} }
    });
    let resp = post_mcp(&client, &base_url, &txs_body, Some(&session_id), Some(&token_a)).await;
    let json: Value = resp.json().await.unwrap();
    let content_text = json["result"]["content"][0]["text"].as_str().unwrap();
    let tool_result: Value = serde_json::from_str(content_text).unwrap();
    assert_eq!(tool_result["transactions"].as_array().unwrap().len(), 1);

    // Agent B does NOT see Agent A's transaction
    let resp = post_mcp(&client, &base_url, &txs_body, Some(&session_id), Some(&token_b)).await;
    let json: Value = resp.json().await.unwrap();
    let content_text = json["result"]["content"][0]["text"].as_str().unwrap();
    let tool_result: Value = serde_json::from_str(content_text).unwrap();
    assert_eq!(
        tool_result["transactions"].as_array().unwrap().len(),
        0,
        "Agent B should not see Agent A's transactions"
    );
}

// =========================================================================
// Policy enforcement end-to-end: per_tx_max blocks overspend
// =========================================================================

#[tokio::test]
async fn test_policy_enforcement_per_tx_max() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    // Agent with per_tx_max = 10
    let (_agent_id, token) = create_agent_with_token(&db, "PolicyBot", "10", "1000");

    // Send 15 -> exceeds per_tx_max -> denied
    let send_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "send_payment", "arguments": { "to": "0xdddddddddddddddddddddddddddddddddddddddd", "amount": "15" } }
    });
    let resp = post_mcp(&client, &base_url, &send_body, Some(&session_id), Some(&token)).await;
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["code"], -32001, "Should be PolicyViolation");

    // Send 5 -> within limits -> succeeds
    let send_body2 = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": { "name": "send_payment", "arguments": { "to": "0xdddddddddddddddddddddddddddddddddddddddd", "amount": "5" } }
    });
    let resp = post_mcp(&client, &base_url, &send_body2, Some(&session_id), Some(&token)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none(), "5 under per_tx_max 10 should succeed: {:?}", json);
}

// =========================================================================
// Daily cap accumulates across multiple transactions
// =========================================================================

#[tokio::test]
async fn test_daily_cap_accumulates() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    // Agent with daily_cap = 20, per_tx_max = 100
    let (_agent_id, token) = create_agent_with_token(&db, "DailyCapBot", "100", "20");

    // Send 12 -> ok (12 < 20)
    let send = |id: u64, amount: &str| {
        serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": { "name": "send_payment", "arguments": { "to": "0xdddddddddddddddddddddddddddddddddddddddd", "amount": amount } }
        })
    };

    let resp = post_mcp(&client, &base_url, &send(1, "12"), Some(&session_id), Some(&token)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none(), "12 should succeed under daily cap 20");

    // Send 12 again -> total 24 > 20 -> denied
    let resp = post_mcp(&client, &base_url, &send(2, "12"), Some(&session_id), Some(&token)).await;
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["code"], -32001, "24 > daily cap 20 should be denied");
}

// =========================================================================
// Kill switch blocks all agents
// =========================================================================

#[tokio::test]
async fn test_kill_switch_blocks_all_agents() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    let (_id_a, token_a) = create_agent_with_token(&db, "KSA", "100", "1000");
    let (_id_b, token_b) = create_agent_with_token(&db, "KSB", "100", "1000");

    // Activate kill switch
    let global_policy = GlobalPolicy {
        id: "default".to_string(),
        daily_cap: "0".to_string(),
        weekly_cap: "0".to_string(),
        monthly_cap: "0".to_string(),
        min_reserve_balance: "0".to_string(),
        kill_switch_active: true,
        kill_switch_reason: "Emergency".to_string(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    queries::upsert_global_policy(&db, &global_policy).unwrap();

    let send = |id: u64| serde_json::json!({
        "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": { "name": "send_payment", "arguments": { "to": "0xdddddddddddddddddddddddddddddddddddddddd", "amount": "1" } }
    });

    // Agent A denied
    let resp = post_mcp(&client, &base_url, &send(1), Some(&session_id), Some(&token_a)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json["error"].is_object(), "Agent A should be denied by kill switch");

    // Agent B denied
    let resp = post_mcp(&client, &base_url, &send(2), Some(&session_id), Some(&token_b)).await;
    let json: Value = resp.json().await.unwrap();
    assert!(json["error"].is_object(), "Agent B should be denied by kill switch");
}

// =========================================================================
// Session survives multiple requests
// =========================================================================

#[tokio::test]
async fn test_session_survives_multiple_requests() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    let (_, token) = create_agent_with_token(&db, "PersistBot", "100", "1000");

    // Make several requests on the same session
    for i in 1..=5 {
        let body = serde_json::json!({
            "jsonrpc": "2.0", "id": i, "method": "tools/call",
            "params": { "name": "check_balance", "arguments": {} }
        });
        let resp = post_mcp(&client, &base_url, &body, Some(&session_id), Some(&token)).await;
        assert_eq!(resp.status(), 200);
        let json: Value = resp.json().await.unwrap();
        assert!(json.get("error").is_none(), "Request {} should succeed", i);
    }
}

// =========================================================================
// Re-initialize after session deletion
// =========================================================================

#[tokio::test]
async fn test_re_initialize_after_session_deletion() {
    let (base_url, _db) = start_test_server().await;
    let client = reqwest::Client::new();

    // Create and destroy a session
    let session1 = initialize(&client, &base_url).await;
    let resp = client
        .delete(format!("{}/mcp", base_url))
        .header("mcp-session-id", &session1)
        .header("origin", "http://localhost:1420")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Re-initialize -> new session
    let session2 = initialize(&client, &base_url).await;
    assert_ne!(session1, session2);

    // New session works
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/list"
    });
    let resp = post_mcp(&client, &base_url, &body, Some(&session2), None).await;
    assert_eq!(resp.status(), 200);
    let json: Value = resp.json().await.unwrap();
    assert!(json.get("error").is_none());
}

// =========================================================================
// Invalid token returns auth error
// =========================================================================

#[tokio::test]
async fn test_invalid_token_returns_auth_error() {
    let (base_url, _db) = start_test_server().await;
    let client = reqwest::Client::new();
    let session_id = initialize(&client, &base_url).await;

    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "check_balance", "arguments": {} }
    });
    let resp = post_mcp(&client, &base_url, &body, Some(&session_id), Some("bad_token_xyz")).await;
    let json: Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["code"], -32000);
}

// =========================================================================
// Multiple concurrent sessions
// =========================================================================

#[tokio::test]
async fn test_multiple_concurrent_sessions() {
    let (base_url, db) = start_test_server().await;
    let client = reqwest::Client::new();

    let (_, token) = create_agent_with_token(&db, "ConcBot", "100", "1000");

    // Create 3 sessions
    let s1 = initialize(&client, &base_url).await;
    let s2 = initialize(&client, &base_url).await;
    let s3 = initialize(&client, &base_url).await;

    // All sessions work independently
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "check_balance", "arguments": {} }
    });

    for sid in [&s1, &s2, &s3] {
        let resp = post_mcp(&client, &base_url, &body, Some(sid), Some(&token)).await;
        let json: Value = resp.json().await.unwrap();
        assert!(json.get("error").is_none(), "Session {} should work", sid);
    }

    // Delete session 2 — others still work
    client
        .delete(format!("{}/mcp", base_url))
        .header("mcp-session-id", &s2)
        .header("origin", "http://localhost:1420")
        .send()
        .await
        .unwrap();

    let resp = post_mcp(&client, &base_url, &body, Some(&s1), Some(&token)).await;
    assert_eq!(resp.status(), 200);

    let resp = post_mcp(&client, &base_url, &body, Some(&s2), Some(&token)).await;
    assert_eq!(resp.status(), 404, "Deleted session should return 404");

    let resp = post_mcp(&client, &base_url, &body, Some(&s3), Some(&token)).await;
    assert_eq!(resp.status(), 200);
}
