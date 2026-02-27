//! Integration Test: Scenario 1 — Happy Path Agent Lifecycle
//!
//! Tests the complete agent lifecycle end-to-end through the REST API:
//! invitation -> registration -> approval -> token retrieval -> send -> poll tx -> list txs

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, ServiceExt};
use http::Request;

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::db::queries::insert_invitation_code;
use agent_neo_bank_lib::test_helpers::create_test_invitation;

// =========================================================================
// Step 1-4: Generate invitation, register agent, check pending status
// =========================================================================

#[tokio::test]
async fn test_lifecycle_register_returns_pending() {
    let (_router, state) = create_test_app();

    // 1. Generate invitation code (insert directly)
    let invitation = create_test_invitation("INV-lifecycle-001", "Lifecycle test");
    insert_invitation_code(&state.db, &invitation).unwrap();

    // 2. POST /v1/agents/register with invitation code + rich metadata
    let app = ApiServer::router(state.clone());
    let body = serde_json::json!({
        "name": "LifecycleBot",
        "invitation_code": "INV-lifecycle-001",
        "purpose": "End-to-end lifecycle testing",
        "agent_type": "automated",
        "capabilities": ["send", "receive"],
        "description": "A test bot for lifecycle integration testing"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/agents/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 3. Assert 201 with status: "pending"
    assert_eq!(response.status(), 201);
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "pending");
    let agent_id = resp_body["agent_id"].as_str().unwrap();
    assert!(!agent_id.is_empty());

    // 4. GET /v1/agents/register/{agent_id}/status -> pending
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/agents/register/{}/status", agent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "pending");
    assert_eq!(resp_body["agent_id"], agent_id);
}

// =========================================================================
// Steps 5-7: Approve agent, retrieve token, token gone on second retrieval
// =========================================================================

#[tokio::test]
async fn test_lifecycle_approve_and_retrieve_token() {
    let (_router, state) = create_test_app();

    // Register agent
    let invitation = create_test_invitation("INV-lifecycle-002", "Lifecycle test");
    insert_invitation_code(&state.db, &invitation).unwrap();

    let app = ApiServer::router(state.clone());
    let body = serde_json::json!({
        "name": "TokenBot",
        "invitation_code": "INV-lifecycle-002",
        "purpose": "Token retrieval test"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/agents/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp_body = body_json(response).await;
    let agent_id = resp_body["agent_id"].as_str().unwrap().to_string();

    // 5. Approve agent via agent_registry.approve()
    let _raw_token = state.agent_registry.approve(&agent_id).unwrap();

    // 6. GET status again -> active + token "anb_..."
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/agents/register/{}/status", agent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "active");
    let token = resp_body["token"].as_str().unwrap();
    assert!(
        token.starts_with("anb_"),
        "Token should start with anb_: {}",
        token
    );

    // 7. GET status third time -> token: null (already consumed)
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/agents/register/{}/status", agent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "active");
    assert!(
        resp_body.get("token").is_none() || resp_body["token"].is_null(),
        "Token should be null on second retrieval, got: {:?}",
        resp_body.get("token")
    );
}

// =========================================================================
// Steps 8-11: Send transaction, poll status, list transactions
// =========================================================================

#[tokio::test]
async fn test_lifecycle_send_and_poll_transaction() {
    let (_router, state) = create_test_app();

    // Register, approve, and get token with proper spending policy
    let (_agent_id, token) = common::register_agent_with_policy(
        &state,
        "INV-lifecycle-003",
        "SendBot",
        "100",   // per_tx_max
        "1000",  // daily_cap
        "5000",  // weekly_cap
        "20000", // monthly_cap
        "50",    // auto_approve_max
    )
    .await;

    // 8. POST /v1/send with Bearer token, amount 5.00
    let app = ApiServer::router(state.clone());
    let send_body = serde_json::json!({
        "to": "0xRecipient123",
        "amount": "5.00",
        "asset": "USDC",
        "description": "Lifecycle test payment"
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

    // 9. Assert 202 with status: "executing"
    assert_eq!(
        response.status(),
        202,
        "Send should return 202 Accepted"
    );
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "executing");
    let tx_id = resp_body["tx_id"].as_str().unwrap().to_string();
    assert!(!tx_id.is_empty());

    // 10. Wait briefly for the background execution to complete, then GET /v1/transactions/{tx_id}
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
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "confirmed");
    assert!(
        resp_body["chain_tx_hash"].is_string(),
        "Confirmed tx should have chain_tx_hash"
    );

    // 11. GET /v1/transactions?limit=10 -> tx appears
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            "/v1/transactions?limit=10",
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let resp_body = body_json(response).await;
    assert!(resp_body["total"].as_i64().unwrap() >= 1);
    let txs = resp_body["data"].as_array().unwrap();
    let found = txs.iter().any(|tx| tx["id"].as_str() == Some(&tx_id));
    assert!(found, "Transaction {} should appear in list", tx_id);
}

// =========================================================================
// Additional: Full lifecycle in a single test (register -> send -> confirm)
// =========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_lifecycle_full_happy_path() {
    let (_router, state) = create_test_app();

    // 1. Insert invitation code
    let invitation = create_test_invitation("INV-full-001", "Full lifecycle");
    insert_invitation_code(&state.db, &invitation).unwrap();

    // 2. Register
    let app = ApiServer::router(state.clone());
    let body = serde_json::json!({
        "name": "FullLifecycleBot",
        "invitation_code": "INV-full-001",
        "purpose": "Full lifecycle test",
        "agent_type": "automated"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/agents/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 201);
    let resp_body = body_json(response).await;
    let agent_id = resp_body["agent_id"].as_str().unwrap().to_string();
    assert_eq!(resp_body["status"], "pending");

    // 3. Verify pending via status endpoint
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/agents/register/{}/status", agent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "pending");

    // 4. Approve -- capture the raw token directly
    let token = state.agent_registry.approve(&agent_id).unwrap();
    assert!(token.starts_with("anb_"));

    // 5. Retrieve token via status endpoint -- verify it matches
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/agents/register/{}/status", agent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "active");
    let endpoint_token = resp_body["token"].as_str().unwrap().to_string();
    assert_eq!(token, endpoint_token, "Token from approve() should match status endpoint");

    // 6. Token consumed -- second call returns no token
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/agents/register/{}/status", agent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "active");
    assert!(
        resp_body.get("token").is_none() || resp_body["token"].is_null(),
        "Token should not be available after first retrieval"
    );

    // 7. Update spending policy so we can send
    {
        let conn = state.db.get_connection().unwrap();
        conn.execute(
            "UPDATE spending_policies SET per_tx_max = '100', daily_cap = '1000', weekly_cap = '5000', monthly_cap = '20000', auto_approve_max = '50', updated_at = ?1 WHERE agent_id = ?2",
            rusqlite::params![chrono::Utc::now().timestamp(), agent_id],
        ).unwrap();
    } // conn dropped here

    // 8. Send 5.00 USDC
    let app = ApiServer::router(state.clone());
    let send_body = serde_json::json!({
        "to": "0xRecipient",
        "amount": "5.00",
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
    assert_eq!(response.status(), 202);
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "executing");
    let tx_id = resp_body["tx_id"].as_str().unwrap().to_string();

    // 9. Wait for background execution
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 10. Poll tx -> confirmed
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
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "confirmed");

    // 11. List transactions -> tx appears
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(bearer_request(
            "GET",
            "/v1/transactions?limit=10",
            &token,
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let resp_body = body_json(response).await;
    assert!(resp_body["total"].as_i64().unwrap() >= 1);
}
