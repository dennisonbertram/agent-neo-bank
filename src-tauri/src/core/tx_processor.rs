use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::cli::commands::AwalCommand;
use crate::cli::executor::CliExecutable;
use crate::core::spending_policy::{
    daily_period_key, monthly_period_key, weekly_period_key,
};
use crate::db::models::{
    ApprovalRequest, ApprovalRequestType, ApprovalStatus, Transaction, TxStatus, TxType,
};
use crate::db::queries::{
    check_policy_and_reserve_atomic, get_transaction, insert_approval_request,
    insert_transaction, rollback_reservation, update_transaction_status, AtomicPolicyResult,
};
use crate::db::schema::Database;
use crate::error::AppError;

// -------------------------------------------------------------------------
// Types
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: Decimal,
    pub asset: Option<String>,
    pub description: Option<String>,
    pub memo: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionResult {
    Accepted { tx_id: String, status: String },
    Denied { tx_id: String, reason: String },
}

#[derive(Debug, Clone)]
pub enum TxEvent {
    TransactionConfirmed(String),
    TransactionDenied(String),
    TransactionFailed(String),
}

// -------------------------------------------------------------------------
// TransactionProcessor
// -------------------------------------------------------------------------

pub struct TransactionProcessor {
    db: Arc<Database>,
    cli: Arc<dyn CliExecutable>,
    current_balance: Arc<tokio::sync::RwLock<Decimal>>,
    event_tx: broadcast::Sender<TxEvent>,
}

impl TransactionProcessor {
    pub fn new(
        db: Arc<Database>,
        cli: Arc<dyn CliExecutable>,
        current_balance: Decimal,
        event_channel_capacity: usize,
    ) -> (Self, broadcast::Receiver<TxEvent>) {
        let (event_tx, event_rx) = broadcast::channel(event_channel_capacity);

        let processor = Self {
            db,
            cli,
            current_balance: Arc::new(tokio::sync::RwLock::new(current_balance)),
            event_tx,
        };

        (processor, event_rx)
    }

    /// Subscribe to transaction events.
    pub fn subscribe(&self) -> broadcast::Receiver<TxEvent> {
        self.event_tx.subscribe()
    }

    /// Process a send request. Returns immediately with Accepted or Denied.
    /// For auto-approved transactions, spawns background execution.
    ///
    /// Uses atomic reserve-then-execute pattern: policy check + ledger reservation
    /// happen in a single BEGIN EXCLUSIVE transaction to prevent TOCTOU race conditions.
    pub async fn process_send(
        &self,
        agent_id: &str,
        request: SendRequest,
    ) -> Result<TransactionResult, AppError> {
        let now = Utc::now();
        let tx_id = uuid::Uuid::new_v4().to_string();
        let asset = request.asset.clone().unwrap_or_else(|| "USDC".to_string());
        let period_daily = daily_period_key(&now);
        let period_weekly = weekly_period_key(&now);
        let period_monthly = monthly_period_key(&now);

        // 1. Atomic policy check + reservation (eliminates TOCTOU)
        let balance = *self.current_balance.read().await;
        let atomic_result = check_policy_and_reserve_atomic(
            &self.db,
            agent_id,
            &request.amount.to_string(),
            &request.to,
            &balance.to_string(),
            &period_daily,
            &period_weekly,
            &period_monthly,
            now.timestamp(),
        )?;

        match atomic_result {
            AtomicPolicyResult::Denied { reason } => {
                let tx = self.build_transaction(
                    &tx_id,
                    agent_id,
                    &request,
                    &asset,
                    TxStatus::Denied,
                    &period_daily,
                    &period_weekly,
                    &period_monthly,
                    now.timestamp(),
                );
                insert_transaction(&self.db, &tx)?;
                let _ = self.event_tx.send(TxEvent::TransactionDenied(tx_id.clone()));
                Ok(TransactionResult::Denied { tx_id, reason })
            }
            AtomicPolicyResult::AutoApproved => {
                // Ledger already reserved — insert tx and spawn execution
                let tx = self.build_transaction(
                    &tx_id,
                    agent_id,
                    &request,
                    &asset,
                    TxStatus::Executing,
                    &period_daily,
                    &period_weekly,
                    &period_monthly,
                    now.timestamp(),
                );
                insert_transaction(&self.db, &tx)?;

                // Spawn background execution
                let db = self.db.clone();
                let cli = self.cli.clone();
                let event_tx = self.event_tx.clone();
                let tx_id_clone = tx_id.clone();
                let to = request.to.clone();
                let amount = request.amount;
                let asset_clone = asset.clone();
                let webhook_url = request.webhook_url.clone();
                let agent_id_owned = agent_id.to_string();
                let pd = period_daily.clone();
                let pw = period_weekly.clone();
                let pm = period_monthly.clone();
                let balance = self.current_balance.clone();

                tokio::spawn(async move {
                    Self::execute_send(
                        db,
                        cli,
                        event_tx,
                        tx_id_clone,
                        to,
                        amount,
                        asset_clone,
                        webhook_url,
                        agent_id_owned,
                        pd,
                        pw,
                        pm,
                        balance,
                    )
                    .await;
                });

                Ok(TransactionResult::Accepted {
                    tx_id,
                    status: "executing".to_string(),
                })
            }
            AtomicPolicyResult::RequiresApproval { .. } => {
                // Ledger already reserved — insert tx and create approval request
                let tx = self.build_transaction(
                    &tx_id,
                    agent_id,
                    &request,
                    &asset,
                    TxStatus::AwaitingApproval,
                    &period_daily,
                    &period_weekly,
                    &period_monthly,
                    now.timestamp(),
                );
                insert_transaction(&self.db, &tx)?;

                // Create approval request
                let approval = ApprovalRequest {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: agent_id.to_string(),
                    request_type: ApprovalRequestType::Transaction,
                    payload: serde_json::json!({
                        "tx_id": tx_id,
                        "to": request.to,
                        "amount": request.amount.to_string(),
                        "asset": asset,
                    })
                    .to_string(),
                    status: ApprovalStatus::Pending,
                    tx_id: Some(tx_id.clone()),
                    expires_at: now.timestamp() + 86400, // 24 hours
                    created_at: now.timestamp(),
                    resolved_at: None,
                    resolved_by: None,
                };
                insert_approval_request(&self.db, &approval)?;

                Ok(TransactionResult::Accepted {
                    tx_id,
                    status: "awaiting_approval".to_string(),
                })
            }
        }
    }

    /// Background execution of a send transaction.
    ///
    /// The spending ledger was already reserved by check_policy_and_reserve_atomic().
    /// On CLI success: update tx status to Confirmed (ledger already reserved).
    /// On CLI failure: rollback the reservation and mark tx as Failed.
    async fn execute_send(
        db: Arc<Database>,
        cli: Arc<dyn CliExecutable>,
        event_tx: broadcast::Sender<TxEvent>,
        tx_id: String,
        to: String,
        amount: Decimal,
        asset: String,
        webhook_url: Option<String>,
        agent_id: String,
        period_daily: String,
        period_weekly: String,
        period_monthly: String,
        current_balance: Arc<tokio::sync::RwLock<Decimal>>,
    ) {
        let cli_result = cli
            .run(AwalCommand::Send {
                to: to.clone(),
                amount,
                asset: asset.clone(),
            })
            .await;

        let now = Utc::now().timestamp();

        match cli_result {
            Ok(output) => {
                let chain_tx_hash = output
                    .data
                    .get("tx_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Ledger already reserved — just update tx status + chain hash
                let update_result = update_transaction_status(
                    &db,
                    &tx_id,
                    &TxStatus::Confirmed,
                    Some(&chain_tx_hash),
                    None,
                    now,
                );

                match update_result {
                    Ok(()) => {
                        // Update cached balance after successful send
                        let mut bal = current_balance.write().await;
                        *bal -= amount;

                        let _ = event_tx.send(TxEvent::TransactionConfirmed(tx_id.clone()));
                    }
                    Err(e) => {
                        tracing::error!(
                            tx_id = %tx_id,
                            error = %e,
                            "Transaction status update failed after CLI success"
                        );
                        // Status update failed — rollback reservation and mark failed
                        if let Err(rb_err) = rollback_reservation(
                            &db,
                            &agent_id,
                            &amount.to_string(),
                            &period_daily,
                            &period_weekly,
                            &period_monthly,
                            now,
                        ) {
                            tracing::error!(
                                tx_id = %tx_id,
                                error = %rb_err,
                                "CRITICAL: rollback_reservation failed — spending ledger is permanently overstated"
                            );
                        }
                        let _ = update_transaction_status(
                            &db,
                            &tx_id,
                            &TxStatus::Failed,
                            None,
                            Some("Transaction status update failed"),
                            now,
                        );
                        let _ = event_tx.send(TxEvent::TransactionFailed(tx_id.clone()));
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    tx_id = %tx_id,
                    error = %e,
                    "CLI execution failed for transaction"
                );
                // CLI failed — rollback the reservation and mark tx as failed
                if let Err(rb_err) = rollback_reservation(
                    &db,
                    &agent_id,
                    &amount.to_string(),
                    &period_daily,
                    &period_weekly,
                    &period_monthly,
                    now,
                ) {
                    tracing::error!(
                        tx_id = %tx_id,
                        error = %rb_err,
                        "CRITICAL: rollback_reservation failed — spending ledger is permanently overstated"
                    );
                }
                if let Err(status_err) = update_transaction_status(
                    &db,
                    &tx_id,
                    &TxStatus::Failed,
                    None,
                    Some(&e.to_string()),
                    now,
                ) {
                    tracing::error!(
                        tx_id = %tx_id,
                        error = %status_err,
                        "Failed to update transaction status to Failed"
                    );
                }
                let _ = event_tx.send(TxEvent::TransactionFailed(tx_id.clone()));
            }
        }

        // Webhook callback with retry (up to 3 attempts, 1s delay, 5s timeout)
        if let Some(url) = webhook_url {
            let status = match get_transaction(&db, &tx_id) {
                Ok(tx) => tx.status.to_string(),
                Err(_) => "unknown".to_string(),
            };
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            for attempt in 1..=3u32 {
                let result = client
                    .post(&url)
                    .json(&serde_json::json!({
                        "tx_id": tx_id,
                        "status": status,
                    }))
                    .send()
                    .await;

                match result {
                    Ok(resp) if resp.status().is_success() => break,
                    Ok(resp) => {
                        tracing::warn!(
                            tx_id = %tx_id,
                            attempt = attempt,
                            status = %resp.status(),
                            url = %url,
                            "Webhook delivery received non-success status"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            tx_id = %tx_id,
                            attempt = attempt,
                            error = %e,
                            url = %url,
                            "Webhook delivery failed"
                        );
                    }
                }

                if attempt < 3 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    fn build_transaction(
        &self,
        tx_id: &str,
        agent_id: &str,
        request: &SendRequest,
        asset: &str,
        status: TxStatus,
        period_daily: &str,
        period_weekly: &str,
        period_monthly: &str,
        now: i64,
    ) -> Transaction {
        Transaction {
            id: tx_id.to_string(),
            agent_id: Some(agent_id.to_string()),
            tx_type: TxType::Send,
            amount: request.amount.to_string(),
            asset: asset.to_string(),
            recipient: Some(request.to.clone()),
            sender: None,
            chain_tx_hash: None,
            status,
            category: "agent_send".to_string(),
            memo: request.memo.clone().unwrap_or_default(),
            description: request.description.clone().unwrap_or_default(),
            service_name: String::new(),
            service_url: String::new(),
            reason: String::new(),
            webhook_url: request.webhook_url.clone(),
            error_message: None,
            period_daily: period_daily.to_string(),
            period_weekly: period_weekly.to_string(),
            period_monthly: period_monthly.to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::executor::{CliError, CliOutput, MockCliExecutor};
    use crate::db::models::AgentStatus;
    use crate::db::queries::{
        get_spending_for_period, get_transaction, insert_agent, insert_spending_policy,
    };
    use crate::test_helpers::{create_test_agent, create_test_spending_policy, setup_test_db};
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    /// A CLI executor that always fails.
    struct FailingCliExecutor;

    #[async_trait]
    impl CliExecutable for FailingCliExecutor {
        async fn run(&self, _cmd: AwalCommand) -> Result<CliOutput, CliError> {
            Err(CliError::CommandFailed {
                stderr: "CLI execution failed".to_string(),
                exit_code: Some(1),
            })
        }
    }

    /// A CLI executor that takes a configurable delay before responding.
    struct SlowCliExecutor {
        delay: Duration,
    }

    #[async_trait]
    impl CliExecutable for SlowCliExecutor {
        async fn run(&self, _cmd: AwalCommand) -> Result<CliOutput, CliError> {
            tokio::time::sleep(self.delay).await;
            Ok(CliOutput {
                success: true,
                data: serde_json::json!({"tx_hash": "0xslow_hash"}),
                raw: r#"{"tx_hash": "0xslow_hash"}"#.to_string(),
                stderr: String::new(),
            })
        }
    }

    /// Helper: set up a processor with a mock CLI, agent with spending policy.
    /// Returns (processor, event_rx, agent_id).
    fn setup_processor(
        per_tx_max: &str,
        daily_cap: &str,
        weekly_cap: &str,
        monthly_cap: &str,
        auto_approve_max: &str,
        balance: Decimal,
        cli: Arc<dyn CliExecutable>,
    ) -> (TransactionProcessor, broadcast::Receiver<TxEvent>, String) {
        let db = setup_test_db();
        let agent = create_test_agent("TxTestBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(
            &agent.id,
            per_tx_max,
            daily_cap,
            weekly_cap,
            monthly_cap,
            auto_approve_max,
        );
        insert_spending_policy(&db, &policy).unwrap();

        let (processor, event_rx) = TransactionProcessor::new(db, cli, balance, 16);
        (processor, event_rx, agent.id)
    }

    fn make_send_request(to: &str, amount: Decimal) -> SendRequest {
        SendRequest {
            to: to.to_string(),
            amount,
            asset: None,
            description: None,
            memo: None,
            webhook_url: None,
        }
    }

    // ---------------------------------------------------------------
    // Test 1: Successful send returns Accepted with status "executing"
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_successful_send_returns_202() {
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let (processor, _rx, agent_id) =
            setup_processor("100", "1000", "5000", "20000", "50", dec!(10000), mock);

        let request = make_send_request("0xRecipient", dec!(10));
        let result = processor.process_send(&agent_id, request).await.unwrap();

        match result {
            TransactionResult::Accepted { status, .. } => {
                assert_eq!(status, "executing");
            }
            other => panic!("Expected Accepted, got {:?}", other),
        }
    }

    // ---------------------------------------------------------------
    // Test 2: Denied when per_tx_max exceeded
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_denied_exceeds_per_tx_max() {
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let (processor, _rx, agent_id) =
            setup_processor("25", "1000", "5000", "20000", "10", dec!(10000), mock);

        let request = make_send_request("0xRecipient", dec!(30));
        let result = processor.process_send(&agent_id, request).await.unwrap();

        match result {
            TransactionResult::Denied { reason, .. } => {
                assert!(
                    reason.contains("per-tx limit"),
                    "Expected per-tx limit in reason: {}",
                    reason
                );
            }
            other => panic!("Expected Denied, got {:?}", other),
        }
    }

    // ---------------------------------------------------------------
    // Test 3: Requires approval above auto_approve_max
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_requires_approval_above_auto_approve() {
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let (processor, _rx, agent_id) =
            setup_processor("100", "1000", "5000", "20000", "10", dec!(10000), mock);

        let request = make_send_request("0xRecipient", dec!(15));
        let result = processor.process_send(&agent_id, request).await.unwrap();

        match result {
            TransactionResult::Accepted { status, .. } => {
                assert_eq!(status, "awaiting_approval");
            }
            other => panic!("Expected Accepted with awaiting_approval, got {:?}", other),
        }
    }

    // ---------------------------------------------------------------
    // Test 4: Period keys set at creation time
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_period_keys_set_at_creation_time() {
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let db = setup_test_db();
        let agent = create_test_agent("PeriodKeyBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&db, &policy).unwrap();

        let (processor, _rx) =
            TransactionProcessor::new(db.clone(), Arc::new(MockCliExecutor::with_defaults()), dec!(10000), 16);

        let request = make_send_request("0xRecipient", dec!(5));
        let result = processor.process_send(&agent.id, request).await.unwrap();

        let tx_id = match result {
            TransactionResult::Accepted { tx_id, .. } => tx_id,
            TransactionResult::Denied { tx_id, .. } => tx_id,
        };

        let tx = get_transaction(&db, &tx_id).unwrap();
        let now = Utc::now();

        assert!(
            tx.period_daily.starts_with("daily:"),
            "Expected daily period key, got: {}",
            tx.period_daily
        );
        assert!(
            tx.period_weekly.starts_with("weekly:"),
            "Expected weekly period key, got: {}",
            tx.period_weekly
        );
        assert!(
            tx.period_monthly.starts_with("monthly:"),
            "Expected monthly period key, got: {}",
            tx.period_monthly
        );

        // Verify they match the current date
        assert_eq!(tx.period_daily, daily_period_key(&now));
        assert_eq!(tx.period_weekly, weekly_period_key(&now));
        assert_eq!(tx.period_monthly, monthly_period_key(&now));
    }

    // ---------------------------------------------------------------
    // Test 5: CLI failure mid-transaction marks tx failed, ledger NOT updated
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_cli_failure_mid_transaction_marks_failed() {
        let failing_cli = Arc::new(FailingCliExecutor);
        let db = setup_test_db();
        let agent = create_test_agent("FailBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&db, &policy).unwrap();

        let (processor, mut rx) =
            TransactionProcessor::new(db.clone(), failing_cli, dec!(10000), 16);

        let request = make_send_request("0xRecipient", dec!(5));
        let result = processor.process_send(&agent.id, request).await.unwrap();

        let tx_id = match result {
            TransactionResult::Accepted { tx_id, .. } => tx_id,
            other => panic!("Expected Accepted, got {:?}", other),
        };

        // Wait for the background task to complete
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        match event {
            TxEvent::TransactionFailed(id) => assert_eq!(id, tx_id),
            other => panic!("Expected TransactionFailed, got {:?}", other),
        }

        // Verify tx is marked failed
        let tx = get_transaction(&db, &tx_id).unwrap();
        assert_eq!(tx.status, TxStatus::Failed);
        assert!(tx.error_message.is_some());

        // Verify spending ledger is effectively zero (reservation was rolled back)
        let now = Utc::now();
        let daily_key = daily_period_key(&now);
        let ledger = get_spending_for_period(&db, &agent.id, &daily_key).unwrap();
        match ledger {
            None => {} // No ledger entry — OK
            Some(l) => {
                let total: f64 = l.total.parse().unwrap_or(0.0);
                assert!(
                    total.abs() < 0.01,
                    "Spending ledger total should be 0 after rollback, got {}",
                    total
                );
            }
        }
    }

    // ---------------------------------------------------------------
    // Test 6: Ledger update atomicity
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_ledger_update_atomicity() {
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let db = setup_test_db();
        let agent = create_test_agent("AtomicBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&db, &policy).unwrap();

        let (processor, mut rx) =
            TransactionProcessor::new(db.clone(), mock, dec!(10000), 16);

        let request = make_send_request("0xRecipient", dec!(25));
        let result = processor.process_send(&agent.id, request).await.unwrap();

        let tx_id = match result {
            TransactionResult::Accepted { tx_id, .. } => tx_id,
            other => panic!("Expected Accepted, got {:?}", other),
        };

        // Wait for background task
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        match event {
            TxEvent::TransactionConfirmed(id) => assert_eq!(id, tx_id),
            other => panic!("Expected TransactionConfirmed, got {:?}", other),
        }

        // Verify tx is confirmed with hash
        let tx = get_transaction(&db, &tx_id).unwrap();
        assert_eq!(tx.status, TxStatus::Confirmed);
        assert!(tx.chain_tx_hash.is_some());

        // Verify spending ledger IS updated
        let now = Utc::now();
        let daily_key = daily_period_key(&now);
        let ledger = get_spending_for_period(&db, &agent.id, &daily_key).unwrap();
        assert!(ledger.is_some(), "Spending ledger should be updated after successful send");
        let ledger = ledger.unwrap();
        assert_eq!(ledger.total, "25");
    }

    // ---------------------------------------------------------------
    // Test 7: Webhook callback on success
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_webhook_callback_on_success() {
        // For unit tests, we verify the webhook_url is set on the tx record
        // and that the transaction completes successfully. Full HTTP webhook
        // testing is done in integration tests.
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let db = setup_test_db();
        let agent = create_test_agent("WebhookBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&db, &policy).unwrap();

        let (processor, mut rx) =
            TransactionProcessor::new(db.clone(), mock, dec!(10000), 16);

        let request = SendRequest {
            to: "0xRecipient".to_string(),
            amount: dec!(10),
            asset: None,
            description: None,
            memo: None,
            webhook_url: Some("https://example.com/webhook".to_string()),
        };

        let result = processor.process_send(&agent.id, request).await.unwrap();
        let tx_id = match result {
            TransactionResult::Accepted { tx_id, .. } => tx_id,
            other => panic!("Expected Accepted, got {:?}", other),
        };

        // Wait for background task
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        match event {
            TxEvent::TransactionConfirmed(id) => assert_eq!(id, tx_id),
            other => panic!("Expected TransactionConfirmed, got {:?}", other),
        }

        // Verify webhook_url is stored on the transaction
        let tx = get_transaction(&db, &tx_id).unwrap();
        assert_eq!(
            tx.webhook_url,
            Some("https://example.com/webhook".to_string())
        );
        assert_eq!(tx.status, TxStatus::Confirmed);
    }

    // ---------------------------------------------------------------
    // Test 8: Webhook callback on failure
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_webhook_callback_on_failure() {
        let failing_cli = Arc::new(FailingCliExecutor);
        let db = setup_test_db();
        let agent = create_test_agent("WebhookFailBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&db, &policy).unwrap();

        let (processor, mut rx) =
            TransactionProcessor::new(db.clone(), failing_cli, dec!(10000), 16);

        let request = SendRequest {
            to: "0xRecipient".to_string(),
            amount: dec!(10),
            asset: None,
            description: None,
            memo: None,
            webhook_url: Some("https://example.com/webhook-fail".to_string()),
        };

        let result = processor.process_send(&agent.id, request).await.unwrap();
        let tx_id = match result {
            TransactionResult::Accepted { tx_id, .. } => tx_id,
            other => panic!("Expected Accepted, got {:?}", other),
        };

        // Wait for background task
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        match event {
            TxEvent::TransactionFailed(id) => assert_eq!(id, tx_id),
            other => panic!("Expected TransactionFailed, got {:?}", other),
        }

        // Verify webhook_url is stored and tx is failed
        let tx = get_transaction(&db, &tx_id).unwrap();
        assert_eq!(
            tx.webhook_url,
            Some("https://example.com/webhook-fail".to_string())
        );
        assert_eq!(tx.status, TxStatus::Failed);
    }

    // ---------------------------------------------------------------
    // Test 9: Rollback reservation restores ledger to zero
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_rollback_reservation_restores_ledger() {
        // This test verifies that rollback_reservation properly decrements
        // the spending ledger entries that were reserved.
        use crate::db::queries::{
            check_policy_and_reserve_atomic, rollback_reservation,
        };

        let db = setup_test_db();
        let agent = create_test_agent("RollbackBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&db, &policy).unwrap();

        let now = Utc::now();
        let daily = daily_period_key(&now);
        let weekly = weekly_period_key(&now);
        let monthly = monthly_period_key(&now);

        // Reserve via atomic policy check
        let result = check_policy_and_reserve_atomic(
            &db,
            &agent.id,
            "10",
            "0xRecipient",
            "10000",
            &daily,
            &weekly,
            &monthly,
            now.timestamp(),
        )
        .unwrap();
        assert_eq!(result, crate::db::queries::AtomicPolicyResult::AutoApproved);

        // Verify ledger was reserved
        let ledger = get_spending_for_period(&db, &agent.id, &daily)
            .unwrap()
            .unwrap();
        assert_eq!(ledger.total, "10");

        // Rollback the reservation
        rollback_reservation(
            &db,
            &agent.id,
            "10",
            &daily,
            &weekly,
            &monthly,
            now.timestamp(),
        )
        .unwrap();

        // Verify ledger is back to zero
        let ledger = get_spending_for_period(&db, &agent.id, &daily)
            .unwrap()
            .unwrap();
        let total: f64 = ledger.total.parse().unwrap();
        assert!(
            total.abs() < 0.01,
            "Ledger total should be 0 after rollback, got {}",
            total
        );
    }

    // ---------------------------------------------------------------
    // Test 10: Async 202 response is immediate
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_async_202_response_immediate() {
        // Use a slow CLI to verify process_send returns before CLI completes
        let slow_cli = Arc::new(SlowCliExecutor {
            delay: Duration::from_secs(5),
        });
        let db = setup_test_db();
        let agent = create_test_agent("AsyncBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();
        let policy = create_test_spending_policy(&agent.id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(&db, &policy).unwrap();

        let (processor, _rx) =
            TransactionProcessor::new(db.clone(), slow_cli, dec!(10000), 16);

        let request = make_send_request("0xRecipient", dec!(5));

        // process_send should return well under 1 second, not wait for the 5s CLI
        let start = std::time::Instant::now();
        let result = processor.process_send(&agent.id, request).await.unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(1),
            "process_send should return immediately, took {:?}",
            elapsed
        );

        match result {
            TransactionResult::Accepted { status, .. } => {
                assert_eq!(status, "executing");
            }
            other => panic!("Expected Accepted, got {:?}", other),
        }
    }

    // ---------------------------------------------------------------
    // Test 11: Event emitted on confirmation
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_event_emitted_on_confirmation() {
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let (processor, mut rx, agent_id) =
            setup_processor("100", "1000", "5000", "20000", "50", dec!(10000), mock);

        let request = make_send_request("0xRecipient", dec!(5));
        let result = processor.process_send(&agent_id, request).await.unwrap();

        let tx_id = match result {
            TransactionResult::Accepted { tx_id, .. } => tx_id,
            other => panic!("Expected Accepted, got {:?}", other),
        };

        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        match event {
            TxEvent::TransactionConfirmed(id) => assert_eq!(id, tx_id),
            other => panic!("Expected TransactionConfirmed, got {:?}", other),
        }
    }

    // ---------------------------------------------------------------
    // Test 12: Event emitted on denial
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_tx_processor_event_emitted_on_denial() {
        let mock = Arc::new(MockCliExecutor::with_defaults());
        let (processor, mut rx, agent_id) =
            setup_processor("25", "1000", "5000", "20000", "10", dec!(10000), mock);

        // Subscribe before process_send
        let mut sub = processor.subscribe();

        let request = make_send_request("0xRecipient", dec!(30));
        let result = processor.process_send(&agent_id, request).await.unwrap();

        let tx_id = match result {
            TransactionResult::Denied { tx_id, .. } => tx_id,
            other => panic!("Expected Denied, got {:?}", other),
        };

        // Check via subscriber
        let event = tokio::time::timeout(Duration::from_secs(1), sub.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        match event {
            TxEvent::TransactionDenied(id) => assert_eq!(id, tx_id),
            other => panic!("Expected TransactionDenied, got {:?}", other),
        }
    }
}
