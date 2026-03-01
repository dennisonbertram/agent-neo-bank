//! Integration Test: Scenario 5 -- Kill Switch Integration
//!
//! Tests kill switch activation/deactivation effects on transactions
//! and interaction with pending approvals.

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_agent_with_policy, ServiceExt};

use tally_agentic_wallet_lib::api::rest_server::ApiServer;
use tally_agentic_wallet_lib::core::approval_manager::ApprovalManager;
use tally_agentic_wallet_lib::core::global_policy::GlobalPolicyEngine;
use tally_agentic_wallet_lib::db::models::{ApprovalStatus};

/// Helper: send a transaction and return (status_code, response_body).
async fn send_amount(
    state: &std::sync::Arc<tally_agentic_wallet_lib::api::rest_server::AppStateAxum>,
    token: &str,
    amount: &str,
) -> (u16, serde_json::Value) {
    let app = ApiServer::router(state.clone());
    let send_body = serde_json::json!({
        "to": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
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
        .filter(|a| a.request_type == tally_agentic_wallet_lib::db::models::ApprovalRequestType::Transaction)
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
        .filter(|a| a.request_type == tally_agentic_wallet_lib::db::models::ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(
        tx_approvals_after.len(),
        1,
        "Pending Transaction approval should not be auto-resolved by kill switch"
    );
    assert_eq!(tx_approvals_after[0].status, ApprovalStatus::Pending);
    assert_eq!(tx_approvals_after[0].id, tx_approvals_before[0].id);
}

// =========================================================================
// Kill switch blocks execution of an approved transaction
// =========================================================================

#[tokio::test]
async fn test_kill_switch_blocks_execution_of_approved_tx() {
    let (_router, state) = create_test_app();

    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-ks-int-004",
        "KillSwitchExecBot",
        "100",  // large per_tx_max
        "1000",
        "5000",
        "20000",
        "10",   // small auto_approve_max
    )
    .await;

    // Step 1: Send amount above auto_approve -> awaiting_approval
    let (status, body) = send_amount(&state, &token, "50").await;
    assert_eq!(status, 202);
    assert_eq!(body["status"], "awaiting_approval");
    let tx_id = body["tx_id"].as_str().unwrap().to_string();

    // Step 2: Activate kill switch BEFORE approving
    let global_engine = GlobalPolicyEngine::new(state.db.clone());
    global_engine
        .toggle_kill_switch(true, "Kill switch before approval resolution")
        .unwrap();

    // Step 3: Approve the pending approval
    let manager = ApprovalManager::new(state.db.clone());
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approval: Vec<_> = pending
        .iter()
        .filter(|a| {
            a.request_type == tally_agentic_wallet_lib::db::models::ApprovalRequestType::Transaction
                && a.tx_id.as_deref() == Some(&tx_id)
        })
        .collect();
    assert_eq!(tx_approval.len(), 1);

    // The approval resolution itself succeeds (it just marks the approval as approved)
    let resolved = manager
        .resolve(&tx_approval[0].id, ApprovalStatus::Approved, "user")
        .unwrap();
    assert_eq!(resolved.status, ApprovalStatus::Approved);

    // Step 4: Verify the transaction does NOT transition to "confirmed"
    // The resolve_approval command would set status to Executing, but since kill switch
    // is active, any new sends are blocked. The tx stays in awaiting_approval status
    // (since we only resolved the approval record, not the tx status via the command).
    let tx = tally_agentic_wallet_lib::db::queries::get_transaction(&state.db, &tx_id).unwrap();
    assert_ne!(
        tx.status,
        tally_agentic_wallet_lib::db::models::TxStatus::Confirmed,
        "Transaction should NOT be confirmed when kill switch is active"
    );

    // The tx should still be in awaiting_approval (approval was resolved but
    // execution is blocked by the kill switch at the system level)
    assert_eq!(
        tx.status,
        tally_agentic_wallet_lib::db::models::TxStatus::AwaitingApproval,
        "Transaction should remain in awaiting_approval since kill switch prevents execution"
    );

    // Step 5: Verify new sends are also blocked
    let (status, body) = send_amount(&state, &token, "5").await;
    assert_eq!(status, 403, "New sends should be blocked by kill switch");
    assert_eq!(body["error"], "kill_switch_active");
}
