//! MCP End-to-End Tests
//!
//! Comprehensive tests for the MCP server lifecycle: JSON-RPC request handling,
//! tool invocation, token validation, tool listing, and spending policy enforcement.

mod common;

use std::sync::Arc;

use tally_agentic_wallet_lib::api::mcp_server::{JsonRpcRequest, McpServer};
use tally_agentic_wallet_lib::core::spending_policy::{daily_period_key, weekly_period_key, monthly_period_key};
use tally_agentic_wallet_lib::db::models::{AgentStatus, GlobalPolicy};
use tally_agentic_wallet_lib::db::queries;
use tally_agentic_wallet_lib::db::schema::Database;
use tally_agentic_wallet_lib::test_helpers::{
    create_test_agent, create_test_spending_policy, setup_test_db,
};

// =========================================================================
// Helpers
// =========================================================================

/// Create an active agent with a spending policy and return (db, agent_id).
fn setup_agent_and_db(
    per_tx_max: &str,
    daily_cap: &str,
    weekly_cap: &str,
    monthly_cap: &str,
    auto_approve_max: &str,
) -> (Arc<Database>, String) {
    let db = setup_test_db();
    let agent = create_test_agent("McpE2EAgent", AgentStatus::Active);
    let agent_id = agent.id.clone();
    queries::insert_agent(&db, &agent).unwrap();
    let policy = create_test_spending_policy(
        &agent_id,
        per_tx_max,
        daily_cap,
        weekly_cap,
        monthly_cap,
        auto_approve_max,
    );
    queries::insert_spending_policy(&db, &policy).unwrap();
    (db, agent_id)
}

/// Create an active agent with a SHA-256 token hash stored in the DB.
/// Returns (db, agent_id, raw_token).
fn setup_agent_with_sha256_token(
    name: &str,
) -> (Arc<Database>, String, String) {
    use sha2::{Digest, Sha256};

    let db = setup_test_db();
    let raw_token = format!("anb_test_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    let mut agent = create_test_agent(name, AgentStatus::Active);
    agent.api_token_hash = Some(token_hash);
    agent.token_prefix = Some(raw_token[..12].to_string());
    let agent_id = agent.id.clone();
    queries::insert_agent(&db, &agent).unwrap();

    let policy = create_test_spending_policy(&agent_id, "100", "1000", "5000", "20000", "50");
    queries::insert_spending_policy(&db, &policy).unwrap();

    (db, agent_id, raw_token)
}

/// Build a JSON-RPC tools/call request.
fn make_tools_call_request(id: u64, tool_name: &str, arguments: serde_json::Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(id)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        })),
    }
}

/// Extract the text content from a successful JSON-RPC tools/call response,
/// parsed as a JSON value.
fn extract_tool_result(response: &tally_agentic_wallet_lib::api::mcp_server::JsonRpcResponse) -> serde_json::Value {
    assert!(
        response.error.is_none(),
        "Expected success but got error: {:?}",
        response.error
    );
    let result = response.result.as_ref().unwrap();
    let content = result["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    serde_json::from_str(text).unwrap()
}

// =========================================================================
// test_mcp_send_payment_e2e
// =========================================================================

#[test]
fn test_mcp_send_payment_e2e() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    // Build JSON-RPC request for send_payment
    let request = make_tools_call_request(1, "send_payment", serde_json::json!({
        "to": "0xRecipient",
        "amount": "5.00",
        "asset": "USDC"
    }));

    let response = server.handle_request(&request);
    assert!(response.error.is_none(), "send_payment should succeed: {:?}", response.error);

    // Parse the tool result from the content text
    let tool_result = extract_tool_result(&response);

    // Verify response fields
    assert!(tool_result.get("tx_id").is_some(), "Should have tx_id");
    assert_eq!(tool_result["status"], "pending");
    assert_eq!(tool_result["amount"], "5.00");
    assert_eq!(tool_result["asset"], "USDC");
    assert_eq!(tool_result["to"], "0xRecipient");

    // Verify the transaction was persisted in the DB
    let tx_id = tool_result["tx_id"].as_str().unwrap();
    let tx = queries::get_transaction(&db, tx_id).unwrap();
    assert_eq!(tx.agent_id.as_deref(), Some(agent_id.as_str()));
    assert_eq!(tx.amount, "5.00");
    assert_eq!(tx.asset, "USDC");
    assert_eq!(tx.recipient.as_deref(), Some("0xRecipient"));
}

// =========================================================================
// test_mcp_check_balance_e2e
// =========================================================================

#[test]
fn test_mcp_check_balance_e2e() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let request = make_tools_call_request(2, "check_balance", serde_json::json!({}));

    let response = server.handle_request(&request);
    let tool_result = extract_tool_result(&response);

    // Verify response has balance and asset fields
    assert!(tool_result.get("balance").is_some(), "Should have balance field");
    assert!(tool_result.get("asset").is_some(), "Should have asset field");
    assert_eq!(tool_result["asset"], "USDC");
}

// =========================================================================
// test_mcp_get_spending_limits_e2e
// =========================================================================

#[test]
fn test_mcp_get_spending_limits_e2e() {
    let (db, agent_id) = setup_agent_and_db("50", "500", "2500", "10000", "25");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let request = make_tools_call_request(3, "get_spending_limits", serde_json::json!({}));

    let response = server.handle_request(&request);
    let tool_result = extract_tool_result(&response);

    // Verify all spending limit fields
    assert_eq!(tool_result["per_tx_max"], "50");
    assert_eq!(tool_result["daily_cap"], "500");
    assert_eq!(tool_result["weekly_cap"], "2500");
    assert_eq!(tool_result["monthly_cap"], "10000");
    assert_eq!(tool_result["auto_approve_max"], "25");
    assert!(tool_result.get("allowlist").is_some(), "Should have allowlist");
}

// =========================================================================
// test_mcp_invalid_token_rejected
// =========================================================================

#[test]
fn test_mcp_invalid_token_rejected() {
    let db = setup_test_db();

    // No agents in DB -- any token should fail
    let result = McpServer::validate_token(db.clone(), "anb_invalid_token_xyz");
    assert!(result.is_err(), "Invalid token should be rejected");

    match result.unwrap_err() {
        tally_agentic_wallet_lib::error::AppError::InvalidToken => {}
        other => panic!("Expected InvalidToken, got: {:?}", other),
    }
}

#[test]
fn test_mcp_invalid_token_with_agents_present() {
    // Create an agent with a known SHA-256 token hash, then try a wrong token
    let (db, _agent_id, _valid_token) = setup_agent_with_sha256_token("TokenBot");

    let result = McpServer::validate_token(db.clone(), "anb_completely_wrong_token");
    assert!(result.is_err(), "Wrong token should be rejected even with agents present");

    match result.unwrap_err() {
        tally_agentic_wallet_lib::error::AppError::InvalidToken => {}
        other => panic!("Expected InvalidToken, got: {:?}", other),
    }
}

// =========================================================================
// test_mcp_valid_token_accepted
// =========================================================================

#[test]
fn test_mcp_valid_token_accepted() {
    let (db, agent_id, valid_token) = setup_agent_with_sha256_token("ValidTokenBot");

    let server = McpServer::validate_token(db.clone(), &valid_token).unwrap();
    assert_eq!(server.agent_id(), agent_id, "Server should be bound to correct agent");
}

// =========================================================================
// test_mcp_list_tools
// =========================================================================

#[test]
fn test_mcp_list_tools() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(10)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(&request);
    assert!(response.error.is_none(), "tools/list should not error: {:?}", response.error);

    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 6, "Should return all 6 tools, got {}", tools.len());

    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"send_payment"), "Missing send_payment");
    assert!(tool_names.contains(&"check_balance"), "Missing check_balance");
    assert!(tool_names.contains(&"get_spending_limits"), "Missing get_spending_limits");
    assert!(tool_names.contains(&"request_limit_increase"), "Missing request_limit_increase");
    assert!(tool_names.contains(&"get_transactions"), "Missing get_transactions");
    assert!(tool_names.contains(&"register_agent"), "Missing register_agent");

    // Verify each tool has required schema fields
    for tool in tools {
        assert!(tool.get("name").is_some(), "Tool should have name");
        assert!(tool.get("description").is_some(), "Tool should have description");
        assert!(tool.get("input_schema").is_some(), "Tool should have input_schema");
    }
}

// =========================================================================
// test_mcp_initialize
// =========================================================================

#[test]
fn test_mcp_initialize_e2e() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(0)),
        method: "initialize".to_string(),
        params: None,
    };

    let response = server.handle_request(&request);
    assert!(response.error.is_none(), "initialize should succeed");
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, Some(serde_json::json!(0)));

    let result = response.result.unwrap();
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert_eq!(result["serverInfo"]["name"], "tally-agentic-wallet-mcp");
    assert_eq!(result["serverInfo"]["version"], "0.1.0");
    assert!(result["capabilities"]["tools"].is_object());
}

// =========================================================================
// test_mcp_spending_policy_enforced (no server-side enforcement yet,
// but we verify agents with different policies get different limits)
// =========================================================================

#[test]
fn test_mcp_spending_policy_per_agent_isolation() {
    let db = setup_test_db();

    // Agent A: low limits
    let agent_a = create_test_agent("AgentLow", AgentStatus::Active);
    let id_a = agent_a.id.clone();
    queries::insert_agent(&db, &agent_a).unwrap();
    let policy_a = create_test_spending_policy(&id_a, "10", "100", "500", "2000", "5");
    queries::insert_spending_policy(&db, &policy_a).unwrap();

    // Agent B: high limits
    let agent_b = create_test_agent("AgentHigh", AgentStatus::Active);
    let id_b = agent_b.id.clone();
    queries::insert_agent(&db, &agent_b).unwrap();
    let policy_b = create_test_spending_policy(&id_b, "1000", "10000", "50000", "200000", "500");
    queries::insert_spending_policy(&db, &policy_b).unwrap();

    let server_a = McpServer::new_with_agent_id(db.clone(), id_a.clone()).unwrap();
    let server_b = McpServer::new_with_agent_id(db.clone(), id_b.clone()).unwrap();

    // Agent A sees low limits
    let req_a = make_tools_call_request(1, "get_spending_limits", serde_json::json!({}));
    let limits_a = extract_tool_result(&server_a.handle_request(&req_a));
    assert_eq!(limits_a["per_tx_max"], "10");
    assert_eq!(limits_a["daily_cap"], "100");

    // Agent B sees high limits
    let req_b = make_tools_call_request(2, "get_spending_limits", serde_json::json!({}));
    let limits_b = extract_tool_result(&server_b.handle_request(&req_b));
    assert_eq!(limits_b["per_tx_max"], "1000");
    assert_eq!(limits_b["daily_cap"], "10000");
}

// =========================================================================
// test_mcp_transaction_isolation - agent A's tx not visible to agent B
// =========================================================================

#[test]
fn test_mcp_transaction_isolation_between_agents() {
    let db = setup_test_db();

    let agent_a = create_test_agent("TxIsoA", AgentStatus::Active);
    let id_a = agent_a.id.clone();
    queries::insert_agent(&db, &agent_a).unwrap();
    let policy_a = create_test_spending_policy(&id_a, "100", "1000", "5000", "20000", "50");
    queries::insert_spending_policy(&db, &policy_a).unwrap();

    let agent_b = create_test_agent("TxIsoB", AgentStatus::Active);
    let id_b = agent_b.id.clone();
    queries::insert_agent(&db, &agent_b).unwrap();
    let policy_b = create_test_spending_policy(&id_b, "100", "1000", "5000", "20000", "50");
    queries::insert_spending_policy(&db, &policy_b).unwrap();

    let server_a = McpServer::new_with_agent_id(db.clone(), id_a.clone()).unwrap();
    let server_b = McpServer::new_with_agent_id(db.clone(), id_b.clone()).unwrap();

    // Agent A sends a payment
    let send_req = make_tools_call_request(1, "send_payment", serde_json::json!({
        "to": "0xTarget",
        "amount": "20.00"
    }));
    let send_resp = server_a.handle_request(&send_req);
    assert!(send_resp.error.is_none(), "Agent A send should succeed");

    // Agent A sees the transaction
    let get_txs_a = make_tools_call_request(2, "get_transactions", serde_json::json!({}));
    let txs_a = extract_tool_result(&server_a.handle_request(&get_txs_a));
    assert_eq!(txs_a["transactions"].as_array().unwrap().len(), 1);

    // Agent B does NOT see Agent A's transaction
    let get_txs_b = make_tools_call_request(3, "get_transactions", serde_json::json!({}));
    let txs_b = extract_tool_result(&server_b.handle_request(&get_txs_b));
    assert_eq!(
        txs_b["transactions"].as_array().unwrap().len(),
        0,
        "Agent B should not see Agent A's transactions"
    );
}

// =========================================================================
// test_mcp_suspended_agent_cannot_create_server
// =========================================================================

#[test]
fn test_mcp_suspended_agent_cannot_create_server() {
    let db = setup_test_db();
    let agent = create_test_agent("SuspendedBot", AgentStatus::Suspended);
    queries::insert_agent(&db, &agent).unwrap();

    let result = McpServer::new_with_agent_id(db.clone(), agent.id.clone());
    assert!(result.is_err(), "Suspended agent should not create MCP server");
}

// =========================================================================
// test_mcp_pending_agent_cannot_create_server
// =========================================================================

#[test]
fn test_mcp_pending_agent_cannot_create_server() {
    let db = setup_test_db();
    let agent = create_test_agent("PendingBot", AgentStatus::Pending);
    queries::insert_agent(&db, &agent).unwrap();

    let result = McpServer::new_with_agent_id(db.clone(), agent.id.clone());
    assert!(result.is_err(), "Pending agent should not create MCP server");
}

// =========================================================================
// test_mcp_full_lifecycle: initialize -> tools/list -> tools/call -> verify
// =========================================================================

#[test]
fn test_mcp_full_lifecycle() {
    let (db, agent_id, token) = setup_agent_with_sha256_token("LifecycleBot");

    // Step 1: Validate token and create server
    let server = McpServer::validate_token(db.clone(), &token).unwrap();
    assert_eq!(server.agent_id(), agent_id);

    // Step 2: Initialize
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: None,
    };
    let init_resp = server.handle_request(&init_req);
    assert!(init_resp.error.is_none(), "initialize should succeed");

    // Step 3: List tools
    let list_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(2)),
        method: "tools/list".to_string(),
        params: None,
    };
    let list_resp = server.handle_request(&list_req);
    assert!(list_resp.error.is_none(), "tools/list should succeed");
    let tools = list_resp.result.unwrap()["tools"].as_array().unwrap().len();
    assert_eq!(tools, 6);

    // Step 4: Check balance
    let balance_req = make_tools_call_request(3, "check_balance", serde_json::json!({}));
    let balance_resp = server.handle_request(&balance_req);
    let balance_result = extract_tool_result(&balance_resp);
    assert!(balance_result.get("balance").is_some());

    // Step 5: Send payment
    let send_req = make_tools_call_request(4, "send_payment", serde_json::json!({
        "to": "0xLifecycleRecipient",
        "amount": "10.00",
        "asset": "USDC",
        "memo": "lifecycle test"
    }));
    let send_resp = server.handle_request(&send_req);
    let send_result = extract_tool_result(&send_resp);
    assert!(send_result.get("tx_id").is_some());
    assert_eq!(send_result["status"], "pending");

    // Step 6: Get transactions -- should see our payment
    let txs_req = make_tools_call_request(5, "get_transactions", serde_json::json!({}));
    let txs_result = extract_tool_result(&server.handle_request(&txs_req));
    let txs = txs_result["transactions"].as_array().unwrap();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0]["amount"], "10.00");

    // Step 7: Get spending limits
    let limits_req = make_tools_call_request(6, "get_spending_limits", serde_json::json!({}));
    let limits_result = extract_tool_result(&server.handle_request(&limits_req));
    assert_eq!(limits_result["per_tx_max"], "100");
    assert_eq!(limits_result["daily_cap"], "1000");
}

// =========================================================================
// test_mcp_unknown_method_returns_error
// =========================================================================

#[test]
fn test_mcp_unknown_method_returns_error() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(99)),
        method: "unknown/method".to_string(),
        params: None,
    };

    let response = server.handle_request(&request);
    assert!(response.error.is_some(), "Unknown method should return error");
    assert_eq!(response.error.unwrap().code, -32601);
}

// =========================================================================
// test_mcp_unknown_tool_returns_error
// =========================================================================

#[test]
fn test_mcp_unknown_tool_returns_error() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let request = make_tools_call_request(1, "nonexistent_tool", serde_json::json!({}));
    let response = server.handle_request(&request);

    assert!(response.error.is_some(), "Unknown tool should return error");
    let error = response.error.unwrap();
    assert!(error.message.contains("nonexistent_tool"));
}

// =========================================================================
// test_mcp_send_payment_missing_fields
// =========================================================================

#[test]
fn test_mcp_send_payment_missing_fields() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    // Missing 'to'
    let req1 = make_tools_call_request(1, "send_payment", serde_json::json!({
        "amount": "10"
    }));
    let resp1 = server.handle_request(&req1);
    assert!(resp1.error.is_some(), "Missing 'to' should error");

    // Missing 'amount'
    let req2 = make_tools_call_request(2, "send_payment", serde_json::json!({
        "to": "0x123"
    }));
    let resp2 = server.handle_request(&req2);
    assert!(resp2.error.is_some(), "Missing 'amount' should error");
}

// =========================================================================
// test_mcp_request_limit_increase_e2e
// =========================================================================

#[test]
fn test_mcp_request_limit_increase_e2e() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let request = make_tools_call_request(1, "request_limit_increase", serde_json::json!({
        "new_per_tx_max": "500",
        "new_daily_cap": "5000",
        "reason": "Need higher limits for vendor payments"
    }));

    let response = server.handle_request(&request);
    let tool_result = extract_tool_result(&response);

    assert!(tool_result.get("request_id").is_some(), "Should have request_id");
    assert_eq!(tool_result["status"], "pending");
    assert!(tool_result["message"].as_str().unwrap().contains("approval"));
}

// =========================================================================
// test_mcp_json_rpc_response_format
// =========================================================================

#[test]
fn test_mcp_json_rpc_response_format() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    // Success response
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(42)),
        method: "initialize".to_string(),
        params: None,
    };
    let response = server.handle_request(&request);
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 42);
    assert!(json.get("result").is_some());
    assert!(json.get("error").is_none(), "Error should be absent in success response");

    // Error response
    let err_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(43)),
        method: "bad/method".to_string(),
        params: None,
    };
    let err_response = server.handle_request(&err_request);
    let err_json = serde_json::to_value(&err_response).unwrap();
    assert_eq!(err_json["jsonrpc"], "2.0");
    assert_eq!(err_json["id"], 43);
    assert!(err_json.get("result").is_none(), "Result should be absent in error response");
    assert!(err_json.get("error").is_some());
    assert!(err_json["error"]["code"].is_number());
    assert!(err_json["error"]["message"].is_string());
}

// =========================================================================
// test_mcp_send_payment_enforces_spending_policy
// Agent with per_tx_max:10, send 15 via MCP -> denied
// =========================================================================

#[test]
fn test_mcp_send_payment_enforces_spending_policy() {
    let (db, agent_id) = setup_agent_and_db("10", "1000", "5000", "20000", "5");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    // Send 15 which exceeds per_tx_max of 10
    let request = make_tools_call_request(1, "send_payment", serde_json::json!({
        "to": "0xRecipient",
        "amount": "15",
        "asset": "USDC"
    }));

    let response = server.handle_request(&request);
    assert!(response.error.is_some(), "Payment exceeding per_tx_max should be denied");

    let error = response.error.unwrap();
    assert_eq!(error.code, -32001, "Should return PolicyViolation error code");
    assert!(
        error.message.contains("per-tx limit") || error.message.contains("policy"),
        "Error should mention policy violation, got: {}",
        error.message
    );

    // Verify a denied transaction was recorded for audit
    let (txs, _total) = queries::list_transactions_paginated(&db, Some(&agent_id), None, 10, 0).unwrap();
    assert_eq!(txs.len(), 1, "Should have one denied transaction");
    assert_eq!(txs[0].status.to_string(), "denied");
}

// =========================================================================
// test_mcp_send_payment_enforces_daily_cap
// Agent with daily_cap:20, send 15 (ok), send 10 (denied, 25>20)
// =========================================================================

#[test]
fn test_mcp_send_payment_enforces_daily_cap() {
    let (db, agent_id) = setup_agent_and_db("100", "20", "5000", "20000", "100");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    // First payment: 15 under daily_cap of 20 -> should succeed
    let req1 = make_tools_call_request(1, "send_payment", serde_json::json!({
        "to": "0xRecipient",
        "amount": "15",
        "asset": "USDC"
    }));
    let resp1 = server.handle_request(&req1);
    assert!(resp1.error.is_none(), "First payment of 15 should succeed (under daily cap 20): {:?}", resp1.error);

    // Second payment: 10 would bring total to 25 > daily_cap of 20 -> denied
    let req2 = make_tools_call_request(2, "send_payment", serde_json::json!({
        "to": "0xRecipient",
        "amount": "10",
        "asset": "USDC"
    }));
    let resp2 = server.handle_request(&req2);
    assert!(resp2.error.is_some(), "Second payment should be denied (25 > daily cap 20)");

    let error = resp2.error.unwrap();
    assert_eq!(error.code, -32001, "Should return PolicyViolation error code");
    assert!(
        error.message.contains("daily cap"),
        "Error should mention daily cap, got: {}",
        error.message
    );
}

// =========================================================================
// test_mcp_send_payment_enforces_kill_switch
// Activate kill switch, send via MCP -> denied
// =========================================================================

#[test]
fn test_mcp_send_payment_enforces_kill_switch() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    // Activate the kill switch via global policy
    let global_policy = GlobalPolicy {
        id: "default".to_string(),
        daily_cap: "0".to_string(),
        weekly_cap: "0".to_string(),
        monthly_cap: "0".to_string(),
        min_reserve_balance: "0".to_string(),
        kill_switch_active: true,
        kill_switch_reason: "Emergency shutdown".to_string(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    queries::upsert_global_policy(&db, &global_policy).unwrap();

    // Try to send -- should be denied by kill switch
    let request = make_tools_call_request(1, "send_payment", serde_json::json!({
        "to": "0xRecipient",
        "amount": "5",
        "asset": "USDC"
    }));
    let response = server.handle_request(&request);
    assert!(response.error.is_some(), "Payment should be denied when kill switch is active");

    let error = response.error.unwrap();
    assert!(
        error.message.contains("kill switch") || error.message.contains("Emergency"),
        "Error should mention kill switch, got: {}",
        error.message
    );
}

// =========================================================================
// test_mcp_send_payment_updates_spending_ledger
// Send via MCP, verify spending ledger updated with correct period key format
// =========================================================================

#[test]
fn test_mcp_send_payment_updates_spending_ledger() {
    let (db, agent_id) = setup_agent_and_db("100", "1000", "5000", "20000", "50");
    let server = McpServer::new_with_agent_id(db.clone(), agent_id.clone()).unwrap();

    let now = chrono::Utc::now();
    let expected_daily_key = daily_period_key(&now);
    let expected_weekly_key = weekly_period_key(&now);
    let expected_monthly_key = monthly_period_key(&now);

    // Send a payment
    let request = make_tools_call_request(1, "send_payment", serde_json::json!({
        "to": "0xRecipient",
        "amount": "25.50",
        "asset": "USDC"
    }));
    let response = server.handle_request(&request);
    assert!(response.error.is_none(), "Payment should succeed: {:?}", response.error);

    // Verify spending ledger entries exist with correct period key format
    let daily_ledger = queries::get_spending_for_period(&db, &agent_id, &expected_daily_key)
        .unwrap()
        .expect("Daily ledger entry should exist");
    assert_eq!(daily_ledger.total, "25.50", "Daily ledger should show 25.50, got {}", daily_ledger.total);
    assert_eq!(daily_ledger.tx_count, 1, "Daily ledger should show 1 transaction");

    let weekly_ledger = queries::get_spending_for_period(&db, &agent_id, &expected_weekly_key)
        .unwrap()
        .expect("Weekly ledger entry should exist");
    assert_eq!(weekly_ledger.total, "25.50", "Weekly ledger should show 25.50");

    let monthly_ledger = queries::get_spending_for_period(&db, &agent_id, &expected_monthly_key)
        .unwrap()
        .expect("Monthly ledger entry should exist");
    assert_eq!(monthly_ledger.total, "25.50", "Monthly ledger should show 25.50");

    // Verify the period key format uses the "daily:YYYY-MM-DD" format from spending_policy.rs
    assert!(expected_daily_key.starts_with("daily:"), "Daily key should start with 'daily:', got {}", expected_daily_key);
    assert!(expected_weekly_key.starts_with("weekly:"), "Weekly key should start with 'weekly:', got {}", expected_weekly_key);
    assert!(expected_monthly_key.starts_with("monthly:"), "Monthly key should start with 'monthly:', got {}", expected_monthly_key);
}
