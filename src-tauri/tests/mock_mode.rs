//! Integration Test: Scenario 5 — Mock Mode
//!
//! Tests that the application works correctly in mock mode:
//! health reports mock_mode=true, balance returns fake data,
//! and transactions complete with mock CLI responses.

mod common;

use axum::body::Body;
use common::{
    bearer_request, body_json, create_test_app, create_test_app_with_config,
    register_agent_with_policy, ServiceExt,
};
use http::Request;

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::config::AppConfig;

// =========================================================================
// Step 2: Health endpoint reports mock_mode: true
// =========================================================================

#[tokio::test]
async fn test_mock_mode_health_endpoint() {
    // Step 1: Create state with mock mode (default_test has mock_mode=true)
    let (_router, state) = create_test_app();

    // Step 2: GET /v1/health -> mock_mode: true
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = body_json(response).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["mock_mode"], true, "Health should report mock_mode: true");
    assert!(body["version"].is_string());
    assert!(body["network"].is_string());
}

// =========================================================================
// Step 3: Balance returns fake data in mock mode
// =========================================================================

#[tokio::test]
async fn test_mock_mode_balance_returns_fake_data() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = common::register_approve_and_get_token(
        &state,
        "INV-mock-001",
        "MockBalanceBot",
    )
    .await;

    // Step 3: GET /v1/balance -> returns fake balance
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            "/v1/balance",
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = body_json(response).await;
    assert!(body["balance_visible"].as_bool().unwrap());
    let balance = body["balance"].as_str().unwrap();
    assert!(
        !balance.is_empty(),
        "Balance should be a non-empty string in mock mode"
    );
    let asset = body["asset"].as_str().unwrap();
    assert_eq!(asset, "USDC", "Asset should be USDC");
}

// =========================================================================
// Steps 4-6: Full mock mode lifecycle (register, approve, send, poll)
// =========================================================================

#[tokio::test]
async fn test_mock_mode_full_send_lifecycle() {
    let (_router, state) = create_test_app();

    // Step 4: Register agent, approve, get token
    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-mock-002",
        "MockSendBot",
        "100",
        "1000",
        "5000",
        "20000",
        "50",
    )
    .await;

    // Step 5: POST /v1/send -> 202, mock CLI returns fake tx_hash
    let app = ApiServer::router(state.clone());
    let send_body = serde_json::json!({
        "to": "0xMockRecipient",
        "amount": "10.00",
        "asset": "USDC",
        "description": "Mock mode test payment"
    });
    let response = app
        .oneshot(bearer_request(
            "POST",
            "/v1/send",
            &token,
            Body::from(serde_json::to_string(&send_body).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), 202);
    let body = body_json(response).await;
    assert_eq!(body["status"], "executing");
    let tx_id = body["tx_id"].as_str().unwrap().to_string();

    // Step 6: Wait and poll tx -> confirmed with fake hash
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

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
    let body = body_json(response).await;
    assert_eq!(body["status"], "confirmed");
    let chain_tx_hash = body["chain_tx_hash"].as_str().unwrap();
    assert!(
        !chain_tx_hash.is_empty(),
        "Mock tx should have a chain_tx_hash: {}",
        chain_tx_hash
    );
}

// =========================================================================
// Additional: Non-mock mode health check
// =========================================================================

#[tokio::test]
async fn test_non_mock_mode_health_endpoint() {
    let mut config = AppConfig::default_test();
    config.mock_mode = false;
    let (_router, state) = create_test_app_with_config(config);

    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = body_json(response).await;
    assert_eq!(body["mock_mode"], false, "Health should report mock_mode: false");
}

// =========================================================================
// Additional: Multiple sends in mock mode accumulate correctly
// =========================================================================

#[tokio::test]
async fn test_mock_mode_multiple_sends_accumulate() {
    let (_router, state) = create_test_app();

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-mock-003",
        "MockMultiBot",
        "50",
        "200",
        "5000",
        "20000",
        "50",
    )
    .await;

    // Send three transactions
    for i in 0..3 {
        let app = ApiServer::router(state.clone());
        let send_body = serde_json::json!({
            "to": "0xRecipient",
            "amount": "10.00",
            "asset": "USDC"
        });
        let response = app
            .oneshot(bearer_request(
                "POST",
                "/v1/send",
                &token,
                Body::from(serde_json::to_string(&send_body).unwrap()),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), 202, "Send {} should succeed", i + 1);

        // Wait for each to complete before the next
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // List transactions -> should have at least 3
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            "/v1/transactions?limit=20",
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = body_json(response).await;
    assert!(
        body["total"].as_i64().unwrap() >= 3,
        "Should have at least 3 transactions, got: {}",
        body["total"]
    );
}
