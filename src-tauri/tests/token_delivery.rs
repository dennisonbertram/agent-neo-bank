//! Integration Test: Scenario 6 — Token Delivery Expiry & Re-registration
//!
//! Tests token delivery TTL expiry, successful retrieval within the window,
//! and full re-registration flow after token expiry.

mod common;

use axum::body::Body;
use common::{bearer_request, body_json, create_test_app, register_approve_and_get_token, ServiceExt};
use http::Request;

use tally_agentic_wallet_lib::api::rest_server::ApiServer;

// =========================================================================
// Test 1: Token delivery expires after TTL
// =========================================================================

#[tokio::test]
async fn test_token_delivery_expires_after_ttl() {
    let (_router, state) = create_test_app();

    // Register and approve agent
    let (agent_id, _token) =
        register_approve_and_get_token(&state, "INV-td-expire-001", "ExpiryBot").await;

    // Manipulate token_delivery timestamp to simulate 6 minutes passing
    // (token TTL is 5 minutes, so 6 minutes means it's expired)
    {
        let conn = state.db.get_connection().unwrap();
        conn.execute(
            "UPDATE token_delivery SET created_at = created_at - 360, expires_at = expires_at - 360 WHERE agent_id = ?1",
            rusqlite::params![agent_id],
        )
        .unwrap();
    }

    // Poll agent status — token should be null/absent since it's expired
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
        "Token should be null/absent after TTL expiry, got: {:?}",
        resp_body.get("token")
    );
}

// =========================================================================
// Test 2: Token delivery succeeds within window
// =========================================================================

#[tokio::test]
async fn test_token_delivery_succeeds_within_window() {
    let (_router, state) = create_test_app();

    // Register and approve agent
    let (agent_id, expected_token) =
        register_approve_and_get_token(&state, "INV-td-window-001", "WindowBot").await;

    // Poll immediately — token should be present
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

    let token = resp_body["token"].as_str().expect("Token should be present within delivery window");
    assert!(
        token.starts_with("anb_"),
        "Token should start with anb_: {}",
        token
    );
    assert_eq!(
        token, expected_token,
        "Token from status endpoint should match the one returned by approve()"
    );

    // Second poll — token should be consumed (null)
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
        "Token should be null on second retrieval"
    );
}

// =========================================================================
// Test 3: Re-registration after token expiry — full flow
// =========================================================================

#[tokio::test]
async fn test_token_reregistration_after_expiry() {
    let (_router, state) = create_test_app();

    // --- Phase 1: Register, approve, expire token ---

    let (agent_id, _first_token) =
        register_approve_and_get_token(&state, "INV-td-rereg-001", "ReregBot").await;

    // Expire the token delivery by shifting timestamps back 6 minutes
    {
        let conn = state.db.get_connection().unwrap();
        conn.execute(
            "UPDATE token_delivery SET created_at = created_at - 360, expires_at = expires_at - 360 WHERE agent_id = ?1",
            rusqlite::params![agent_id],
        )
        .unwrap();
    }

    // Verify token is expired
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
    assert!(
        resp_body.get("token").is_none() || resp_body["token"].is_null(),
        "Token should be expired"
    );

    // --- Phase 2: Register a NEW agent (simulating re-registration) ---
    // The original agent's token is gone. A new invitation + registration is needed.

    let (new_agent_id, new_token) =
        register_approve_and_get_token(&state, "INV-td-rereg-002", "ReregBot2").await;

    // Verify new token is retrievable within window
    let app = ApiServer::router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/agents/register/{}/status", new_agent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "active");

    let retrieved_token = resp_body["token"]
        .as_str()
        .expect("New token should be available");
    assert_eq!(retrieved_token, new_token);

    // --- Phase 3: Use the new token to send a transaction ---

    // First set up spending policy for the new agent
    {
        let conn = state.db.get_connection().unwrap();
        conn.execute(
            "UPDATE spending_policies SET per_tx_max = '100', daily_cap = '1000', weekly_cap = '5000', monthly_cap = '20000', auto_approve_max = '50', updated_at = ?1 WHERE agent_id = ?2",
            rusqlite::params![chrono::Utc::now().timestamp(), new_agent_id],
        )
        .unwrap();
    }

    let app = ApiServer::router(state.clone());
    let send_body = serde_json::json!({
        "to": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab",
        "amount": "2.50",
        "asset": "USDC",
        "description": "Re-registration test payment"
    });
    let response = app
        .oneshot(bearer_request(
            "POST",
            "/v1/send",
            &new_token,
            Body::from(serde_json::to_string(&send_body).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        202,
        "Send with new token should return 202 Accepted"
    );
    let resp_body = body_json(response).await;
    assert_eq!(resp_body["status"], "executing");
    assert!(
        resp_body["tx_id"].as_str().is_some(),
        "Should receive a transaction ID"
    );
}
