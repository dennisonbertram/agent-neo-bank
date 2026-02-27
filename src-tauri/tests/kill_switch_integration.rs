//! Integration Test: Scenario 5 -- Kill Switch Integration
//!
//! Tests kill switch activation/deactivation effects on transactions
//! and interaction with pending approvals.

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_agent_with_policy, ServiceExt};

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::core::approval_manager::ApprovalManager;
use agent_neo_bank_lib::core::global_policy::GlobalPolicyEngine;
use agent_neo_bank_lib::db::models::{ApprovalStatus};

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
// Kill switch blocks ALL agent transactions
// =========================================================================

#[tokio::test]
async fn test_kill_switch_blocks_all_transactions() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-ks-int-001",
        "KillSwitchIntBot",
        "100",
        "1000",
        "5000",
        "20000",
        "50",
    )
    .await;

    // Activate kill switch
    let global_engine = GlobalPolicyEngine::new(state.db.clone());
    global_engine
        .toggle_kill_switch(true, "Emergency shutdown test")
        .unwrap();

    // Try to send a small amount -- should be denied
    let (status, body) = send_amount(&state, &token, "5").await;
    assert_eq!(status, 403, "Kill switch should block all transactions");
    assert_eq!(
        body["error"], "kill_switch_active",
        "Error should be kill_switch_active, got: {:?}",
        body
    );
}

// =========================================================================
// Kill switch deactivation resumes transactions
// =========================================================================

#[tokio::test]
async fn test_kill_switch_deactivation_resumes_transactions() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-ks-int-002",
        "KillSwitchResumeBot",
        "100",
        "1000",
        "5000",
        "20000",
        "50",
    )
    .await;

    let global_engine = GlobalPolicyEngine::new(state.db.clone());

    // Step 1: Activate kill switch
    global_engine
        .toggle_kill_switch(true, "Temporary lockdown")
        .unwrap();

    // Step 2: Verify tx denied
    let (status, body) = send_amount(&state, &token, "5").await;
    assert_eq!(status, 403, "Tx should be denied with kill switch active");
    assert_eq!(body["error"], "kill_switch_active");

    // Step 3: Deactivate kill switch
    global_engine
        .toggle_kill_switch(false, "All clear")
        .unwrap();

    // Step 4: Verify tx now succeeds
    let (status, body) = send_amount(&state, &token, "5").await;
    assert_eq!(
        status, 202,
        "Tx should succeed after kill switch deactivated"
    );
    assert_eq!(body["status"], "executing");
}

// =========================================================================
// Kill switch doesn't auto-resolve existing pending approvals
// =========================================================================

#[tokio::test]
async fn test_kill_switch_pending_approvals_not_auto_resolved() {
    let (_router, state) = create_test_app();

    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-ks-int-003",
        "KillSwitchApprovalBot",
        "100",
        "1000",
        "5000",
        "20000",
        "10", // auto_approve_max: 10
    )
    .await;

    // Step 1: Send a tx that requires approval (above auto_approve_max)
    let (status, body) = send_amount(&state, &token, "50").await;
    assert_eq!(status, 202);
    assert_eq!(body["status"], "awaiting_approval");

    // Step 2: Verify Transaction approval exists
    let manager = ApprovalManager::new(state.db.clone());
    let pending_before = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals_before: Vec<_> = pending_before
        .iter()
        .filter(|a| a.request_type == agent_neo_bank_lib::db::models::ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(tx_approvals_before.len(), 1, "Should have one pending Transaction approval");

    // Step 3: Activate kill switch
    let global_engine = GlobalPolicyEngine::new(state.db.clone());
    global_engine
        .toggle_kill_switch(true, "Kill switch test")
        .unwrap();

    // Step 4: Verify Transaction approval is still pending (not auto-denied)
    let pending_after = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals_after: Vec<_> = pending_after
        .iter()
        .filter(|a| a.request_type == agent_neo_bank_lib::db::models::ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(
        tx_approvals_after.len(),
        1,
        "Pending Transaction approval should not be auto-resolved by kill switch"
    );
    assert_eq!(tx_approvals_after[0].status, ApprovalStatus::Pending);
    assert_eq!(tx_approvals_after[0].id, tx_approvals_before[0].id);
}
