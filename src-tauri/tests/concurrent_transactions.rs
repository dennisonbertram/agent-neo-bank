//! Integration Test: Scenario 8 — Concurrent Transaction Safety
//!
//! Tests that concurrent transactions from multiple agents are properly
//! serialized by SQLite's BEGIN EXCLUSIVE locking, ensuring spending
//! ledgers remain accurate and global caps are not violated.
//!
//! IMPORTANT: These tests use file-based SQLite (setup_test_db_file)
//! because in-memory SQLite with pool_size=1 serializes all access
//! at the connection pool level, hiding real concurrency issues.
//! File-based SQLite with pool_size=4 allows multiple connections
//! and exposes true locking behavior.

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use http::Request;
use tower::ServiceExt;

use agent_neo_bank_lib::api::rate_limiter::RateLimiter;
use agent_neo_bank_lib::api::rest_server::{ApiServer, AppStateAxum};
use agent_neo_bank_lib::cli::executor::MockCliExecutor;
use agent_neo_bank_lib::config::AppConfig;
use agent_neo_bank_lib::core::agent_registry::AgentRegistry;
use agent_neo_bank_lib::core::auth_service::AuthService;
use agent_neo_bank_lib::core::tx_processor::TransactionProcessor;
use agent_neo_bank_lib::core::wallet_service::WalletService;
use agent_neo_bank_lib::db::models::GlobalPolicy;
use agent_neo_bank_lib::db::queries::{
    get_global_spending_for_period, upsert_global_policy,
};
use agent_neo_bank_lib::db::schema::Database;

use common::{register_agent_with_policy, body_json};

/// Create a file-based SQLite database for concurrency tests.
/// File-based DB has pool_size=4, enabling real concurrent connections.
/// Returns (db, _tmp_dir) - _tmp_dir must be kept alive for the DB file to persist.
fn setup_test_db_file() -> (Arc<Database>, tempfile::TempDir) {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = tmp_dir.path().join("test.db");
    let db = Database::new(db_path).expect("Failed to create file DB");
    db.run_migrations().expect("Failed to run migrations");
    (Arc::new(db), tmp_dir)
}

use rust_decimal_macros::dec;

/// Build a test app using a file-based database for real concurrency testing.
/// File-based SQLite has pool_size=4, enabling multiple concurrent connections.
fn create_test_app_with_file_db(
    db: Arc<Database>,
    config: AppConfig,
) -> (axum::Router, Arc<AppStateAxum>) {
    let cli: Arc<dyn agent_neo_bank_lib::cli::executor::CliExecutable> =
        Arc::new(MockCliExecutor::with_defaults());
    let auth_service = Arc::new(AuthService::new(
        cli.clone(),
        db.clone(),
        Duration::from_secs(300),
    ));
    let agent_registry = Arc::new(AgentRegistry::new(db.clone(), config.clone()));
    let (tx_processor, _rx) =
        TransactionProcessor::new(db.clone(), cli.clone(), dec!(10000), 16);
    let tx_processor = Arc::new(tx_processor);
    let wallet_service = Arc::new(WalletService::new(
        cli.clone(),
        db.clone(),
        Duration::from_secs(0),
    ));
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_requests_per_minute));

    let state = Arc::new(AppStateAxum {
        db,
        auth_service,
        agent_registry,
        tx_processor,
        wallet_service,
        rate_limiter,
        config,
    });

    let router = ApiServer::router(state.clone());
    (router, state)
}

/// Send a transaction through the Axum router, returning (status_code, response_body).
async fn send_via_router(
    state: &Arc<AppStateAxum>,
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
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/send")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_string(&send_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status().as_u16();
    let body = body_json(response).await;
    (status, body)
}

/// Helper to get the global daily period key (matches global_policy.rs format).
fn global_daily_period_key() -> String {
    let now = chrono::Utc::now();
    format!("daily:{}", now.format("%Y-%m-%d"))
}

// =========================================================================
// Test 1: Concurrent sends from two agents respect global daily cap
// =========================================================================
//
// Two agents each have individual daily_cap:20, global daily_cap:25.
// Concurrently send A=15 and B=15 (total 30 > global cap 25).
// The atomic reserve-then-execute pattern ensures at most one succeeds
// because the second reservation would exceed the global cap.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_sends_no_overspend() {
    let (db, _tmp_dir) = setup_test_db_file();
    let config = AppConfig::default_test();
    let (_router, state) = create_test_app_with_file_db(db, config);

    // Set global daily cap to 25 (15+15=30 > 25, so at most one should succeed)
    let global_policy = GlobalPolicy {
        id: "default".to_string(),
        daily_cap: "25".to_string(),
        weekly_cap: "0".to_string(),
        monthly_cap: "0".to_string(),
        min_reserve_balance: "0".to_string(),
        kill_switch_active: false,
        kill_switch_reason: String::new(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    upsert_global_policy(&state.db, &global_policy).unwrap();

    // Create Agent A: individual daily_cap 20, auto_approve_max 20
    let (_agent_a_id, token_a) = register_agent_with_policy(
        &state,
        "INV-conc-001",
        "ConcAgentA",
        "20", // per_tx_max
        "20", // daily_cap
        "5000",
        "20000",
        "20", // auto_approve_max
    )
    .await;

    // Create Agent B: individual daily_cap 20, auto_approve_max 20
    let (_agent_b_id, token_b) = register_agent_with_policy(
        &state,
        "INV-conc-002",
        "ConcAgentB",
        "20",
        "20",
        "5000",
        "20000",
        "20",
    )
    .await;

    // Send concurrently: A=15, B=15
    let state_a = state.clone();
    let token_a_clone = token_a.clone();
    let state_b = state.clone();
    let token_b_clone = token_b.clone();

    let (result_a, result_b) = tokio::join!(
        tokio::spawn(async move { send_via_router(&state_a, &token_a_clone, "15").await }),
        tokio::spawn(async move { send_via_router(&state_b, &token_b_clone, "15").await }),
    );

    let (status_a, _body_a) = result_a.unwrap();
    let (status_b, _body_b) = result_b.unwrap();

    // Exactly one should be accepted, one should be denied (global cap 25, each sends 15)
    let accepted_count = [status_a, status_b]
        .iter()
        .filter(|&&s| s == 202)
        .count();
    let denied_count = [status_a, status_b]
        .iter()
        .filter(|&&s| s == 403)
        .count();
    assert_eq!(
        accepted_count, 1,
        "Exactly one concurrent send should be accepted (global_cap=25). A={}, B={}",
        status_a, status_b
    );
    assert_eq!(
        denied_count, 1,
        "Exactly one concurrent send should be denied (global_cap=25). A={}, B={}",
        status_a, status_b
    );

    // Wait for background execution to complete and update ledgers
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify the global spending ledger reflects exactly one successful send
    let period = global_daily_period_key();
    let global_ledger = get_global_spending_for_period(&state.db, &period)
        .unwrap()
        .expect("Global spending ledger should exist after one successful send");

    let total: f64 = global_ledger.total.parse().unwrap();
    assert!(
        (total - 15.0).abs() < 0.01,
        "Global ledger total should be 15 (one send succeeded), got {}",
        total
    );
    assert_eq!(
        global_ledger.tx_count, 1,
        "Global ledger should record exactly 1 transaction"
    );
}

// =========================================================================
// Test 2: Concurrent sends both succeed when within all caps
// =========================================================================
//
// Two agents, individual caps 20 each, global cap 40.
// Concurrently send A=15, B=15 (total 30 < 40). Both should succeed.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_sends_both_succeed_within_caps() {
    let (db, _tmp_dir) = setup_test_db_file();
    let config = AppConfig::default_test();
    let (_router, state) = create_test_app_with_file_db(db, config);

    // Set global daily cap to 40 (plenty of room for both)
    let global_policy = GlobalPolicy {
        id: "default".to_string(),
        daily_cap: "40".to_string(),
        weekly_cap: "0".to_string(),
        monthly_cap: "0".to_string(),
        min_reserve_balance: "0".to_string(),
        kill_switch_active: false,
        kill_switch_reason: String::new(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    upsert_global_policy(&state.db, &global_policy).unwrap();

    // Create Agent A
    let (_agent_a_id, token_a) = register_agent_with_policy(
        &state,
        "INV-conc-003",
        "BothSucceedA",
        "20",
        "20",
        "5000",
        "20000",
        "20",
    )
    .await;

    // Create Agent B
    let (_agent_b_id, token_b) = register_agent_with_policy(
        &state,
        "INV-conc-004",
        "BothSucceedB",
        "20",
        "20",
        "5000",
        "20000",
        "20",
    )
    .await;

    // Send concurrently: A=15, B=15
    let state_a = state.clone();
    let token_a_clone = token_a.clone();
    let state_b = state.clone();
    let token_b_clone = token_b.clone();

    let (result_a, result_b) = tokio::join!(
        tokio::spawn(async move { send_via_router(&state_a, &token_a_clone, "15").await }),
        tokio::spawn(async move { send_via_router(&state_b, &token_b_clone, "15").await }),
    );

    let (status_a, body_a) = result_a.unwrap();
    let (status_b, body_b) = result_b.unwrap();

    // Both should succeed (both within individual and global caps)
    assert_eq!(
        status_a, 202,
        "Agent A send should succeed: {:?}",
        body_a
    );
    assert_eq!(
        status_b, 202,
        "Agent B send should succeed: {:?}",
        body_b
    );

    // Wait for background execution
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify global ledger totals
    let period = global_daily_period_key();
    let global_ledger = get_global_spending_for_period(&state.db, &period)
        .unwrap()
        .expect("Global spending ledger should exist after two successful sends");

    let total: f64 = global_ledger.total.parse().unwrap();
    assert!(
        (total - 30.0).abs() < 0.01,
        "Global ledger total should be 30 (15 + 15), got {}",
        total
    );
    assert_eq!(
        global_ledger.tx_count, 2,
        "Global ledger should record 2 transactions"
    );
}

// =========================================================================
// Test 3: Stress test — serialization correctness under load
// =========================================================================
//
// Single agent with daily_cap:20 sends 5 concurrent requests of 5.00 each.
// Total attempted: 25.00, cap: 20.00.
// Expected: exactly 4 succeed (total 20), 1 denied (would be 25).
//
// The atomic reserve-then-execute pattern ensures that policy check + ledger
// reservation happen in a single BEGIN EXCLUSIVE transaction, preventing
// TOCTOU overspend.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_sends_serialization_correctness() {
    let (db, _tmp_dir) = setup_test_db_file();
    let config = AppConfig::default_test();
    let (_router, state) = create_test_app_with_file_db(db, config);

    // No global policy (unlimited) so we only test per-agent caps
    // Agent with daily_cap:20, per_tx_max:10, auto_approve_max:10
    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-conc-005",
        "StressAgent",
        "10",  // per_tx_max
        "20",  // daily_cap
        "5000",
        "20000",
        "10",  // auto_approve_max
    )
    .await;

    // Spawn 5 concurrent sends of 5.00 each
    let mut handles = Vec::new();
    for i in 0..5 {
        let state_clone = state.clone();
        let token_clone = token.clone();
        handles.push(tokio::spawn(async move {
            let result = send_via_router(&state_clone, &token_clone, "5").await;
            (i, result)
        }));
    }

    // Collect all results
    let mut accepted = 0u32;
    let mut denied = 0u32;
    for handle in handles {
        let (idx, (status, body)) = handle.await.unwrap();
        match status {
            202 => {
                accepted += 1;
            }
            403 => {
                denied += 1;
            }
            other => {
                panic!(
                    "Unexpected status {} for request {}: {:?}",
                    other, idx, body
                );
            }
        }
    }

    // Wait for background execution to complete
    tokio::time::sleep(Duration::from_secs(3)).await;

    // With atomic reserve-then-execute, exactly 4 should be accepted and 1 denied
    // because daily_cap=20 and each send is 5.00 (4*5=20, 5*5=25 > 20)
    assert!(
        accepted <= 4,
        "At most 4 of 5 sends should be accepted (daily_cap:20, each 5.00). Got accepted={}, denied={}",
        accepted,
        denied,
    );
    assert!(
        accepted >= 1,
        "At least 1 send should be accepted. Got accepted={}, denied={}",
        accepted,
        denied,
    );

    // Verify the ledger total does not exceed the daily cap
    let period = agent_neo_bank_lib::core::spending_policy::daily_period_key(&chrono::Utc::now());
    let ledger =
        agent_neo_bank_lib::db::queries::get_spending_for_period(&state.db, &_agent_id, &period)
            .unwrap();

    if let Some(ledger) = ledger {
        let total: f64 = ledger.total.parse().unwrap();
        assert!(
            total <= 20.01,
            "Ledger total must not exceed daily cap of 20.00, got {}",
            total
        );
        let expected = accepted as f64 * 5.0;
        assert!(
            (total - expected).abs() < 0.01,
            "Ledger total should be {} ({}*5), got {}. No lost updates allowed.",
            expected,
            accepted,
            total
        );
        assert_eq!(
            ledger.tx_count, accepted as i64,
            "Ledger tx_count should match accepted count"
        );
    } else {
        panic!("Spending ledger should exist after successful sends");
    }

    // Verify no deadlocks occurred (the test completing is proof)
    // Verify total requests accounted for
    assert_eq!(
        accepted + denied,
        5,
        "All 5 requests should be accounted for"
    );
}

// =========================================================================
// Test 4: Strict cap enforcement — exactly 4 of 5 succeed
// =========================================================================
//
// Single agent, daily_cap=20, auto_approve_max=100 (so all auto-approved).
// Send 5 concurrent requests of 5.00 each.
// Assert: exactly 4 succeed, exactly 1 denied, ledger total == 20.00.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_sends_strict_cap_enforcement() {
    let (db, _tmp_dir) = setup_test_db_file();
    let config = AppConfig::default_test();
    let (_router, state) = create_test_app_with_file_db(db, config);

    // No global policy — only per-agent caps
    // daily_cap=20, per_tx_max=100, auto_approve_max=100 (all auto-approved)
    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-conc-006",
        "StrictCapAgent",
        "100",  // per_tx_max (high so no per-tx denial)
        "20",   // daily_cap
        "5000",
        "20000",
        "100",  // auto_approve_max (high so all auto-approved)
    )
    .await;

    // Spawn 5 concurrent sends of 5.00 each
    let mut handles = Vec::new();
    for i in 0..5 {
        let state_clone = state.clone();
        let token_clone = token.clone();
        handles.push(tokio::spawn(async move {
            let result = send_via_router(&state_clone, &token_clone, "5").await;
            (i, result)
        }));
    }

    // Collect all results
    let mut accepted = 0u32;
    let mut denied = 0u32;
    for handle in handles {
        let (idx, (status, body)) = handle.await.unwrap();
        match status {
            202 => {
                accepted += 1;
            }
            403 => {
                denied += 1;
            }
            other => {
                panic!(
                    "Unexpected status {} for request {}: {:?}",
                    other, idx, body
                );
            }
        }
    }

    // Wait for background execution to complete
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Strict enforcement: exactly 4 accepted, exactly 1 denied
    assert_eq!(
        accepted, 4,
        "Exactly 4 of 5 sends should be accepted (daily_cap:20, each 5.00). Got accepted={}, denied={}",
        accepted, denied,
    );
    assert_eq!(
        denied, 1,
        "Exactly 1 of 5 sends should be denied (daily_cap:20, each 5.00). Got accepted={}, denied={}",
        accepted, denied,
    );

    // Verify ledger total is exactly 20.00
    let period = agent_neo_bank_lib::core::spending_policy::daily_period_key(&chrono::Utc::now());
    let ledger =
        agent_neo_bank_lib::db::queries::get_spending_for_period(&state.db, &_agent_id, &period)
            .unwrap()
            .expect("Spending ledger should exist after successful sends");

    let total: f64 = ledger.total.parse().unwrap();
    assert!(
        (total - 20.0).abs() < 0.01,
        "Ledger total should be exactly 20.00, got {}",
        total
    );
    assert_eq!(
        ledger.tx_count, 4,
        "Ledger tx_count should be exactly 4"
    );
}
