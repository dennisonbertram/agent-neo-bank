//! Integration Test: Scenario 3 — Global Policy Enforcement
//!
//! Tests that a global daily cap applies across multiple agents,
//! preventing any agent from sending once the global total is exceeded.

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app_with_config, register_agent_with_policy, ServiceExt};

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::config::AppConfig;
use agent_neo_bank_lib::db::models::GlobalPolicy;
use agent_neo_bank_lib::db::queries::upsert_global_policy;

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
// Full scenario: Two agents, global daily cap of 50
// =========================================================================

#[tokio::test]
async fn test_global_daily_cap_enforcement_across_agents() {
    let config = AppConfig::default_test();
    let (_router, state) = create_test_app_with_config(config);

    // Set global daily cap to 50
    let global_policy = GlobalPolicy {
        id: "default".to_string(),
        daily_cap: "50".to_string(),
        weekly_cap: "0".to_string(),
        monthly_cap: "0".to_string(),
        min_reserve_balance: "0".to_string(),
        kill_switch_active: false,
        kill_switch_reason: String::new(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    upsert_global_policy(&state.db, &global_policy).unwrap();

    // Create two agents. Each has per_tx_max:30, daily_cap:100 (generous per-agent limits)
    let (_agent_a_id, token_a) = register_agent_with_policy(
        &state,
        "INV-global-001",
        "AgentA",
        "30",   // per_tx_max
        "100",  // daily_cap (per agent)
        "5000",
        "20000",
        "30",   // auto_approve_max (matches per_tx_max so everything auto-approves)
    )
    .await;

    let (_agent_b_id, token_b) = register_agent_with_policy(
        &state,
        "INV-global-002",
        "AgentB",
        "30",
        "100",
        "5000",
        "20000",
        "30",
    )
    .await;

    // Step 2: Agent A sends 25 -> 202 (global: 25)
    let (status, body) = send_amount(&state, &token_a, "25").await;
    assert_eq!(status, 202, "Agent A send 25 should succeed");
    assert_eq!(body["status"], "executing");

    // Wait for background execution to update global spending ledger
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 3: Agent B sends 20 -> 202 (global: 25+20=45, within 50)
    let (status, body) = send_amount(&state, &token_b, "20").await;
    assert_eq!(status, 202, "Agent B send 20 should succeed (global: 45)");
    assert_eq!(body["status"], "executing");

    // Wait for background execution
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Step 4: Agent A sends 10 -> 403 (global: 45+10=55, exceeds 50)
    let (status, body) = send_amount(&state, &token_a, "10").await;
    assert_eq!(
        status, 403,
        "Agent A send 10 should fail (global would be 55, exceeds 50)"
    );
    assert!(
        body["error"] == "policy_denied" || body["reason"].as_str().map_or(false, |r| r.contains("Global")),
        "Should be denied due to global policy: {:?}",
        body
    );

    // Step 5: Agent B sends 3 -> 403 (global: 45+3=48... actually this should pass!)
    // Wait -- the scenario says 403 for this case too, but 45+3=48 < 50. Let me re-read...
    // The scenario says: "Agent B sends 3 -> 403 (same reason)" -- this implies global is 55
    // But that's wrong because step 4 was denied, so global is still 45.
    // Let me follow the original scenario intent: after two successful sends totaling 45,
    // the remaining cap is 5. So Agent B sends 6 to exceed it.
    let (status, body) = send_amount(&state, &token_b, "6").await;
    assert_eq!(
        status, 403,
        "Agent B send 6 should fail (global would be 51, exceeds 50)"
    );
    assert!(
        body["error"] == "policy_denied" || body["reason"].as_str().map_or(false, |r| r.contains("Global")),
        "Should be denied due to global policy: {:?}",
        body
    );
}

// =========================================================================
// Additional: Global cap allows exactly at boundary
// =========================================================================

#[tokio::test]
async fn test_global_daily_cap_allows_at_exact_boundary() {
    let config = AppConfig::default_test();
    let (_router, state) = create_test_app_with_config(config);

    // Set global daily cap to 50
    let global_policy = GlobalPolicy {
        id: "default".to_string(),
        daily_cap: "50".to_string(),
        weekly_cap: "0".to_string(),
        monthly_cap: "0".to_string(),
        min_reserve_balance: "0".to_string(),
        kill_switch_active: false,
        kill_switch_reason: String::new(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    upsert_global_policy(&state.db, &global_policy).unwrap();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-global-003",
        "BoundaryAgent",
        "50",   // per_tx_max
        "100",  // daily_cap
        "5000",
        "20000",
        "50",
    )
    .await;

    // Send exactly 50 -> should succeed (at boundary)
    let (status, body) = send_amount(&state, &token, "50").await;
    assert_eq!(status, 202, "Send 50 should succeed (exactly at global cap)");
    assert_eq!(body["status"], "executing");

    // Wait for background execution
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Any further send should fail
    let (status, _body) = send_amount(&state, &token, "1").await;
    assert_eq!(
        status, 403,
        "Send 1 after reaching global cap should fail"
    );
}
