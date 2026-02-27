//! Integration Test: Scenario 4 — Kill Switch
//!
//! Tests that the global kill switch blocks ALL agent transactions
//! and that deactivating it resumes normal operation.

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_agent_with_policy, ServiceExt};

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::core::global_policy::GlobalPolicyEngine;

/// Helper: send a transaction and return (status_code, response_body).
async fn send_amount(
    state: &std::sync::Arc<agent_neo_bank_lib::api::rest_server::AppStateAxum>,
    token: &str,
    amount: &str,
) -> (u16, serde_json::Value) {
    let app = ApiServer::router(state.clone());
    let send_body = serde_json::json!({
        "to": "0xRecipient",
        "amount": amount,
        "asset": "USDC"
    });
    let response = app
        .oneshot(bearer_request(
            "POST",
            "/v1/send",
            token,
            Body::from(serde_json::to_string(&send_body).unwrap()),
        ))
        .await
        .unwrap();

    let status = response.status().as_u16();
    let body = body_json(response).await;
    (status, body)
}

// =========================================================================
// Full kill switch scenario
// =========================================================================

#[tokio::test]
async fn test_kill_switch_blocks_and_resumes() {
    let (_router, state) = create_test_app();

    // Step 1: Two agents with valid spending limits
    let (_agent_a_id, token_a) = register_agent_with_policy(
        &state,
        "INV-kill-001",
        "KillSwitchAgentA",
        "100",  // per_tx_max
        "1000", // daily_cap
        "5000",
        "20000",
        "100",  // auto_approve_max
    )
    .await;

    let (_agent_b_id, token_b) = register_agent_with_policy(
        &state,
        "INV-kill-002",
        "KillSwitchAgentB",
        "100",
        "1000",
        "5000",
        "20000",
        "100",
    )
    .await;

    // Step 2: Agent A sends 5 -> 202 (normal operation)
    let (status, body) = send_amount(&state, &token_a, "5").await;
    assert_eq!(status, 202, "Agent A send 5 should succeed before kill switch");
    assert_eq!(body["status"], "executing");

    // Wait for background execution
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 3: Activate kill switch
    let global_engine = GlobalPolicyEngine::new(state.db.clone());
    global_engine
        .toggle_kill_switch(true, "Emergency shutdown")
        .unwrap();

    // Step 4: Agent A sends 5 -> 403 kill_switch_active
    let (status, body) = send_amount(&state, &token_a, "5").await;
    assert_eq!(
        status, 403,
        "Agent A send should fail with kill switch active"
    );
    assert_eq!(
        body["error"], "kill_switch_active",
        "Error should be kill_switch_active, got: {:?}",
        body
    );

    // Step 5: Agent B sends 1 -> 403 kill_switch_active
    let (status, body) = send_amount(&state, &token_b, "1").await;
    assert_eq!(
        status, 403,
        "Agent B send should also fail with kill switch active"
    );
    assert_eq!(
        body["error"], "kill_switch_active",
        "Error should be kill_switch_active, got: {:?}",
        body
    );

    // Step 6: Deactivate kill switch
    global_engine
        .toggle_kill_switch(false, "All clear")
        .unwrap();

    // Step 7: Agent A sends 5 -> 202 (normal operation resumed)
    let (status, body) = send_amount(&state, &token_a, "5").await;
    assert_eq!(
        status, 202,
        "Agent A send should succeed after kill switch deactivated"
    );
    assert_eq!(body["status"], "executing");
}

// =========================================================================
// Additional: Kill switch with reason is propagated in response
// =========================================================================

#[tokio::test]
async fn test_kill_switch_reason_in_response() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-kill-003",
        "KillReasonBot",
        "100",
        "1000",
        "5000",
        "20000",
        "100",
    )
    .await;

    // Activate kill switch with specific reason
    let global_engine = GlobalPolicyEngine::new(state.db.clone());
    global_engine
        .toggle_kill_switch(true, "Suspicious activity detected")
        .unwrap();

    let (status, body) = send_amount(&state, &token, "5").await;
    assert_eq!(status, 403);
    assert_eq!(body["error"], "kill_switch_active");
    let reason = body["reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("kill switch") || reason.contains("Suspicious activity"),
        "Response reason should mention kill switch: {}",
        reason
    );
}
