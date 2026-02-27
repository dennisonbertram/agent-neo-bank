//! Integration Test: Scenario 7 — CLI Failure & Recovery
//!
//! Tests that CLI failures during background send execution:
//! - Mark the transaction as "failed"
//! - Do NOT update the spending ledger
//! - Allow retry with a subsequent send

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::body::Body;
use common::{
    bearer_request, body_json, create_test_app_with_db_config_and_cli, register_agent_with_policy,
    ServiceExt,
};

use agent_neo_bank_lib::api::rest_server::ApiServer;
use agent_neo_bank_lib::cli::commands::AwalCommand;
use agent_neo_bank_lib::cli::executor::{CliError, CliExecutable, CliOutput, MockCliExecutor};
use agent_neo_bank_lib::config::AppConfig;
use agent_neo_bank_lib::test_helpers::setup_test_db;

// ---------------------------------------------------------------------------
// Switchable CLI executor: can toggle send failures at runtime
// ---------------------------------------------------------------------------

struct SwitchableCliExecutor {
    /// When true, "send" commands return Err.
    send_should_fail: AtomicBool,
    /// Delegate for non-send commands (and successful sends).
    delegate: MockCliExecutor,
}

impl SwitchableCliExecutor {
    fn new(send_should_fail: bool) -> Self {
        Self {
            send_should_fail: AtomicBool::new(send_should_fail),
            delegate: MockCliExecutor::with_defaults(),
        }
    }

    fn set_send_fails(&self, fail: bool) {
        self.send_should_fail.store(fail, Ordering::SeqCst);
    }
}

#[async_trait]
impl CliExecutable for SwitchableCliExecutor {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError> {
        if matches!(cmd, AwalCommand::Send { .. })
            && self.send_should_fail.load(Ordering::SeqCst)
        {
            return Err(CliError::CommandFailed {
                stderr: "CLI error: send failed".to_string(),
                exit_code: Some(1),
            });
        }
        self.delegate.run(cmd).await
    }
}

// ---------------------------------------------------------------------------
// Helper: build app with a switchable CLI
// ---------------------------------------------------------------------------

fn create_failing_send_app(
    cli: Arc<SwitchableCliExecutor>,
) -> (
    axum::Router,
    Arc<agent_neo_bank_lib::api::rest_server::AppStateAxum>,
) {
    let db = setup_test_db();
    let config = AppConfig::default_test();
    create_test_app_with_db_config_and_cli(db, config, cli)
}

/// Poll a transaction until it reaches one of the expected terminal statuses.
async fn wait_for_tx_status(
    state: &Arc<agent_neo_bank_lib::api::rest_server::AppStateAxum>,
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

// =========================================================================
// Test 1: CLI failure marks transaction as "failed"
// =========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cli_failure_tx_status_is_failed() {
    let cli = Arc::new(SwitchableCliExecutor::new(true)); // send fails
    let (_router, state) = create_failing_send_app(cli.clone());

    // Register and approve agent with generous policy
    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-clifail-001",
        "CliFailBot1",
        "100",   // per_tx_max
        "1000",  // daily_cap
        "5000",  // weekly_cap
        "20000", // monthly_cap
        "50",    // auto_approve_max
    )
    .await;

    // Send 5.00 -> should get 202 (accepted for background execution)
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

    assert_eq!(response.status(), 202, "Send should return 202 Accepted");
    let resp_body = body_json(response).await;
    let tx_id = resp_body["tx_id"].as_str().unwrap().to_string();
    assert_eq!(resp_body["status"], "executing");

    // Poll until background execution fails (instead of sleeping)
    let resp_body = wait_for_tx_status(
        &state,
        &tx_id,
        &token,
        &["failed"],
        Duration::from_secs(10),
    )
    .await;

    assert_eq!(
        resp_body["status"], "failed",
        "Transaction should be marked as failed after CLI error"
    );
    assert!(
        resp_body["error_message"].is_string(),
        "Failed transaction should have an error_message"
    );
}

// =========================================================================
// Test 2: CLI failure does NOT update spending ledger
// =========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cli_failure_spending_ledger_not_updated() {
    let cli = Arc::new(SwitchableCliExecutor::new(true)); // send fails
    let (_router, state) = create_failing_send_app(cli.clone());

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-clifail-002",
        "CliFailBot2",
        "100",
        "1000",
        "5000",
        "20000",
        "50",
    )
    .await;

    // Send 5.00 -> 202 -> fails in background
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
    let fail_body = body_json(response).await;
    let fail_tx_id = fail_body["tx_id"].as_str().unwrap().to_string();

    // Poll until background execution fails (instead of sleeping)
    wait_for_tx_status(
        &state,
        &fail_tx_id,
        &token,
        &["failed"],
        Duration::from_secs(10),
    )
    .await;

    // Now switch CLI to succeed and send another 5.00
    // If the failed tx had incorrectly updated the ledger, this would count
    // against the daily spend. Since it didn't, this should pass policy checks.
    cli.set_send_fails(false);

    let app = ApiServer::router(state.clone());
    let send_body2 = serde_json::json!({
        "to": "0xRecipient",
        "amount": "5.00",
        "asset": "USDC",
        "description": "Second send after CLI recovery"
    });
    let response = app
        .oneshot(bearer_request(
            "POST",
            "/v1/send",
            &token,
            Body::from(serde_json::to_string(&send_body2).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        202,
        "Second send should succeed policy check (failed tx should not count)"
    );
    let resp_body = body_json(response).await;
    let tx_id2 = resp_body["tx_id"].as_str().unwrap().to_string();

    // Poll until confirmed (instead of sleeping)
    let resp_body = wait_for_tx_status(
        &state,
        &tx_id2,
        &token,
        &["confirmed"],
        Duration::from_secs(10),
    )
    .await;

    assert_eq!(
        resp_body["status"], "confirmed",
        "Second send should succeed after CLI recovery"
    );
}

// =========================================================================
// Test 3: CLI failure then retry succeeds
// =========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cli_failure_retry_succeeds() {
    let cli = Arc::new(SwitchableCliExecutor::new(true)); // send fails initially
    let (_router, state) = create_failing_send_app(cli.clone());

    let (_agent_id, token) = register_agent_with_policy(
        &state,
        "INV-clifail-003",
        "CliFailBot3",
        "100",
        "1000",
        "5000",
        "20000",
        "50",
    )
    .await;

    // First send: fails in background
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
    let tx_id1 = resp_body["tx_id"].as_str().unwrap().to_string();

    // Poll until background execution fails (instead of sleeping)
    let resp_body = wait_for_tx_status(
        &state,
        &tx_id1,
        &token,
        &["failed"],
        Duration::from_secs(10),
    )
    .await;
    assert_eq!(resp_body["status"], "failed");

    // Reconfigure CLI to succeed
    cli.set_send_fails(false);

    // Retry: send 5.00 again
    let app = ApiServer::router(state.clone());
    let send_body2 = serde_json::json!({
        "to": "0xRecipient",
        "amount": "5.00",
        "asset": "USDC"
    });
    let response = app
        .oneshot(bearer_request(
            "POST",
            "/v1/send",
            &token,
            Body::from(serde_json::to_string(&send_body2).unwrap()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    let resp_body = body_json(response).await;
    let tx_id2 = resp_body["tx_id"].as_str().unwrap().to_string();

    // Poll until confirmed (instead of sleeping)
    let resp_body = wait_for_tx_status(
        &state,
        &tx_id2,
        &token,
        &["confirmed"],
        Duration::from_secs(10),
    )
    .await;

    assert_eq!(
        resp_body["status"], "confirmed",
        "Retry send should succeed after CLI is fixed"
    );
    assert!(
        resp_body["chain_tx_hash"].is_string(),
        "Confirmed tx should have chain_tx_hash"
    );
}
