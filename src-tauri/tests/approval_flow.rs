//! Integration Test: Scenario 4 -- Approval Flow
//!
//! Tests the full approval lifecycle end-to-end through the REST API:
//! agent sends large tx -> approval created -> user approves/denies -> status updated

mod common;

use std::time::{Duration, Instant};

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_agent_with_policy, ServiceExt};

use tally_agentic_wallet_lib::api::rest_server::ApiServer;
use tally_agentic_wallet_lib::core::approval_manager::ApprovalManager;
use tally_agentic_wallet_lib::db::models::{ApprovalRequestType, ApprovalStatus, TxStatus};
use tally_agentic_wallet_lib::db::queries;

/// Poll a transaction until it reaches one of the expected terminal statuses.
async fn wait_for_tx_status(
    state: &std::sync::Arc<tally_agentic_wallet_lib::api::rest_server::AppStateAxum>,
    tx_id: &str,
    token: &str,
    expected_statuses: &[&str],
    timeout: Duration,
) -> serde_json::Value {
    let start = Instant::now();
    loop {
        let app = ApiServer::router(state.clone());
        let resp = app
            .oneshot(bearer_request(
                "GET",
                &format!("/v1/transactions/{}", tx_id),
                token,
                Body::empty(),
            ))
            .await
            .unwrap();
        let body = body_json(resp).await;
        let status = body["status"].as_str().unwrap_or("");
        if expected_statuses.contains(&status) {
            return body;
        }
        if start.elapsed() > timeout {
            panic!(
                "Timed out waiting for tx {} to reach {:?}, current: {}",
                tx_id, expected_statuses, status
            );
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Helper: send a transaction and return (status_code, response_body).
async fn send_amount(
    state: &std::sync::Arc<tally_agentic_wallet_lib::api::rest_server::AppStateAxum>,
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

    // Poll until confirmed (instead of sleeping)
    let tx_id = body["tx_id"].as_str().unwrap();
    let tx_body = wait_for_tx_status(
        &state,
        tx_id,
        &token,
        &["confirmed"],
        Duration::from_secs(10),
    )
    .await;
    assert_eq!(tx_body["status"], "confirmed");
}

// =========================================================================
// Scenario 4: Approve -> poll tx -> confirmed with chain_tx_hash
// =========================================================================

#[tokio::test]
async fn test_approval_flow_approve_then_tx_confirmed() {
    let (_router, state) = create_test_app();

    // Register agent with per_tx_max: 100, auto_approve_max: 5
    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-approve-confirm-001",
        "ApproveConfirmBot",
        "100",   // per_tx_max
        "1000",  // daily_cap
        "5000",  // weekly_cap
        "20000", // monthly_cap
        "5",     // auto_approve_max
    )
    .await;

    // Send 50.00 -> 202 (awaiting_approval)
    let (status, body) = send_amount(&state, &token, "50").await;
    assert_eq!(status, 202, "Large tx should return 202");
    assert_eq!(body["status"], "awaiting_approval");
    let tx_id = body["tx_id"].as_str().unwrap().to_string();

    // Poll GET /v1/transactions/{tx_id} -> assert status is "awaiting_approval"
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
    assert_eq!(tx_body["status"], "awaiting_approval");

    // Resolve approval (approve)
    let manager = ApprovalManager::new(state.db.clone());
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals: Vec<_> = pending
        .iter()
        .filter(|a| a.request_type == ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(tx_approvals.len(), 1);

    manager
        .resolve(&tx_approvals[0].id, ApprovalStatus::Approved, "user")
        .unwrap();

    // Simulate the side effect that the Tauri resolve_approval command performs:
    // 1. Set tx status to Executing
    let now = chrono::Utc::now().timestamp();
    queries::update_transaction_status(
        &state.db,
        &tx_id,
        &TxStatus::Executing,
        None,
        None,
        now,
    )
    .unwrap();

    // 2. Simulate background CLI execution completing (confirmed with hash)
    let tx_record = queries::get_transaction(&state.db, &tx_id).unwrap();
    queries::update_transaction_and_ledgers_atomic(
        &state.db,
        &tx_id,
        "0xmock_approval_hash_001",
        &agent_id,
        &tx_record.amount,
        &tx_record.period_daily,
        &tx_record.period_weekly,
        &tx_record.period_monthly,
        now,
    )
    .unwrap();

    // Poll GET /v1/transactions/{tx_id} -> assert status is "confirmed" with chain_tx_hash
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
    assert_eq!(tx_body["chain_tx_hash"], "0xmock_approval_hash_001");
}

// =========================================================================
// Scenario 4: Deny -> poll tx -> denied
// =========================================================================

#[tokio::test]
async fn test_approval_flow_deny_then_tx_denied() {
    let (_router, state) = create_test_app();

    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-deny-tx-001",
        "DenyTxBot",
        "100",
        "1000",
        "5000",
        "20000",
        "5",
    )
    .await;

    // Send 50.00 -> 202 (awaiting_approval)
    let (status, body) = send_amount(&state, &token, "50").await;
    assert_eq!(status, 202);
    assert_eq!(body["status"], "awaiting_approval");
    let tx_id = body["tx_id"].as_str().unwrap().to_string();

    // Resolve approval (deny)
    let manager = ApprovalManager::new(state.db.clone());
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals: Vec<_> = pending
        .iter()
        .filter(|a| a.request_type == ApprovalRequestType::Transaction)
        .collect();
    assert_eq!(tx_approvals.len(), 1);

    manager
        .resolve(&tx_approvals[0].id, ApprovalStatus::Denied, "admin")
        .unwrap();

    // Simulate the side effect: denied approval -> tx marked denied
    let now = chrono::Utc::now().timestamp();
    queries::update_transaction_status(
        &state.db,
        &tx_id,
        &TxStatus::Denied,
        None,
        Some("Approval denied by admin"),
        now,
    )
    .unwrap();

    // Poll GET /v1/transactions/{tx_id} -> assert status is "denied"
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
    assert_eq!(tx_body["status"], "denied");
}

// =========================================================================
// Scenario 4: Full 8-step approval sequence (approve + deny + history)
// =========================================================================

#[tokio::test]
async fn test_approval_flow_full_sequence() {
    let (_router, state) = create_test_app();

    // Step 1: Register agent with per_tx_max: 100, auto_approve_max: 5
    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-full-seq-001",
        "FullSequenceBot",
        "100",
        "1000",
        "5000",
        "20000",
        "5",
    )
    .await;

    let manager = ApprovalManager::new(state.db.clone());

    // Step 2: Send 20.00 -> 202 (awaiting_approval)
    let (status, body1) = send_amount(&state, &token, "20").await;
    assert_eq!(status, 202);
    assert_eq!(body1["status"], "awaiting_approval");
    let tx_id_1 = body1["tx_id"].as_str().unwrap().to_string();

    // Step 3: Poll tx -> awaiting_approval
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            &format!("/v1/transactions/{}", tx_id_1),
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let tx_body = body_json(response).await;
    assert_eq!(tx_body["status"], "awaiting_approval");

    // Step 4: Approve -> simulate execution -> poll tx -> confirmed with chain_tx_hash
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals: Vec<_> = pending
        .iter()
        .filter(|a| {
            a.request_type == ApprovalRequestType::Transaction
                && a.tx_id.as_deref() == Some(tx_id_1.as_str())
        })
        .collect();
    assert_eq!(tx_approvals.len(), 1);

    manager
        .resolve(&tx_approvals[0].id, ApprovalStatus::Approved, "user")
        .unwrap();

    // Simulate side effects: Executing -> Confirmed
    let now = chrono::Utc::now().timestamp();
    queries::update_transaction_status(
        &state.db,
        &tx_id_1,
        &TxStatus::Executing,
        None,
        None,
        now,
    )
    .unwrap();

    let tx_record = queries::get_transaction(&state.db, &tx_id_1).unwrap();
    queries::update_transaction_and_ledgers_atomic(
        &state.db,
        &tx_id_1,
        "0xfull_seq_hash_001",
        &agent_id,
        &tx_record.amount,
        &tx_record.period_daily,
        &tx_record.period_weekly,
        &tx_record.period_monthly,
        now,
    )
    .unwrap();

    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            &format!("/v1/transactions/{}", tx_id_1),
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let tx_body = body_json(response).await;
    assert_eq!(tx_body["status"], "confirmed");
    assert_eq!(tx_body["chain_tx_hash"], "0xfull_seq_hash_001");

    // Step 5: Send another 20.00 -> 202 (awaiting_approval)
    let (status, body2) = send_amount(&state, &token, "20").await;
    assert_eq!(status, 202);
    assert_eq!(body2["status"], "awaiting_approval");
    let tx_id_2 = body2["tx_id"].as_str().unwrap().to_string();

    // Step 6: Poll tx -> awaiting_approval
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            &format!("/v1/transactions/{}", tx_id_2),
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let tx_body = body_json(response).await;
    assert_eq!(tx_body["status"], "awaiting_approval");

    // Step 7: Deny -> update tx -> poll tx -> denied
    let pending = manager.list_pending(Some(&agent_id)).unwrap();
    let tx_approvals: Vec<_> = pending
        .iter()
        .filter(|a| {
            a.request_type == ApprovalRequestType::Transaction
                && a.tx_id.as_deref() == Some(tx_id_2.as_str())
        })
        .collect();
    assert_eq!(tx_approvals.len(), 1);

    manager
        .resolve(&tx_approvals[0].id, ApprovalStatus::Denied, "admin")
        .unwrap();

    let now = chrono::Utc::now().timestamp();
    queries::update_transaction_status(
        &state.db,
        &tx_id_2,
        &TxStatus::Denied,
        None,
        Some("Approval denied by admin"),
        now,
    )
    .unwrap();

    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            &format!("/v1/transactions/{}", tx_id_2),
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let tx_body = body_json(response).await;
    assert_eq!(tx_body["status"], "denied");

    // Step 8: Verify transaction history shows both (one confirmed, one denied)
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            "/v1/transactions",
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let history = body_json(response).await;

    let txs = history["data"].as_array().unwrap();
    // Should have at least 2 transactions
    assert!(
        txs.len() >= 2,
        "Transaction history should contain at least 2 transactions, got {}",
        txs.len()
    );

    // Find our two specific transactions
    let confirmed_tx = txs.iter().find(|t| t["id"] == tx_id_1);
    let denied_tx = txs.iter().find(|t| t["id"] == tx_id_2);

    assert!(confirmed_tx.is_some(), "Confirmed transaction should appear in history");
    assert!(denied_tx.is_some(), "Denied transaction should appear in history");

    assert_eq!(confirmed_tx.unwrap()["status"], "confirmed");
    assert_eq!(denied_tx.unwrap()["status"], "denied");
}
