//! Integration Test: Scenario 2 — Spending Limit Enforcement
//!
//! Tests per-transaction max, daily cap, auto-approve thresholds,
//! and cumulative daily tracking through the HTTP API.

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_agent_with_policy, ServiceExt};

use agent_neo_bank_lib::api::rest_server::ApiServer;

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
// Step 1-2: per_tx_max enforcement — amount exceeding per_tx_max returns 403
// =========================================================================

#[tokio::test]
async fn test_spending_per_tx_max_exceeded_returns_403() {
    let (_router, state) = create_test_app();

    // Agent with per_tx_max:10, daily_cap:25, auto_approve_max:5
    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-spend-001",
        "SpendBot1",
        "10",  // per_tx_max
        "25",  // daily_cap
        "5000", // weekly_cap
        "20000", // monthly_cap
        "5",   // auto_approve_max
    )
    .await;

    // Step 2: POST /v1/send amount:15 -> 403 (exceeds per_tx_max of 10)
    let (status, body) = send_amount(&state, &token, "15").await;
    assert_eq!(status, 403, "Amount 15 exceeds per_tx_max 10, should be 403");
    assert_eq!(body["error"], "policy_denied");
}

// =========================================================================
// Step 3: Amount above auto_approve_max but within per_tx_max -> awaiting_approval
// =========================================================================

#[tokio::test]
async fn test_spending_requires_approval_above_auto_approve() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-spend-002",
        "SpendBot2",
        "10",  // per_tx_max
        "25",  // daily_cap
        "5000",
        "20000",
        "5",   // auto_approve_max
    )
    .await;

    // Step 3: POST /v1/send amount:8 -> 202 awaiting_approval (8 > auto_approve_max of 5)
    let (status, body) = send_amount(&state, &token, "8").await;
    assert_eq!(status, 202, "Amount 8 within per_tx_max 10, should be accepted");
    assert_eq!(body["status"], "awaiting_approval");
}

// =========================================================================
// Step 4-7: Cumulative daily cap tracking
// =========================================================================

#[tokio::test]
async fn test_spending_daily_cap_cumulative_enforcement() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-spend-003",
        "SpendBot3",
        "10",  // per_tx_max
        "25",  // daily_cap
        "5000",
        "20000",
        "50",  // auto_approve_max (high so everything auto-approves for simplicity)
    )
    .await;

    // Step 4: POST /v1/send amount:8 -> 202 (auto-approved, daily total: 8)
    let (status, body) = send_amount(&state, &token, "8").await;
    assert_eq!(status, 202, "First send of 8 should succeed");
    assert_eq!(body["status"], "executing");

    // Wait for background execution to complete and update ledger
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 5: POST /v1/send amount:9 -> 202 (daily: 8+9=17, within 25)
    let (status, body) = send_amount(&state, &token, "9").await;
    assert_eq!(status, 202, "Second send of 9 should succeed (daily total: 17)");
    assert_eq!(body["status"], "executing");

    // Wait for background execution
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 6: POST /v1/send amount:9 -> 403 (17+9=26, exceeds daily cap of 25)
    let (status, body) = send_amount(&state, &token, "9").await;
    assert_eq!(
        status, 403,
        "Third send of 9 should fail (daily would be 26, exceeds 25)"
    );
    assert_eq!(body["error"], "policy_denied");

    // Step 7: POST /v1/send amount:8 -> 202 (17+8=25, exactly at cap)
    let (status, body) = send_amount(&state, &token, "8").await;
    assert_eq!(
        status, 202,
        "Fourth send of 8 should succeed (daily total: 25, exactly at cap)"
    );
    assert_eq!(body["status"], "executing");
}

// =========================================================================
// Additional: auto_approve_max boundary test
// =========================================================================

#[tokio::test]
async fn test_spending_auto_approve_boundary() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-spend-004",
        "SpendBot4",
        "100",  // per_tx_max
        "1000", // daily_cap
        "5000",
        "20000",
        "10",   // auto_approve_max
    )
    .await;

    // Amount exactly at auto_approve_max -> auto-approved (executing)
    let (status, body) = send_amount(&state, &token, "10").await;
    assert_eq!(status, 202);
    assert_eq!(body["status"], "executing", "Amount at auto_approve_max should auto-approve");

    // Wait for background execution
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Amount just above auto_approve_max -> requires approval
    let (status, body) = send_amount(&state, &token, "10.01").await;
    assert_eq!(status, 202);
    assert_eq!(
        body["status"], "awaiting_approval",
        "Amount above auto_approve_max should require approval"
    );
}
