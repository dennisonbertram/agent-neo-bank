//! Integration Test: MCP End-to-End
//!
//! Tests the MCP server lifecycle: creating a server with an agent,
//! calling tools, listing tools, and handling suspended agents.

mod common;

use tally_agentic_wallet_lib::api::mcp_server::{JsonRpcRequest, McpServer};
use tally_agentic_wallet_lib::db::models::AgentStatus;
use tally_agentic_wallet_lib::db::queries;
use tally_agentic_wallet_lib::test_helpers::{
    create_test_agent, create_test_spending_policy, setup_test_db,
};

// =========================================================================
// MCP: create server -> call send_payment -> tx created
// =========================================================================

#[test]
fn test_mcp_end_to_end_send_payment() {
    let db = setup_test_db();
    let agent = create_test_agent("McpE2EBot", AgentStatus::Active);
    queries::insert_agent(&db, &agent).unwrap();
    let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
    queries::insert_spending_policy(&db, &policy).unwrap();

    let server = McpServer::new_with_agent_id(db.clone(), agent.id.clone()).unwrap();

    // Call send_payment tool
    let result = server
        .handle_tool_call(
            "send_payment",
            serde_json::json!({
                "to": "0xRecipient",
                "amount": "25.00"
            }),
        )
        .unwrap();

    // Verify result contains tx_id
    assert!(result.get("tx_id").is_some(), "Result should have tx_id");
    let tx_id = result["tx_id"].as_str().unwrap();
    assert!(!tx_id.is_empty());

    // Verify transaction was persisted in DB
    let tx = queries::get_transaction(&db, tx_id).unwrap();
    assert_eq!(tx.agent_id.as_deref(), Some(agent.id.as_str()));
    assert_eq!(tx.amount, "25.00");
    assert_eq!(tx.asset, "USDC");
    assert_eq!(tx.recipient.as_deref(), Some("0xRecipient"));
}

// =========================================================================
// MCP: tools/list returns all 6 tools
// =========================================================================

#[test]
fn test_mcp_end_to_end_tools_list() {
    let db = setup_test_db();
    let agent = create_test_agent("McpListBot", AgentStatus::Active);
    queries::insert_agent(&db, &agent).unwrap();
    let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
    queries::insert_spending_policy(&db, &policy).unwrap();

    let server = McpServer::new_with_agent_id(db.clone(), agent.id.clone()).unwrap();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(&request);
    assert!(
        response.error.is_none(),
        "tools/list should not error: {:?}",
        response.error
    );

    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();
    assert_eq!(
        tools.len(),
        6,
        "MCP server should expose 6 tools, got: {}",
        tools.len()
    );

    // Verify expected tool names are present
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(tool_names.contains(&"send_payment"), "Should have send_payment");
    assert!(tool_names.contains(&"check_balance"), "Should have check_balance");
    assert!(
        tool_names.contains(&"get_spending_limits"),
        "Should have get_spending_limits"
    );
    assert!(
        tool_names.contains(&"request_limit_increase"),
        "Should have request_limit_increase"
    );
    assert!(
        tool_names.contains(&"get_transactions"),
        "Should have get_transactions"
    );
    assert!(
        tool_names.contains(&"register_agent"),
        "Should have register_agent"
    );
}

// =========================================================================
// MCP: suspended agent cannot create server
// =========================================================================

#[test]
fn test_mcp_suspended_agent_rejected() {
    let db = setup_test_db();
    let agent = create_test_agent("SuspendedMcpBot", AgentStatus::Suspended);
    queries::insert_agent(&db, &agent).unwrap();

    let result = McpServer::new_with_agent_id(db.clone(), agent.id.clone());
    assert!(
        result.is_err(),
        "Suspended agent should not be able to create MCP server"
    );
}
