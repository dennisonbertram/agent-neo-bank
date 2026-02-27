//! Integration Test: Limit Increase End-to-End
//!
//! Tests the full limit increase flow: agent requests increase -> approval
//! created -> user approves/denies -> policy updated/preserved.

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_agent_with_policy, ServiceExt};

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::core::approval_manager::ApprovalManager;
use agent_neo_bank_lib::db::models::{ApprovalRequestType, ApprovalStatus};
use agent_neo_bank_lib::db::queries;

/// Helper: send a transaction and return (status_code, response_body).
async fn send_amount(
    state: &std::sync::Arc<agent_neo_bank_lib::api::rest_server::AppStateAxum>,
    token: &str,
    amount: &str,
) -> (u16, serde_json::Value) {
    let app = ApiServer::router(state.clone());
    let send_body = serde_json::json!({
        "to": "0xVendor",
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
// Full limit increase flow: request -> approve -> policy updated -> higher tx allowed
// =========================================================================

#[tokio::test]
async fn test_limit_increase_full_flow() {
    let (_router, state) = create_test_app();

    // Agent with low limits: per_tx_max:50, daily_cap:500
    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-limit-001",
        "LimitIncreaseBot",
        "50",    // per_tx_max
        "500",   // daily_cap
        "2500",  // weekly_cap
        "10000", // monthly_cap
        "25",    // auto_approve_max
    )
    .await;

    // Step 1: Verify a tx of 100 is denied (exceeds per_tx_max of 50)
    let (status, body) = send_amount(&state, &token, "100").await;
    assert_eq!(
        status, 403,
        "100 should exceed per_tx_max of 50"
    );
    assert_eq!(body["error"], "policy_denied");

    // Step 2: Create limit increase approval request via ApprovalManager
    let manager = ApprovalManager::new(state.db.clone());
    let payload = serde_json::json!({
        "current": { "per_tx_max": "50", "daily_cap": "500" },
        "proposed": { "per_tx_max": "200", "daily_cap": "2000" },
        "reason": "Need higher limits for vendor payments"
    });
    let approval = manager
        .create_request(
            &agent_id,
            ApprovalRequestType::LimitIncrease,
            payload,
            None,
            None,
        )
        .unwrap();

    // Step 3: Verify approval was created as pending
    assert_eq!(approval.status, ApprovalStatus::Pending);
    assert_eq!(approval.request_type, ApprovalRequestType::LimitIncrease);

    // Step 4: User approves
    let resolved = manager
        .resolve(&approval.id, ApprovalStatus::Approved, "admin")
        .unwrap();
    assert_eq!(resolved.status, ApprovalStatus::Approved);

    // Step 5: Simulate the side-effect of approval -- update policy
    // (In the real app, the resolve_approval Tauri command handles this)
    let resolved_payload: serde_json::Value =
        serde_json::from_str(&resolved.payload).unwrap();
    if let Some(proposed) = resolved_payload.get("proposed") {
        let mut updated_policy = queries::get_spending_policy(&state.db, &agent_id).unwrap();
        if let Some(v) = proposed.get("per_tx_max").and_then(|v| v.as_str()) {
            updated_policy.per_tx_max = v.to_string();
        }
        if let Some(v) = proposed.get("daily_cap").and_then(|v| v.as_str()) {
            updated_policy.daily_cap = v.to_string();
        }
        updated_policy.updated_at = chrono::Utc::now().timestamp();
        queries::update_spending_policy(&state.db, &updated_policy).unwrap();
    }

    // Step 6: Verify policy was updated
    let new_policy = queries::get_spending_policy(&state.db, &agent_id).unwrap();
    assert_eq!(new_policy.per_tx_max, "200");
    assert_eq!(new_policy.daily_cap, "2000");
    // Unchanged fields should be preserved
    assert_eq!(new_policy.weekly_cap, "2500");
    assert_eq!(new_policy.monthly_cap, "10000");

    // Step 7: Verify agent can now send 100 (was denied before, now below new per_tx_max of 200)
    let (status, body) = send_amount(&state, &token, "100").await;
    assert!(
        status == 202,
        "100 should now be accepted with new per_tx_max of 200, got status {}: {:?}",
        status,
        body
    );
    // 100 > auto_approve_max of 25, so it should require approval
    assert!(
        body["status"] == "executing" || body["status"] == "awaiting_approval",
        "Status should be executing or awaiting_approval, got: {}",
        body["status"]
    );
}

// =========================================================================
// Denied limit increase preserves old policy
// =========================================================================

#[tokio::test]
async fn test_limit_increase_denied_preserves_old_policy() {
    let (_router, state) = create_test_app();

    let (agent_id, token) = register_agent_with_policy(
        &state,
        "INV-limit-002",
        "LimitDenyBot",
        "50",
        "500",
        "2500",
        "10000",
        "25",
    )
    .await;

    // Create limit increase request
    let manager = ApprovalManager::new(state.db.clone());
    let payload = serde_json::json!({
        "current": { "per_tx_max": "50", "daily_cap": "500" },
        "proposed": { "per_tx_max": "200", "daily_cap": "2000" },
        "reason": "Want higher limits"
    });
    let approval = manager
        .create_request(
            &agent_id,
            ApprovalRequestType::LimitIncrease,
            payload,
            None,
            None,
        )
        .unwrap();

    // Deny the request
    let resolved = manager
        .resolve(&approval.id, ApprovalStatus::Denied, "admin")
        .unwrap();
    assert_eq!(resolved.status, ApprovalStatus::Denied);

    // Verify policy is unchanged
    let policy = queries::get_spending_policy(&state.db, &agent_id).unwrap();
    assert_eq!(policy.per_tx_max, "50", "per_tx_max should remain 50");
    assert_eq!(policy.daily_cap, "500", "daily_cap should remain 500");
    assert_eq!(policy.weekly_cap, "2500", "weekly_cap should remain 2500");
    assert_eq!(
        policy.monthly_cap, "10000",
        "monthly_cap should remain 10000"
    );

    // Verify agent still can't send above old limit
    let (status, body) = send_amount(&state, &token, "100").await;
    assert_eq!(
        status, 403,
        "100 should still exceed per_tx_max of 50 after denied increase"
    );
    assert_eq!(body["error"], "policy_denied");
}
