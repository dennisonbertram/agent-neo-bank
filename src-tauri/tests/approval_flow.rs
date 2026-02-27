//! Integration Test: Scenario 4 -- Approval Flow
//!
//! Tests the full approval lifecycle end-to-end through the REST API:
//! agent sends large tx -> approval created -> user approves/denies -> status updated

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_agent_with_policy, ServiceExt};

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::core::approval_manager::ApprovalManager;
use agent_neo_bank_lib::db::models::{ApprovalRequestType, ApprovalStatus};

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
// Scenario 4: Agent sends large tx -> approval created -> user approves
// =========================================================================

#[tokio::test]
async fn test_approval_flow_agent_sends_large_tx_user_approves() {
    let (_router, state) = create_test_app();

    // Agent with per_tx_max:100, auto_approve_max:10
    // Amounts above 10 but <= 100 require approval
    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-approval-001",
        "ApprovalFlowBot",
        "100",   // per_tx_max
        "1000",  // daily_cap
        "5000",  // weekly_cap
        "20000", // monthly_cap
        "10",    // auto_approve_max
    )
    .await;

    // Step 1: Agent sends 50 USDC (above auto_approve_max of 10)
    let (status, body) = send_amount(&state, &token, "50").await;
    assert_eq!(status, 202, "Large tx should be accepted");
    assert_eq!(body["status"], "awaiting_approval");
    let tx_id = body["tx_id"].as_str().unwrap().to_string();
    assert!(!tx_id.is_empty());

    // Step 2: Verify a Transaction approval request was created
    // Note: agent registration also creates a Registration approval, so filter by type.
    let manager = ApprovalManager::new(state.db.clone());
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals: Vec<_> = pending
        .iter()
        .filter(|a| a.request_type == ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(tx_approvals.len(), 1, "Should have exactly one pending Transaction approval");
    assert_eq!(tx_approvals[0].tx_id.as_deref(), Some(tx_id.as_str()));

    // Step 3: User approves
    let resolved = manager
        .resolve(&tx_approvals[0].id, ApprovalStatus::Approved, "user")
        .unwrap();
    assert_eq!(resolved.status, ApprovalStatus::Approved);
    assert!(resolved.resolved_at.is_some());
    assert_eq!(resolved.resolved_by.as_deref(), Some("user"));

    // Step 4: Verify transaction approval is no longer pending
    let remaining = manager.list_pending(Some(&agent_id)).unwrap();
    let remaining_tx: Vec<_> = remaining
        .iter()
        .filter(|a| a.request_type == ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(remaining_tx.len(), 0, "No pending Transaction approvals should remain");

    // Step 5: Verify transaction record exists in the DB
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            &format!("/v1/transactions/{}", tx_id),
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let tx_body = body_json(response).await;
    assert_eq!(tx_body["amount"], "50");
}

// =========================================================================
// Scenario 4 variant: Agent sends large tx -> user denies
// =========================================================================

#[tokio::test]
async fn test_approval_flow_agent_sends_large_tx_user_denies() {
    let (_router, state) = create_test_app();

    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-approval-002",
        "ApprovalDenyBot",
        "100",
        "1000",
        "5000",
        "20000",
        "10",
    )
    .await;

    // Send 50 USDC -> awaiting_approval
    let (status, body) = send_amount(&state, &token, "50").await;
    assert_eq!(status, 202);
    assert_eq!(body["status"], "awaiting_approval");
    let tx_id = body["tx_id"].as_str().unwrap().to_string();

    // User denies the approval
    let manager = ApprovalManager::new(state.db.clone());
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals: Vec<_> = pending
        .iter()
        .filter(|a| a.request_type == ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(tx_approvals.len(), 1);

    let resolved = manager
        .resolve(&tx_approvals[0].id, ApprovalStatus::Denied, "admin")
        .unwrap();
    assert_eq!(resolved.status, ApprovalStatus::Denied);
    assert!(resolved.resolved_at.is_some());
    assert_eq!(resolved.resolved_by.as_deref(), Some("admin"));

    // Verify no pending Transaction approvals remain
    let remaining = manager.list_pending(Some(&agent_id)).unwrap();
    let remaining_tx: Vec<_> = remaining
        .iter()
        .filter(|a| a.request_type == ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(remaining_tx.len(), 0);

    // Verify the transaction still exists (it was created, just the approval was denied)
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            &format!("/v1/transactions/{}", tx_id),
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

// =========================================================================
// Auto-approved transactions don't create approvals
// =========================================================================

#[tokio::test]
async fn test_approval_flow_small_tx_auto_approved() {
    let (_router, state) = create_test_app();

    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-approval-003",
        "AutoApproveBot",
        "100",
        "1000",
        "5000",
        "20000",
        "50", // auto_approve_max: 50
    )
    .await;

    // Send 5 USDC (below auto_approve_max of 50)
    let (status, body) = send_amount(&state, &token, "5").await;
    assert_eq!(status, 202);
    assert_eq!(
        body["status"], "executing",
        "Small tx should be auto-approved and executing"
    );

    // Verify NO Transaction approval requests were created
    // (Registration approval from agent setup may exist)
    let manager = ApprovalManager::new(state.db.clone());
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals: Vec<_> = pending
        .iter()
        .filter(|a| a.request_type == ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(
        tx_approvals.len(),
        0,
        "Auto-approved tx should not create Transaction approval requests"
    );

    // Wait for background execution to complete
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Verify transaction is confirmed
    let tx_id = body["tx_id"].as_str().unwrap();
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            &format!("/v1/transactions/{}", tx_id),
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let tx_body = body_json(response).await;
    assert_eq!(tx_body["status"], "confirmed");
}
