//! Transport-agnostic MCP request router.
//!
//! `McpRouter` holds references to the shared database and dispatches
//! tool calls to the correct handler. Both the stdio-based `McpServer`
//! and the future HTTP-based MCP server use this router, avoiding code
//! duplication.

use std::sync::Arc;

use serde_json::Value;

use crate::cli::commands::AwalCommand;
use crate::cli::executor::CliExecutable;
use crate::core::spending_policy::{daily_period_key, monthly_period_key, weekly_period_key};
use crate::db::models::*;
use crate::db::queries;
use crate::db::queries::AtomicPolicyResult;
use crate::db::schema::Database;
use crate::error::AppError;

use super::mcp_tools::{get_tool_definitions, McpTool};

/// The MCP protocol version supported by this server.
/// Used by both the stdio (`McpServer`) and HTTP (`McpHttpServer`) transports.
pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

/// Transport-agnostic router that handles MCP tool calls.
///
/// The router is bound to a specific agent via `agent_id` and enforces
/// per-agent isolation for all queries and policy checks.
pub struct McpRouter {
    db: Arc<Database>,
    agent_id: String,
    cli: Option<Arc<dyn CliExecutable>>,
}

impl McpRouter {
    /// Create a new router bound to the given agent.
    ///
    /// The caller is responsible for authenticating the agent beforehand.
    pub fn new(db: Arc<Database>, agent_id: String) -> Self {
        Self { db, agent_id, cli: None }
    }

    /// Create a new router with a CLI executor for real awal calls.
    pub fn new_with_cli(db: Arc<Database>, agent_id: String, cli: Arc<dyn CliExecutable>) -> Self {
        Self { db, agent_id, cli: Some(cli) }
    }

    /// Execute an awal CLI command. Returns an error if no CLI executor is configured.
    ///
    /// When a Tokio runtime handle is available, uses `handle.block_on()` on a
    /// scoped thread so we reuse the existing runtime's I/O driver without
    /// creating a brand-new runtime per call. When no runtime exists (e.g. in
    /// sync unit tests), creates a lightweight current-thread runtime.
    ///
    /// **Why the extra thread is necessary**: Even though `handle_tools_call` in
    /// `mcp_http_server.rs` dispatches via `spawn_blocking`, `block_on()` still
    /// panics on a blocking-pool thread because that thread is managed by the
    /// same Tokio runtime. The scoped-thread pattern is the canonical workaround:
    /// it creates a short-lived OS thread that is *not* part of the runtime's
    /// thread pool, so `handle.block_on()` succeeds. The scoped thread is
    /// joined immediately, so the lifetime cost is minimal.
    fn run_cli(&self, cmd: AwalCommand) -> Result<crate::cli::executor::CliOutput, AppError> {
        let cli = self.cli.as_ref().ok_or_else(|| {
            AppError::Internal("CLI executor not configured".to_string())
        })?;
        let cli = cli.clone();
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // A runtime exists. We may be on a runtime thread (tokio::test)
                // or on a spawn_blocking thread. Use a scoped thread so we can
                // safely call handle.block_on without "cannot block from within
                // a runtime" panics, while still reusing the existing runtime's
                // I/O reactor instead of creating a new one.
                std::thread::scope(|s| {
                    s.spawn(|| {
                        handle
                            .block_on(cli.run(cmd))
                            .map_err(|e| {
                                eprintln!("CLI error (sanitized): {}", e);
                                AppError::CliError("Wallet operation failed. The wallet owner may need to re-authenticate.".into())
                            })
                    })
                    .join()
                    .unwrap_or_else(|_| Err(AppError::Internal("CLI thread panicked".to_string())))
                })
            }
            Err(_) => {
                // No runtime active — create a lightweight one
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| AppError::Internal(format!("Runtime error: {}", e)))?;
                rt.block_on(cli.run(cmd))
                    .map_err(|e| {
                        eprintln!("CLI error (sanitized): {}", e);
                        AppError::CliError("Wallet operation failed. The wallet owner may need to re-authenticate.".into())
                    })
            }
        }
    }

    /// Extract a transaction hash from CLI output, trying multiple key names.
    ///
    /// Different CLI versions or backends may return the hash under different
    /// keys: `tx_hash`, `transaction_hash`, `hash`, or nested under
    /// `transaction.hash`. This helper tries all known variants.
    fn extract_tx_hash(data: &serde_json::Value) -> Option<String> {
        data.get("tx_hash")
            .or_else(|| data.get("transaction_hash"))
            .or_else(|| data.get("hash"))
            .or_else(|| data.get("transaction").and_then(|t| t.get("hash")))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Fetch the current USDC balance from the CLI if available.
    ///
    /// Returns "0" if the CLI is not configured or the balance fetch fails.
    /// This is conservative — a "0" balance will deny transactions if a
    /// min_reserve policy is set, which is safer than allowing them.
    fn fetch_current_balance(&self) -> String {
        if self.cli.is_none() {
            return "0".to_string();
        }
        match self.run_cli(AwalCommand::GetBalance { chain: None }) {
            Ok(output) => {
                // Try to extract USDC balance from the CLI output
                output.data.get("balances")
                    .and_then(|b| b.get("USDC"))
                    .and_then(|u| u.get("formatted"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "0".to_string())
            }
            Err(_) => "0".to_string(),
        }
    }

    /// Get the agent ID this router is bound to.
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Return the list of available tools.
    ///
    /// When `authenticated` is true, all tools are returned.
    /// When false, only `register_agent` is returned (the one tool
    /// that doesn't require prior authentication).
    pub fn list_tools(&self, authenticated: bool) -> Vec<McpTool> {
        if authenticated {
            get_tool_definitions()
        } else {
            get_tool_definitions()
                .into_iter()
                .filter(|t| t.name == "register_agent")
                .collect()
        }
    }

    /// Route a `tools/call` request to the appropriate handler.
    pub fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, AppError> {
        match tool_name {
            "send_payment" => self.handle_send_payment(arguments),
            "check_balance" => self.handle_check_balance(arguments),
            "get_spending_limits" => self.handle_get_spending_limits(arguments),
            "request_limit_increase" => self.handle_request_limit_increase(arguments),
            "get_transactions" => self.handle_get_transactions(arguments),
            "register_agent" => self.handle_register_agent(arguments),
            "get_address" => self.handle_get_address(),
            "trade_tokens" => self.handle_trade_tokens(arguments),
            "pay_x402" => self.handle_pay_x402(arguments),
            "list_x402_services" => self.handle_list_x402_services(),
            "search_x402_services" => self.handle_search_x402_services(arguments),
            "get_x402_details" => self.handle_get_x402_details(arguments),
            "get_agent_info" => self.handle_get_agent_info(),
            _ => Err(AppError::NotFound(format!("Unknown tool: {}", tool_name))),
        }
    }

    // -----------------------------------------------------------------
    // Tool handlers
    // -----------------------------------------------------------------

    /// Validate that a string is a valid Ethereum address (0x + 40 hex chars).
    fn validate_eth_address(addr: &str) -> Result<(), AppError> {
        if !addr.starts_with("0x") || addr.len() != 42 || !addr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::InvalidInput(format!("Invalid Ethereum address: {}", addr)));
        }
        Ok(())
    }

    fn handle_send_payment(&self, arguments: Value) -> Result<Value, AppError> {
        let to = arguments
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'to' field".to_string()))?
            .to_string();
        Self::validate_eth_address(&to)?;
        let amount = arguments
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'amount' field".to_string()))?
            .to_string();
        // Validate amount is a positive number
        let parsed_amount = amount.parse::<rust_decimal::Decimal>()
            .map_err(|_| AppError::InvalidInput("Invalid amount format".to_string()))?;
        if parsed_amount <= rust_decimal::Decimal::ZERO {
            return Err(AppError::InvalidInput("Amount must be positive".to_string()));
        }
        let asset = "USDC".to_string();
        let memo = String::new();

        let now = chrono::Utc::now();
        let now_ts = now.timestamp();
        let tx_id = uuid::Uuid::new_v4().to_string();

        let period_d = daily_period_key(&now);
        let period_w = weekly_period_key(&now);
        let period_m = monthly_period_key(&now);

        let current_balance = self.fetch_current_balance();

        let policy_result = queries::check_policy_and_reserve_atomic(
            &self.db,
            &self.agent_id,
            &amount,
            &to,
            &current_balance,
            &period_d,
            &period_w,
            &period_m,
            now_ts,
        )?;

        match policy_result {
            AtomicPolicyResult::Denied { reason } => {
                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: amount.clone(),
                    asset: asset.clone(),
                    recipient: Some(to.clone()),
                    sender: None,
                    chain_tx_hash: None,
                    status: TxStatus::Denied,
                    category: "payment".to_string(),
                    memo,
                    description: format!("MCP send {} {} to {}", amount, asset, to),
                    service_name: "mcp".to_string(),
                    service_url: String::new(),
                    reason: reason.clone(),
                    webhook_url: None,
                    error_message: Some(reason.clone()),
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;
                Err(AppError::PolicyViolation(reason))
            }
            AtomicPolicyResult::AutoApproved => {
                // Execute send via CLI if available
                let (chain_tx_hash, cli_status, error_msg) = if self.cli.is_some() {
                    let decimal_amount = amount.parse::<rust_decimal::Decimal>()
                        .map_err(|_| AppError::InvalidInput(format!("Invalid amount: {}", amount)))?;
                    match self.run_cli(AwalCommand::Send {
                        to: to.clone(),
                        amount: decimal_amount,
                        chain: None,
                    }) {
                        Ok(output) => {
                            eprintln!("CLI send output data: {:?}", output.data);
                            let hash = Self::extract_tx_hash(&output.data);
                            // If CLI succeeded but returned no tx_hash, mark as Pending (not Confirmed)
                            let status = if hash.is_some() { TxStatus::Confirmed } else { TxStatus::Pending };
                            (hash, status, None)
                        }
                        Err(e) => {
                            // CLI failed — rollback the spending reservation
                            if let Err(rollback_err) = queries::rollback_reservation(
                                &self.db, &self.agent_id, &amount,
                                &period_d, &period_w, &period_m, now_ts,
                            ) {
                                eprintln!("CRITICAL: Failed to rollback reservation for agent {}: {}", self.agent_id, rollback_err);
                            }
                            (None, TxStatus::Failed, Some(e.to_string()))
                        }
                    }
                } else {
                    (None, TxStatus::Pending, None)
                };

                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: amount.clone(),
                    asset: asset.clone(),
                    recipient: Some(to.clone()),
                    sender: None,
                    chain_tx_hash: chain_tx_hash.clone(),
                    status: cli_status.clone(),
                    category: "payment".to_string(),
                    memo,
                    description: format!("MCP send {} {} to {}", amount, asset, to),
                    service_name: "mcp".to_string(),
                    service_url: String::new(),
                    reason: String::new(),
                    webhook_url: None,
                    error_message: error_msg,
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;

                Ok(serde_json::json!({
                    "tx_id": tx_id,
                    "status": cli_status.to_string(),
                    "chain_tx_hash": chain_tx_hash,
                    "amount": amount,
                    "asset": asset,
                    "to": to
                }))
            }
            AtomicPolicyResult::RequiresApproval { reason } => {
                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: amount.clone(),
                    asset: asset.clone(),
                    recipient: Some(to.clone()),
                    sender: None,
                    chain_tx_hash: None,
                    status: TxStatus::AwaitingApproval,
                    category: "payment".to_string(),
                    memo: memo.clone(),
                    description: format!("MCP send {} {} to {}", amount, asset, to),
                    service_name: "mcp".to_string(),
                    service_url: String::new(),
                    reason: reason.clone(),
                    webhook_url: None,
                    error_message: None,
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;

                let approval_id = uuid::Uuid::new_v4().to_string();
                let payload = serde_json::json!({
                    "amount": amount,
                    "asset": asset,
                    "to": to,
                    "memo": memo,
                    "reason": reason,
                });
                let approval = ApprovalRequest {
                    id: approval_id.clone(),
                    agent_id: self.agent_id.clone(),
                    request_type: ApprovalRequestType::Transaction,
                    payload: payload.to_string(),
                    status: ApprovalStatus::Pending,
                    tx_id: Some(tx_id.clone()),
                    expires_at: now_ts + 86400,
                    created_at: now_ts,
                    resolved_at: None,
                    resolved_by: None,
                };
                queries::insert_approval_request(&self.db, &approval)?;

                Ok(serde_json::json!({
                    "tx_id": tx_id,
                    "status": "awaiting_approval",
                    "approval_id": approval_id,
                    "reason": reason,
                    "amount": amount,
                    "asset": asset,
                    "to": to
                }))
            }
        }
    }

    fn handle_check_balance(&self, _arguments: Value) -> Result<Value, AppError> {
        let agent = queries::get_agent(&self.db, &self.agent_id)?;
        if !agent.balance_visible {
            return Ok(serde_json::json!({
                "balance": "hidden",
                "asset": "USDC",
                "message": "Balance visibility is disabled for this agent"
            }));
        }

        if self.cli.is_some() {
            let output = self.run_cli(AwalCommand::GetBalance { chain: None })?;
            // Parse the real balance from CLI output
            if let Some(balances) = output.data.get("balances") {
                let usdc_balance = balances
                    .get("USDC")
                    .and_then(|v| v.get("formatted"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.00");
                return Ok(serde_json::json!({
                    "balance": usdc_balance,
                    "asset": "USDC",
                    "all_balances": balances
                }));
            }
        }

        Ok(serde_json::json!({
            "balance": "0.00",
            "asset": "USDC"
        }))
    }

    fn handle_get_spending_limits(&self, _arguments: Value) -> Result<Value, AppError> {
        let policy = queries::get_spending_policy(&self.db, &self.agent_id)?;
        Ok(serde_json::json!({
            "per_tx_max": policy.per_tx_max,
            "daily_cap": policy.daily_cap,
            "weekly_cap": policy.weekly_cap,
            "monthly_cap": policy.monthly_cap,
            "auto_approve_max": policy.auto_approve_max,
            "allowlist": policy.allowlist
        }))
    }

    fn handle_request_limit_increase(&self, arguments: Value) -> Result<Value, AppError> {
        let reason = arguments
            .get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'reason' field".to_string()))?
            .to_string();

        let new_per_tx_max = arguments
            .get("new_per_tx_max")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let new_daily_cap = arguments
            .get("new_daily_cap")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let now = chrono::Utc::now().timestamp();
        let request_id = uuid::Uuid::new_v4().to_string();

        let payload = serde_json::json!({
            "new_per_tx_max": new_per_tx_max,
            "new_daily_cap": new_daily_cap,
            "reason": reason,
        });

        let approval = ApprovalRequest {
            id: request_id.clone(),
            agent_id: self.agent_id.clone(),
            request_type: ApprovalRequestType::LimitIncrease,
            payload: payload.to_string(),
            status: ApprovalStatus::Pending,
            tx_id: None,
            expires_at: now + 86400,
            created_at: now,
            resolved_at: None,
            resolved_by: None,
        };

        queries::insert_approval_request(&self.db, &approval)?;

        Ok(serde_json::json!({
            "request_id": request_id,
            "status": "pending",
            "message": "Limit increase request submitted for approval"
        }))
    }

    fn handle_get_transactions(&self, arguments: Value) -> Result<Value, AppError> {
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(10);
        let status_filter = arguments
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let (txs, total) = queries::list_transactions_paginated(
            &self.db,
            Some(&self.agent_id),
            status_filter.as_deref(),
            limit,
            0,
        )?;

        let tx_summaries: Vec<Value> = txs
            .iter()
            .map(|tx| {
                serde_json::json!({
                    "id": tx.id,
                    "type": tx.tx_type.to_string(),
                    "amount": tx.amount,
                    "asset": tx.asset,
                    "status": tx.status.to_string(),
                    "recipient": tx.recipient,
                    "memo": tx.memo,
                    "created_at": tx.created_at,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "transactions": tx_summaries,
            "total": total
        }))
    }

    fn handle_get_address(&self) -> Result<Value, AppError> {
        if self.cli.is_some() {
            let output = self.run_cli(AwalCommand::GetAddress)?;
            let address = output.data.as_str()
                .unwrap_or_else(|| output.data.get("address").and_then(|v| v.as_str()).unwrap_or("unknown"));
            return Ok(serde_json::json!({
                "address": address,
                "network": "base"
            }));
        }

        Ok(serde_json::json!({
            "address": "0x0000000000000000000000000000000000000000",
            "network": "base",
            "message": "Address retrieval not yet wired to CLI"
        }))
    }

    /// Handle a token trade request (e.g. ETH -> USDC).
    ///
    /// **Spending cap semantics**: the `amount` field is denominated in units of the
    /// *source* asset (`from_asset`), **not** in USD or any common numeraire.  This
    /// means a per-transaction cap of "0.5" will deny a trade of "1.0" ETH even
    /// though 1 ETH may be worth far more than $0.50.  The policy engine compares
    /// the raw amount string against the agent's spending policy limits directly.
    fn handle_trade_tokens(&self, arguments: Value) -> Result<Value, AppError> {
        let from_asset = arguments
            .get("from_asset")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'from_asset' field".to_string()))?
            .to_string();
        let to_asset = arguments
            .get("to_asset")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'to_asset' field".to_string()))?
            .to_string();
        let amount = arguments
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'amount' field".to_string()))?
            .to_string();
        // Validate amount is a positive number
        let parsed_amount = amount.parse::<rust_decimal::Decimal>()
            .map_err(|_| AppError::InvalidInput("Invalid amount format".to_string()))?;
        if parsed_amount <= rust_decimal::Decimal::ZERO {
            return Err(AppError::InvalidInput("Amount must be positive".to_string()));
        }
        let slippage = arguments
            .get("slippage")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        if from_asset == to_asset {
            return Err(AppError::InvalidInput(
                "Cannot trade a token for itself".to_string(),
            ));
        }

        let now = chrono::Utc::now();
        let now_ts = now.timestamp();
        let tx_id = uuid::Uuid::new_v4().to_string();

        let period_d = daily_period_key(&now);
        let period_w = weekly_period_key(&now);
        let period_m = monthly_period_key(&now);

        let current_balance = self.fetch_current_balance();

        let policy_result = queries::check_policy_and_reserve_atomic(
            &self.db,
            &self.agent_id,
            &amount,
            &format!("trade:{}>{}", from_asset, to_asset),
            &current_balance,
            &period_d,
            &period_w,
            &period_m,
            now_ts,
        )?;

        match policy_result {
            AtomicPolicyResult::Denied { reason } => {
                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: amount.clone(),
                    asset: from_asset.clone(),
                    recipient: None,
                    sender: None,
                    chain_tx_hash: None,
                    status: TxStatus::Denied,
                    category: "trade".to_string(),
                    memo: String::new(),
                    description: format!("Trade {} {} -> {}", amount, from_asset, to_asset),
                    service_name: "mcp".to_string(),
                    service_url: String::new(),
                    reason: reason.clone(),
                    webhook_url: None,
                    error_message: Some(reason.clone()),
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;
                Err(AppError::PolicyViolation(reason))
            }
            AtomicPolicyResult::AutoApproved => {
                // Execute trade via CLI if available
                let (chain_tx_hash, cli_status, error_msg) = if self.cli.is_some() {
                    match self.run_cli(AwalCommand::Trade {
                        from: from_asset.clone(),
                        to: to_asset.clone(),
                        amount: amount.clone(),
                        slippage,
                    }) {
                        Ok(output) => {
                            eprintln!("CLI trade output data: {:?}", output.data);
                            let hash = Self::extract_tx_hash(&output.data);
                            let status = if hash.is_some() { TxStatus::Confirmed } else { TxStatus::Pending };
                            (hash, status, None)
                        }
                        Err(e) => {
                            // CLI failed — rollback the spending reservation
                            if let Err(rollback_err) = queries::rollback_reservation(
                                &self.db, &self.agent_id, &amount,
                                &period_d, &period_w, &period_m, now_ts,
                            ) {
                                eprintln!("CRITICAL: Failed to rollback reservation for agent {}: {}", self.agent_id, rollback_err);
                            }
                            (None, TxStatus::Failed, Some(e.to_string()))
                        }
                    }
                } else {
                    (None, TxStatus::Pending, None)
                };

                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: amount.clone(),
                    asset: from_asset.clone(),
                    recipient: None,
                    sender: None,
                    chain_tx_hash: chain_tx_hash.clone(),
                    status: cli_status.clone(),
                    category: "trade".to_string(),
                    memo: String::new(),
                    description: format!("Trade {} {} -> {}", amount, from_asset, to_asset),
                    service_name: "mcp".to_string(),
                    service_url: String::new(),
                    reason: String::new(),
                    webhook_url: None,
                    error_message: error_msg,
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;

                Ok(serde_json::json!({
                    "tx_id": tx_id,
                    "status": cli_status.to_string(),
                    "chain_tx_hash": chain_tx_hash,
                    "from_asset": from_asset,
                    "to_asset": to_asset,
                    "amount": amount
                }))
            }
            AtomicPolicyResult::RequiresApproval { reason } => {
                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: amount.clone(),
                    asset: from_asset.clone(),
                    recipient: None,
                    sender: None,
                    chain_tx_hash: None,
                    status: TxStatus::AwaitingApproval,
                    category: "trade".to_string(),
                    memo: String::new(),
                    description: format!("Trade {} {} -> {}", amount, from_asset, to_asset),
                    service_name: "mcp".to_string(),
                    service_url: String::new(),
                    reason: reason.clone(),
                    webhook_url: None,
                    error_message: None,
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;

                let approval_id = uuid::Uuid::new_v4().to_string();
                let payload = serde_json::json!({
                    "from_asset": from_asset,
                    "to_asset": to_asset,
                    "amount": amount,
                    "reason": reason,
                });
                let approval = ApprovalRequest {
                    id: approval_id.clone(),
                    agent_id: self.agent_id.clone(),
                    request_type: ApprovalRequestType::Transaction,
                    payload: payload.to_string(),
                    status: ApprovalStatus::Pending,
                    tx_id: Some(tx_id.clone()),
                    expires_at: now_ts + 86400,
                    created_at: now_ts,
                    resolved_at: None,
                    resolved_by: None,
                };
                queries::insert_approval_request(&self.db, &approval)?;

                Ok(serde_json::json!({
                    "tx_id": tx_id,
                    "status": "awaiting_approval",
                    "approval_id": approval_id,
                    "reason": reason,
                    "from_asset": from_asset,
                    "to_asset": to_asset,
                    "amount": amount
                }))
            }
        }
    }

    fn handle_pay_x402(&self, arguments: Value) -> Result<Value, AppError> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'url' field".to_string()))?
            .to_string();
        let max_amount = arguments
            .get("max_amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("max_amount is required for x402 payments".into()))?
            .to_string();
        // Validate max_amount is a positive number
        let parsed_max_amount = max_amount.parse::<rust_decimal::Decimal>()
            .map_err(|_| AppError::InvalidInput("Invalid amount format".to_string()))?;
        if parsed_max_amount <= rust_decimal::Decimal::ZERO {
            return Err(AppError::InvalidInput("Amount must be positive".to_string()));
        }
        let method = arguments
            .get("method")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let data = arguments
            .get("data")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let headers = arguments
            .get("headers")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // For x402, we use the max_amount as the policy-checked amount.
        let policy_amount = max_amount.as_str();

        let now = chrono::Utc::now();
        let now_ts = now.timestamp();
        let tx_id = uuid::Uuid::new_v4().to_string();

        let period_d = daily_period_key(&now);
        let period_w = weekly_period_key(&now);
        let period_m = monthly_period_key(&now);

        let current_balance = self.fetch_current_balance();

        let policy_result = queries::check_policy_and_reserve_atomic(
            &self.db,
            &self.agent_id,
            policy_amount,
            &url,
            &current_balance,
            &period_d,
            &period_w,
            &period_m,
            now_ts,
        )?;

        match policy_result {
            AtomicPolicyResult::Denied { reason } => {
                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: policy_amount.to_string(),
                    asset: "USDC".to_string(),
                    recipient: Some(url.clone()),
                    sender: None,
                    chain_tx_hash: None,
                    status: TxStatus::Denied,
                    category: "x402".to_string(),
                    memo: String::new(),
                    description: format!("X402 payment to {}", url),
                    service_name: "x402".to_string(),
                    service_url: url.clone(),
                    reason: reason.clone(),
                    webhook_url: None,
                    error_message: Some(reason.clone()),
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;
                Err(AppError::PolicyViolation(reason))
            }
            AtomicPolicyResult::AutoApproved => {
                // Execute x402 payment via CLI if available
                let (chain_tx_hash, cli_status, error_msg, response_body, amount_paid, response_status) = if self.cli.is_some() {
                    match self.run_cli(AwalCommand::X402Pay {
                        url: url.clone(),
                        max_amount: Some(max_amount.clone()),
                        method: method.clone(),
                        data: data.clone(),
                        headers: headers.clone(),
                    }) {
                        Ok(output) => {
                            eprintln!("CLI x402 output data: {:?}", output.data);
                            let hash = Self::extract_tx_hash(&output.data);
                            let status = if hash.is_some() { TxStatus::Confirmed } else { TxStatus::Pending };
                            // Capture response data for the agent
                            let response_body = output.data.get("response_body").cloned();
                            let amount_paid = output.data.get("amount_paid")
                                .or_else(|| output.data.get("amount"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let response_status = output.data.get("response_status")
                                .or_else(|| output.data.get("status_code"))
                                .and_then(|v| v.as_u64());
                            (hash, status, None, response_body, amount_paid, response_status)
                        }
                        Err(e) => {
                            // CLI failed — rollback the spending reservation
                            if let Err(rollback_err) = queries::rollback_reservation(
                                &self.db, &self.agent_id, policy_amount,
                                &period_d, &period_w, &period_m, now_ts,
                            ) {
                                eprintln!("CRITICAL: Failed to rollback reservation for agent {}: {}", self.agent_id, rollback_err);
                            }
                            (None, TxStatus::Failed, Some(e.to_string()), None, None, None)
                        }
                    }
                } else {
                    (None, TxStatus::Pending, None, None, None, None)
                };

                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: policy_amount.to_string(),
                    asset: "USDC".to_string(),
                    recipient: Some(url.clone()),
                    sender: None,
                    chain_tx_hash: chain_tx_hash.clone(),
                    status: cli_status.clone(),
                    category: "x402".to_string(),
                    memo: String::new(),
                    description: format!("X402 payment to {}", url),
                    service_name: "x402".to_string(),
                    service_url: url.clone(),
                    reason: String::new(),
                    webhook_url: None,
                    error_message: error_msg,
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;

                Ok(serde_json::json!({
                    "tx_id": tx_id,
                    "status": cli_status.to_string(),
                    "chain_tx_hash": chain_tx_hash,
                    "url": url,
                    "amount": policy_amount,
                    "response_body": response_body,
                    "amount_paid": amount_paid,
                    "response_status": response_status
                }))
            }
            AtomicPolicyResult::RequiresApproval { reason } => {
                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: policy_amount.to_string(),
                    asset: "USDC".to_string(),
                    recipient: Some(url.clone()),
                    sender: None,
                    chain_tx_hash: None,
                    status: TxStatus::AwaitingApproval,
                    category: "x402".to_string(),
                    memo: String::new(),
                    description: format!("X402 payment to {}", url),
                    service_name: "x402".to_string(),
                    service_url: url.clone(),
                    reason: reason.clone(),
                    webhook_url: None,
                    error_message: None,
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;

                let approval_id = uuid::Uuid::new_v4().to_string();
                let payload = serde_json::json!({
                    "url": url,
                    "amount": policy_amount,
                    "reason": reason,
                });
                let approval = ApprovalRequest {
                    id: approval_id.clone(),
                    agent_id: self.agent_id.clone(),
                    request_type: ApprovalRequestType::Transaction,
                    payload: payload.to_string(),
                    status: ApprovalStatus::Pending,
                    tx_id: Some(tx_id.clone()),
                    expires_at: now_ts + 86400,
                    created_at: now_ts,
                    resolved_at: None,
                    resolved_by: None,
                };
                queries::insert_approval_request(&self.db, &approval)?;

                Ok(serde_json::json!({
                    "tx_id": tx_id,
                    "status": "awaiting_approval",
                    "approval_id": approval_id,
                    "reason": reason,
                    "url": url,
                    "amount": policy_amount
                }))
            }
        }
    }

    fn handle_list_x402_services(&self) -> Result<Value, AppError> {
        if self.cli.is_some() {
            let output = self.run_cli(AwalCommand::X402BazaarList)?;
            return Ok(output.data);
        }

        Ok(serde_json::json!({
            "services": [],
            "message": "X402 bazaar listing not yet wired to CLI"
        }))
    }

    fn handle_search_x402_services(&self, arguments: Value) -> Result<Value, AppError> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'query' field".to_string()))?
            .to_string();

        if self.cli.is_some() {
            let output = self.run_cli(AwalCommand::X402BazaarSearch { query: query.clone() })?;
            // Client-side filtering in case CLI returns unfiltered results
            let query_lower = query.to_lowercase();
            if let Some(services) = output.data.get("services").and_then(|s| s.as_array()) {
                let filtered: Vec<&Value> = services.iter().filter(|svc| {
                    let desc = svc.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    let resource = svc.get("resource").and_then(|v| v.as_str()).unwrap_or("");
                    let name = svc.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    desc.to_lowercase().contains(&query_lower)
                        || resource.to_lowercase().contains(&query_lower)
                        || name.to_lowercase().contains(&query_lower)
                }).collect();
                return Ok(serde_json::json!({
                    "query": query,
                    "services": filtered,
                    "total": filtered.len()
                }));
            }
            return Ok(output.data);
        }

        Ok(serde_json::json!({
            "query": query,
            "services": [],
            "message": "X402 bazaar search not yet wired to CLI"
        }))
    }

    fn handle_get_x402_details(&self, arguments: Value) -> Result<Value, AppError> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'url' field".to_string()))?
            .to_string();

        if self.cli.is_some() {
            let output = self.run_cli(AwalCommand::X402Details { url: url.clone() })?;
            return Ok(output.data);
        }

        Ok(serde_json::json!({
            "url": url,
            "amount": null,
            "asset": null,
            "description": null,
            "message": "X402 details not yet wired to CLI"
        }))
    }

    fn handle_get_agent_info(&self) -> Result<Value, AppError> {
        let agent = queries::get_agent(&self.db, &self.agent_id)?;
        Ok(serde_json::json!({
            "agent_id": agent.id,
            "name": agent.name,
            "status": agent.status.to_string(),
            "created_at": agent.created_at,
            "purpose": agent.purpose,
            "agent_type": agent.agent_type
        }))
    }

    fn handle_register_agent(&self, arguments: Value) -> Result<Value, AppError> {
        use sha2::{Digest, Sha256};

        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'name' field".to_string()))?
            .to_string();
        let purpose = arguments
            .get("purpose")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'purpose' field".to_string()))?
            .to_string();
        let invitation_code = arguments
            .get("invitation_code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AppError::InvalidInput("Missing 'invitation_code' field".to_string())
            })?
            .to_string();

        let now = chrono::Utc::now().timestamp();
        let agent_id = uuid::Uuid::new_v4().to_string();

        // Atomically check and consume the invitation code (prevents race conditions)
        queries::try_consume_invitation_code(&self.db, &invitation_code, &agent_id)?;

        // Generate a 32-byte random API token for the agent
        use rand::Rng;
        let token_bytes: [u8; 32] = rand::thread_rng().gen();
        let token: String = token_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        let token_prefix = token[..8].to_string();

        // Hash the token with SHA-256 for storage
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        let agent = Agent {
            id: agent_id.clone(),
            name: name.clone(),
            description: String::new(),
            purpose,
            agent_type: "mcp".to_string(),
            capabilities: vec!["send".to_string()],
            status: AgentStatus::Pending,
            api_token_hash: Some(token_hash),
            token_prefix: Some(token_prefix),
            balance_visible: true,
            invitation_code: Some(invitation_code.clone()),
            created_at: now,
            updated_at: now,
            last_active_at: None,
            metadata: "{}".to_string(),
        };

        queries::insert_agent(&self.db, &agent)?;

        // Now that the agent row exists, set used_by on the invitation code (FK-safe)
        queries::set_invitation_code_used_by(&self.db, &invitation_code, &agent_id)?;

        Ok(serde_json::json!({
            "agent_id": agent_id,
            "name": name,
            "status": "pending",
            "token": token,
            "message": "Agent registration submitted, pending approval. Save this token — it will not be shown again."
        }))
    }
}

/// Map AppError variants to JSON-RPC error codes.
pub fn error_code(err: &AppError) -> i32 {
    match err {
        AppError::NotFound(_) => -32601,
        AppError::InvalidInput(_) => -32602,
        AppError::InvalidToken => -32000,
        AppError::AuthError(_) => -32000,
        AppError::PolicyViolation(_) => -32001,
        AppError::KillSwitchActive(_) => -32002,
        AppError::AgentSuspended(_) => -32003,
        AppError::InvalidInvitationCode => -32004,
        AppError::RateLimited => -32005,
        AppError::CliError(_) => -32006,
        _ => -32603,
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::queries::{insert_agent, insert_spending_policy};
    use crate::test_helpers::{
        create_test_agent, create_test_invitation, create_test_spending_policy, setup_test_db,
    };

    fn make_router(db: Arc<Database>, agent_id: &str) -> McpRouter {
        McpRouter::new(db, agent_id.to_string())
    }

    fn setup_agent_with_policy(db: &Arc<Database>) -> String {
        let agent = create_test_agent("TestBot", AgentStatus::Active);
        let agent_id = agent.id.clone();
        insert_agent(db, &agent).unwrap();

        let policy = create_test_spending_policy(&agent_id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(db, &policy).unwrap();

        agent_id
    }

    // -- list_tools tests --

    #[test]
    fn test_list_tools_authenticated_returns_all() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let tools = router.list_tools(true);
        assert_eq!(tools.len(), 13);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"send_payment"));
        assert!(names.contains(&"check_balance"));
        assert!(names.contains(&"register_agent"));
        assert!(names.contains(&"get_address"));
        assert!(names.contains(&"trade_tokens"));
        assert!(names.contains(&"pay_x402"));
        assert!(names.contains(&"list_x402_services"));
        assert!(names.contains(&"search_x402_services"));
        assert!(names.contains(&"get_x402_details"));
        assert!(names.contains(&"get_agent_info"));
    }

    #[test]
    fn test_list_tools_unauthenticated_returns_only_register() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let tools = router.list_tools(false);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "register_agent");
    }

    // -- handle_tool_call dispatch tests --

    #[test]
    fn test_dispatch_send_payment() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db.clone(), &agent_id);

        let result = router
            .handle_tool_call(
                "send_payment",
                serde_json::json!({
                    "to": "0x1234567890abcdef1234567890abcdef12345678",
                    "amount": "25.50",
                    "asset": "USDC",
                    "memo": "test payment"
                }),
            )
            .unwrap();

        assert!(result.get("tx_id").is_some());
        assert_eq!(result["status"], "pending");
        assert_eq!(result["amount"], "25.50");
        assert_eq!(result["asset"], "USDC");
        assert_eq!(result["to"], "0x1234567890abcdef1234567890abcdef12345678");

        // Verify persistence
        let tx_id = result["tx_id"].as_str().unwrap();
        let tx = queries::get_transaction(&db, tx_id).unwrap();
        assert_eq!(tx.agent_id.as_deref(), Some(agent_id.as_str()));
        assert_eq!(tx.amount, "25.50");
    }

    #[test]
    fn test_dispatch_check_balance() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call("check_balance", serde_json::json!({}))
            .unwrap();

        assert!(result.get("balance").is_some());
        assert!(result.get("asset").is_some());
        assert_eq!(result["asset"], "USDC");
    }

    #[test]
    fn test_dispatch_get_spending_limits() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call("get_spending_limits", serde_json::json!({}))
            .unwrap();

        assert_eq!(result["per_tx_max"], "100");
        assert_eq!(result["daily_cap"], "1000");
        assert_eq!(result["weekly_cap"], "5000");
        assert_eq!(result["monthly_cap"], "20000");
        assert_eq!(result["auto_approve_max"], "50");
        assert!(result.get("allowlist").is_some());
    }

    #[test]
    fn test_dispatch_request_limit_increase() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call(
                "request_limit_increase",
                serde_json::json!({
                    "new_per_tx_max": "500",
                    "new_daily_cap": "5000",
                    "reason": "Need higher limits for vendor payments"
                }),
            )
            .unwrap();

        assert!(result.get("request_id").is_some());
        assert_eq!(result["status"], "pending");
    }

    #[test]
    fn test_dispatch_get_transactions() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db.clone(), &agent_id);

        // Create some transactions first
        for i in 0..3 {
            router
                .handle_tool_call(
                    "send_payment",
                    serde_json::json!({
                        "to": format!("0x{:0>40}", format!("{:04}", i)),
                        "amount": format!("{}.00", i + 1)
                    }),
                )
                .unwrap();
        }

        let result = router
            .handle_tool_call("get_transactions", serde_json::json!({ "limit": 2 }))
            .unwrap();

        let txs = result["transactions"].as_array().unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(result["total"], 3);
    }

    #[test]
    fn test_dispatch_register_agent() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db.clone(), &agent_id);

        let invitation = create_test_invitation("INV-ROUTER-001", "Router test invite");
        queries::insert_invitation_code(&db, &invitation).unwrap();

        let result = router
            .handle_tool_call(
                "register_agent",
                serde_json::json!({
                    "name": "NewAgent",
                    "purpose": "Automated payments",
                    "invitation_code": "INV-ROUTER-001"
                }),
            )
            .unwrap();

        assert!(result.get("agent_id").is_some());
        assert_eq!(result["status"], "pending");
        assert_eq!(result["name"], "NewAgent");
    }

    #[test]
    fn test_dispatch_unknown_tool_returns_error() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call("nonexistent_tool", serde_json::json!({}));
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::NotFound(msg) => assert!(msg.contains("nonexistent_tool")),
            other => panic!("Expected NotFound, got: {:?}", other),
        }
    }

    // -- per-agent isolation --

    #[test]
    fn test_per_agent_isolation() {
        let db = setup_test_db();

        let agent_a = create_test_agent("AgentA", AgentStatus::Active);
        let agent_b = create_test_agent("AgentB", AgentStatus::Active);
        let id_a = agent_a.id.clone();
        let id_b = agent_b.id.clone();
        insert_agent(&db, &agent_a).unwrap();
        insert_agent(&db, &agent_b).unwrap();

        let policy_a = create_test_spending_policy(&id_a, "100", "1000", "5000", "20000", "50");
        let policy_b = create_test_spending_policy(&id_b, "200", "2000", "10000", "40000", "100");
        insert_spending_policy(&db, &policy_a).unwrap();
        insert_spending_policy(&db, &policy_b).unwrap();

        let router_a = make_router(db.clone(), &id_a);
        let router_b = make_router(db.clone(), &id_b);

        // Different policies
        let limits_a = router_a
            .handle_tool_call("get_spending_limits", serde_json::json!({}))
            .unwrap();
        assert_eq!(limits_a["per_tx_max"], "100");

        let limits_b = router_b
            .handle_tool_call("get_spending_limits", serde_json::json!({}))
            .unwrap();
        assert_eq!(limits_b["per_tx_max"], "200");

        // Transaction isolation
        router_a
            .handle_tool_call(
                "send_payment",
                serde_json::json!({ "to": "0x000000000000000000000000000000000000aaaa", "amount": "10" }),
            )
            .unwrap();

        let txs_b = router_b
            .handle_tool_call("get_transactions", serde_json::json!({}))
            .unwrap();
        assert_eq!(
            txs_b["transactions"].as_array().unwrap().len(),
            0,
            "Agent B should not see Agent A's transactions"
        );
    }

    // -- missing fields --

    #[test]
    fn test_send_payment_missing_fields() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        // Missing 'to'
        assert!(router
            .handle_tool_call("send_payment", serde_json::json!({ "amount": "10" }))
            .is_err());

        // Missing 'amount'
        assert!(router
            .handle_tool_call("send_payment", serde_json::json!({ "to": "0x123" }))
            .is_err());
    }

    // -- error_code mapping --

    // -- get_address tests --

    #[test]
    fn test_get_address_returns_address_string() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call("get_address", serde_json::json!({}))
            .unwrap();

        assert!(result.get("address").is_some());
        let addr = result["address"].as_str().unwrap();
        assert!(addr.starts_with("0x"));
        assert!(result.get("network").is_some());
    }

    // -- trade_tokens tests --

    #[test]
    fn test_trade_tokens_validates_required_fields() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        // Missing from_asset
        assert!(router
            .handle_tool_call(
                "trade_tokens",
                serde_json::json!({ "to_asset": "USDC", "amount": "1.0" })
            )
            .is_err());

        // Missing to_asset
        assert!(router
            .handle_tool_call(
                "trade_tokens",
                serde_json::json!({ "from_asset": "ETH", "amount": "1.0" })
            )
            .is_err());

        // Missing amount
        assert!(router
            .handle_tool_call(
                "trade_tokens",
                serde_json::json!({ "from_asset": "ETH", "to_asset": "USDC" })
            )
            .is_err());
    }

    #[test]
    fn test_trade_tokens_same_asset_error() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "trade_tokens",
            serde_json::json!({ "from_asset": "ETH", "to_asset": "ETH", "amount": "1.0" }),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("itself")),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn test_trade_tokens_applies_policy() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        // Amount within auto-approve (50)
        let result = router
            .handle_tool_call(
                "trade_tokens",
                serde_json::json!({
                    "from_asset": "ETH",
                    "to_asset": "USDC",
                    "amount": "25"
                }),
            )
            .unwrap();

        assert!(result.get("tx_id").is_some());
        assert_eq!(result["status"], "pending");
        assert_eq!(result["from_asset"], "ETH");
        assert_eq!(result["to_asset"], "USDC");
        assert_eq!(result["amount"], "25");
    }

    #[test]
    fn test_trade_tokens_over_per_tx_max_denied() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db); // per_tx_max = 100
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "trade_tokens",
            serde_json::json!({
                "from_asset": "ETH",
                "to_asset": "USDC",
                "amount": "200"
            }),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::PolicyViolation(_) => {}
            other => panic!("Expected PolicyViolation, got: {:?}", other),
        }
    }

    /// Spending caps are denominated in the source asset's units, not USD.
    /// A per_tx_max of "0.5" must deny a trade of "1.0" regardless of the
    /// assets' dollar values.
    #[test]
    fn test_trade_tokens_caps_denominated_in_source_asset() {
        let db = setup_test_db();
        let agent = create_test_agent("SmallCapBot", AgentStatus::Active);
        let agent_id = agent.id.clone();
        insert_agent(&db, &agent).unwrap();
        // per_tx_max = 0.5
        let policy = create_test_spending_policy(&agent_id, "0.5", "1000", "5000", "20000", "0.5");
        insert_spending_policy(&db, &policy).unwrap();

        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "trade_tokens",
            serde_json::json!({
                "from_asset": "ETH",
                "to_asset": "USDC",
                "amount": "1.0"
            }),
        );

        assert!(result.is_err(), "Trade of 1.0 should be denied with per_tx_max of 0.5");
        match result.unwrap_err() {
            AppError::PolicyViolation(msg) => {
                assert!(msg.contains("per-tx"), "Denial reason should mention per-tx limit, got: {}", msg);
            }
            other => panic!("Expected PolicyViolation, got: {:?}", other),
        }
    }

    // -- pay_x402 tests --

    #[test]
    fn test_pay_x402_validates_url() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        // Missing url
        let result = router.handle_tool_call("pay_x402", serde_json::json!({}));
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("url")),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn test_pay_x402_requires_max_amount() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        // url present but max_amount missing -> should error
        let result = router.handle_tool_call(
            "pay_x402",
            serde_json::json!({ "url": "https://example.com/api" }),
        );
        assert!(result.is_err(), "pay_x402 without max_amount should fail");
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => {
                assert!(msg.contains("max_amount"), "Error should mention max_amount, got: {}", msg);
            }
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn test_pay_x402_applies_policy() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call(
                "pay_x402",
                serde_json::json!({
                    "url": "https://example.com/api/data",
                    "max_amount": "10"
                }),
            )
            .unwrap();

        assert!(result.get("tx_id").is_some());
        assert_eq!(result["status"], "pending");
        assert_eq!(result["url"], "https://example.com/api/data");
    }

    #[test]
    fn test_pay_x402_over_limit_denied() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db); // per_tx_max = 100
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "pay_x402",
            serde_json::json!({
                "url": "https://expensive-service.com/api",
                "max_amount": "200"
            }),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::PolicyViolation(_) => {}
            other => panic!("Expected PolicyViolation, got: {:?}", other),
        }
    }

    // -- list_x402_services tests --

    #[test]
    fn test_list_x402_services_returns_expected_shape() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call("list_x402_services", serde_json::json!({}))
            .unwrap();

        assert!(result.get("services").is_some());
        assert!(result["services"].is_array());
    }

    // -- search_x402_services tests --

    #[test]
    fn test_search_x402_services_validates_query() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        // Missing query
        let result = router.handle_tool_call("search_x402_services", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_search_x402_services_returns_expected_shape() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call(
                "search_x402_services",
                serde_json::json!({ "query": "weather" }),
            )
            .unwrap();

        assert!(result.get("services").is_some());
        assert!(result["services"].is_array());
        assert_eq!(result["query"], "weather");
    }

    // -- get_x402_details tests --

    #[test]
    fn test_get_x402_details_validates_url() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call("get_x402_details", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_x402_details_returns_expected_shape() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call(
                "get_x402_details",
                serde_json::json!({ "url": "https://example.com/api" }),
            )
            .unwrap();

        assert_eq!(result["url"], "https://example.com/api");
    }

    // -- get_agent_info tests --

    #[test]
    fn test_get_agent_info_returns_agent_profile() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router
            .handle_tool_call("get_agent_info", serde_json::json!({}))
            .unwrap();

        assert_eq!(result["agent_id"], agent_id);
        assert_eq!(result["name"], "TestBot");
        assert!(result.get("status").is_some());
        assert!(result.get("created_at").is_some());
        assert!(result.get("purpose").is_some());
        assert!(result.get("agent_type").is_some());
    }

    // =====================================================================
    // CLI-wired handler tests
    // =====================================================================

    fn make_router_with_cli(db: Arc<Database>, agent_id: &str) -> McpRouter {
        use crate::cli::executor::MockCliExecutor;
        let mock = MockCliExecutor::with_defaults();
        McpRouter::new_with_cli(db, agent_id.to_string(), Arc::new(mock))
    }

    #[tokio::test]
    async fn test_check_balance_with_cli_returns_real_balance() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router_with_cli(db, &agent_id);

        let result = router
            .handle_tool_call("check_balance", serde_json::json!({}))
            .unwrap();

        assert_eq!(result["balance"], "1247.83");
        assert_eq!(result["asset"], "USDC");
        assert!(result.get("all_balances").is_some());
    }

    #[tokio::test]
    async fn test_get_address_with_cli_returns_real_address() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router_with_cli(db, &agent_id);

        let result = router
            .handle_tool_call("get_address", serde_json::json!({}))
            .unwrap();

        assert_eq!(result["address"], "0xMockWalletAddress123");
        assert_eq!(result["network"], "base");
        assert!(result.get("message").is_none());
    }

    #[tokio::test]
    async fn test_list_x402_services_with_cli() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router_with_cli(db, &agent_id);

        let result = router
            .handle_tool_call("list_x402_services", serde_json::json!({}))
            .unwrap();

        let services = result["services"].as_array().unwrap();
        assert_eq!(services.len(), 2);
        assert_eq!(services[0]["name"], "Weather API");
    }

    #[tokio::test]
    async fn test_search_x402_services_with_cli() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router_with_cli(db, &agent_id);

        let result = router
            .handle_tool_call("search_x402_services", serde_json::json!({ "query": "weather" }))
            .unwrap();

        let results = result["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_get_x402_details_with_cli() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router_with_cli(db, &agent_id);

        let result = router
            .handle_tool_call("get_x402_details", serde_json::json!({ "url": "https://weather.x402.org" }))
            .unwrap();

        assert_eq!(result["price"], "0.01");
        assert_eq!(result["asset"], "USDC");
    }

    #[tokio::test]
    async fn test_send_payment_with_cli_executes_and_confirms() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router_with_cli(db.clone(), &agent_id);

        let result = router
            .handle_tool_call(
                "send_payment",
                serde_json::json!({
                    "to": "0x1234567890abcdef1234567890abcdef12345678",
                    "amount": "25.50",
                    "asset": "USDC"
                }),
            )
            .unwrap();

        assert_eq!(result["status"], "confirmed");
        assert_eq!(result["chain_tx_hash"], "0xmock_tx_hash_abc123");
    }

    #[tokio::test]
    async fn test_trade_tokens_with_cli_executes_and_confirms() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router_with_cli(db.clone(), &agent_id);

        let result = router
            .handle_tool_call(
                "trade_tokens",
                serde_json::json!({
                    "from_asset": "ETH",
                    "to_asset": "USDC",
                    "amount": "1.0"
                }),
            )
            .unwrap();

        assert_eq!(result["status"], "confirmed");
        assert_eq!(result["chain_tx_hash"], "0xmock_trade_hash_def456");
    }

    #[test]
    fn test_error_code_mapping() {
        assert_eq!(error_code(&AppError::NotFound("x".into())), -32601);
        assert_eq!(error_code(&AppError::InvalidInput("x".into())), -32602);
        assert_eq!(error_code(&AppError::InvalidToken), -32000);
        assert_eq!(error_code(&AppError::AuthError("x".into())), -32000);
        assert_eq!(error_code(&AppError::PolicyViolation("x".into())), -32001);
        assert_eq!(error_code(&AppError::KillSwitchActive("x".into())), -32002);
        assert_eq!(error_code(&AppError::AgentSuspended("x".into())), -32003);
        assert_eq!(error_code(&AppError::InvalidInvitationCode), -32004);
        assert_eq!(error_code(&AppError::RateLimited), -32005);
        assert_eq!(error_code(&AppError::Internal("x".into())), -32603);
    }

    // =====================================================================
    // Regression tests for Round 2 review fixes
    // =====================================================================

    /// Regression: CLI failure must rollback spending reservation so the cap
    /// isn't permanently consumed. A subsequent transaction within limits
    /// should succeed (not be denied by a phantom reservation).
    #[test]
    fn test_cli_failure_rolls_back_spending_reservation() {
        use crate::cli::executor::{CliError, CliExecutable, CliOutput};

        // A CLI executor that always fails
        struct FailingCli;

        #[async_trait::async_trait]
        impl CliExecutable for FailingCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Err(CliError::CommandFailed {
                    stderr: "Simulated CLI failure".to_string(),
                    exit_code: Some(1),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db); // daily_cap=1000, per_tx=100, auto_approve=50

        // First: send with failing CLI — should fail but NOT consume cap
        let failing_router = McpRouter::new_with_cli(
            db.clone(), agent_id.clone(), Arc::new(FailingCli),
        );
        let result = failing_router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x000000000000000000000000000000000000abcd", "amount": "40" }),
        ).unwrap();
        assert_eq!(result["status"], "failed", "CLI failure should produce 'failed' status");

        // Second: send again with no CLI (Pending path) — should succeed
        // because the first reservation was rolled back
        let router = make_router(db.clone(), &agent_id);
        let result2 = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x0000000000000000000000000000000000000def", "amount": "40" }),
        ).unwrap();
        assert_eq!(result2["status"], "pending", "Second tx should succeed after rollback");

        // Verify we can still do more — the 40 from the failed tx isn't counted
        // Daily cap is 1000, so 40 (pending) + 40 (this) = 80 < 1000
        let result3 = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x00000000000000000000000000000000000000ab", "amount": "40" }),
        ).unwrap();
        assert_eq!(result3["status"], "pending");
    }

    /// Regression: CLI failure must store the error message in the transaction record.
    #[test]
    fn test_cli_failure_stores_error_message_in_transaction() {
        use crate::cli::executor::{CliError, CliExecutable, CliOutput};

        struct FailingCli;

        #[async_trait::async_trait]
        impl CliExecutable for FailingCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Err(CliError::CommandFailed {
                    stderr: "insufficient funds for gas".to_string(),
                    exit_code: Some(1),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(FailingCli));

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x000000000000000000000000000000000000abcd", "amount": "25" }),
        ).unwrap();

        let tx_id = result["tx_id"].as_str().unwrap();
        let tx = queries::get_transaction(&db, tx_id).unwrap();
        assert_eq!(tx.status, TxStatus::Failed);
        assert!(
            tx.error_message.is_some(),
            "Failed transaction should have error_message set"
        );
        // After CLI error sanitization (BUG-1), the stored error is the sanitized message
        let err_msg = tx.error_message.unwrap();
        assert!(
            err_msg.contains("Wallet operation failed") || err_msg.contains("insufficient funds"),
            "Error message should contain sanitized or original error text, got: {}", err_msg
        );
    }

    /// Regression: CLI success without tx_hash should mark as Pending, not Confirmed.
    #[test]
    fn test_cli_success_without_tx_hash_marks_pending() {
        use crate::cli::executor::{CliExecutable, CliOutput, CliError};

        // A CLI that succeeds but returns no tx_hash
        struct NoHashCli;

        #[async_trait::async_trait]
        impl CliExecutable for NoHashCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Ok(CliOutput {
                    success: true,
                    data: serde_json::json!({"message": "submitted"}),
                    raw: r#"{"message":"submitted"}"#.to_string(),
                    stderr: String::new(),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(NoHashCli));

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x000000000000000000000000000000000000abcd", "amount": "25" }),
        ).unwrap();

        assert_eq!(
            result["status"], "pending",
            "CLI success without tx_hash should be 'pending', not 'confirmed'"
        );
        assert!(result["chain_tx_hash"].is_null());
    }

    /// Regression: register_agent must return a non-empty token that can authenticate.
    #[test]
    fn test_register_agent_returns_valid_token() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db.clone(), &agent_id);

        let invitation = create_test_invitation("INV-TOKEN-001", "Token test");
        queries::insert_invitation_code(&db, &invitation).unwrap();

        let result = router.handle_tool_call(
            "register_agent",
            serde_json::json!({
                "name": "TokenBot",
                "purpose": "Test token generation",
                "invitation_code": "INV-TOKEN-001"
            }),
        ).unwrap();

        // Token must be present and non-empty
        let token = result["token"].as_str().expect("Response must include 'token'");
        assert!(!token.is_empty(), "Token must not be empty");
        assert_eq!(token.len(), 64, "Token should be 32 bytes hex-encoded (64 chars)");

        // The token hash should be stored in the agent record
        let new_agent_id = result["agent_id"].as_str().unwrap();
        let agent = queries::get_agent(&db, new_agent_id).unwrap();
        assert!(agent.api_token_hash.is_some(), "Agent should have token hash");
        assert!(agent.token_prefix.is_some(), "Agent should have token prefix");

        // Verify the token actually hashes to the stored value
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let expected_hash = format!("{:x}", hasher.finalize());
        assert_eq!(
            agent.api_token_hash.unwrap(), expected_hash,
            "Stored hash must match SHA-256 of the returned token"
        );
    }

    /// Regression: register_agent must increment invitation code use_count.
    #[test]
    fn test_register_agent_increments_invitation_use_count() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db.clone(), &agent_id);

        let invitation = create_test_invitation("INV-COUNT-001", "Count test");
        queries::insert_invitation_code(&db, &invitation).unwrap();

        // Before registration
        let code_before = queries::get_invitation_code(&db, "INV-COUNT-001").unwrap();
        assert_eq!(code_before.use_count, 0);

        router.handle_tool_call(
            "register_agent",
            serde_json::json!({
                "name": "CountBot",
                "purpose": "Test code consumption",
                "invitation_code": "INV-COUNT-001"
            }),
        ).unwrap();

        // After registration — use_count should be 1
        let code_after = queries::get_invitation_code(&db, "INV-COUNT-001").unwrap();
        assert_eq!(code_after.use_count, 1, "use_count must be incremented after registration");
    }

    /// Regression: invitation code with max_uses=1 must reject a second registration.
    #[test]
    fn test_invitation_code_max_uses_enforced() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db.clone(), &agent_id);

        // max_uses defaults to 1 in create_test_invitation
        let invitation = create_test_invitation("INV-ONCE-001", "Single use");
        queries::insert_invitation_code(&db, &invitation).unwrap();

        // First registration succeeds
        router.handle_tool_call(
            "register_agent",
            serde_json::json!({
                "name": "FirstBot",
                "purpose": "First use",
                "invitation_code": "INV-ONCE-001"
            }),
        ).unwrap();

        // Second registration with same code must fail
        let result = router.handle_tool_call(
            "register_agent",
            serde_json::json!({
                "name": "SecondBot",
                "purpose": "Second use",
                "invitation_code": "INV-ONCE-001"
            }),
        );
        assert!(result.is_err(), "Second use of max_uses=1 code must fail");
    }

    /// Regression: trade_tokens CLI failure must rollback reservation.
    #[test]
    fn test_trade_cli_failure_rolls_back_reservation() {
        use crate::cli::executor::{CliError, CliExecutable, CliOutput};

        struct FailingCli;

        #[async_trait::async_trait]
        impl CliExecutable for FailingCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Err(CliError::CommandFailed {
                    stderr: "trade failed".to_string(),
                    exit_code: Some(1),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);

        // Fail a trade
        let failing_router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(FailingCli));
        let result = failing_router.handle_tool_call(
            "trade_tokens",
            serde_json::json!({ "from_asset": "ETH", "to_asset": "USDC", "amount": "40" }),
        ).unwrap();
        assert_eq!(result["status"], "failed");

        // Subsequent trade should succeed (reservation was rolled back)
        let router = make_router(db.clone(), &agent_id);
        let result2 = router.handle_tool_call(
            "trade_tokens",
            serde_json::json!({ "from_asset": "ETH", "to_asset": "USDC", "amount": "40" }),
        ).unwrap();
        assert_eq!(result2["status"], "pending");
    }

    /// Regression: pay_x402 CLI failure must rollback reservation and store error.
    #[test]
    fn test_x402_cli_failure_rolls_back_and_stores_error() {
        use crate::cli::executor::{CliError, CliExecutable, CliOutput};

        struct FailingCli;

        #[async_trait::async_trait]
        impl CliExecutable for FailingCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Err(CliError::CommandFailed {
                    stderr: "x402 payment rejected".to_string(),
                    exit_code: Some(1),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);

        let failing_router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(FailingCli));
        let result = failing_router.handle_tool_call(
            "pay_x402",
            serde_json::json!({ "url": "https://example.com/api", "max_amount": "30" }),
        ).unwrap();
        assert_eq!(result["status"], "failed");

        // Verify error stored (sanitized after BUG-1 fix)
        let tx_id = result["tx_id"].as_str().unwrap();
        let tx = queries::get_transaction(&db, tx_id).unwrap();
        assert!(tx.error_message.is_some());
        let err_msg = tx.error_message.unwrap();
        assert!(
            err_msg.contains("Wallet operation failed") || err_msg.contains("x402 payment rejected"),
            "Error message should be sanitized, got: {}", err_msg
        );

        // Subsequent x402 should succeed (reservation rolled back)
        let router = make_router(db.clone(), &agent_id);
        let result2 = router.handle_tool_call(
            "pay_x402",
            serde_json::json!({ "url": "https://example.com/api", "max_amount": "30" }),
        ).unwrap();
        assert_eq!(result2["status"], "pending");
    }

    // =====================================================================
    // BUG-1, BUG-2, BUG-3 — TDD tests (written first, then implemented)
    // =====================================================================

    /// A valid Ethereum address for use in tests (42 chars: 0x + 40 hex).
    const VALID_ETH_ADDR: &str = "0x1234567890abcdef1234567890abcdef12345678";

    // -- BUG-2: Negative/invalid amount validation --

    #[test]
    fn test_send_payment_negative_amount() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": VALID_ETH_ADDR, "amount": "-1.0" }),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("positive"), "Error should mention 'positive', got: {}", msg),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn test_send_payment_zero_amount() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": VALID_ETH_ADDR, "amount": "0" }),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("positive"), "Error should mention 'positive', got: {}", msg),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn test_send_payment_non_numeric_amount() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": VALID_ETH_ADDR, "amount": "abc" }),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("Invalid amount"), "Error should mention 'Invalid amount', got: {}", msg),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    // -- BUG-3: Ethereum address validation --

    #[test]
    fn test_send_payment_invalid_address_no_prefix() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "1234567890abcdef1234567890abcdef12345678", "amount": "10" }),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("Invalid Ethereum address"), "Got: {}", msg),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn test_send_payment_invalid_address_short() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x1234", "amount": "10" }),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("Invalid Ethereum address"), "Got: {}", msg),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    #[test]
    fn test_send_payment_invalid_address_non_hex() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = make_router(db, &agent_id);

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG", "amount": "10" }),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(msg) => assert!(msg.contains("Invalid Ethereum address"), "Got: {}", msg),
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    // -- BUG-1: CLI error sanitization --

    #[test]
    fn test_cli_error_sanitized() {
        use crate::cli::executor::{CliError, CliExecutable, CliOutput};

        /// A CLI that fails with a message containing internal CLI details.
        struct LeakyCli;

        #[async_trait::async_trait]
        impl CliExecutable for LeakyCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Err(CliError::CommandFailed {
                    stderr: "npx awal auth login failed: ECONNREFUSED 127.0.0.1:3000".to_string(),
                    exit_code: Some(1),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(LeakyCli));

        // send_payment exposes the CLI error in the tx record
        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": VALID_ETH_ADDR, "amount": "25" }),
        ).unwrap();

        // The tx should be "failed" but the error should be sanitized
        assert_eq!(result["status"], "failed");

        // Verify the error_message field does NOT leak raw CLI internals
        let tx_id = result["tx_id"].as_str().unwrap();
        let tx = queries::get_transaction(&db, tx_id).unwrap();
        let err_msg = tx.error_message.unwrap_or_default();
        // The sanitized error should contain the generic message
        assert!(
            err_msg.contains("Wallet operation failed") || err_msg.contains("re-authenticate"),
            "Error message should be sanitized, got: {}", err_msg
        );
        // It should NOT contain the raw CLI command/output
        assert!(
            !err_msg.contains("npx awal"),
            "Sanitized error must not leak CLI command, got: {}", err_msg
        );
    }

    // -- BUG-1: error_code mapping for CliError --

    #[test]
    fn test_error_code_cli_error() {
        assert_eq!(error_code(&AppError::CliError("x".into())), -32006);
    }

    // =====================================================================
    // BUG-4/5/6 TDD tests: tx hash propagation, x402 filtering, response body
    // =====================================================================

    /// BUG-4: CLI returns tx hash under "transaction_hash" key instead of "tx_hash".
    /// The router should try multiple key names.
    #[tokio::test]
    async fn test_send_payment_cli_returns_transaction_hash() {
        use crate::cli::executor::{CliExecutable, CliOutput, CliError};

        struct TransactionHashCli;

        #[async_trait::async_trait]
        impl CliExecutable for TransactionHashCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Ok(CliOutput {
                    success: true,
                    data: serde_json::json!({"transaction_hash": "0xalt_hash_abc"}),
                    raw: r#"{"transaction_hash":"0xalt_hash_abc"}"#.to_string(),
                    stderr: String::new(),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(TransactionHashCli));

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x1234567890abcdef1234567890abcdef12345678", "amount": "25" }),
        ).unwrap();

        assert_eq!(
            result["status"], "confirmed",
            "CLI returning 'transaction_hash' should be recognized as confirmed"
        );
        assert_eq!(
            result["chain_tx_hash"], "0xalt_hash_abc",
            "chain_tx_hash should be extracted from 'transaction_hash' key"
        );
    }

    /// BUG-4: CLI returns tx hash nested under "transaction.hash".
    /// The router should try nested lookup.
    #[tokio::test]
    async fn test_send_payment_cli_returns_nested_hash() {
        use crate::cli::executor::{CliExecutable, CliOutput, CliError};

        struct NestedHashCli;

        #[async_trait::async_trait]
        impl CliExecutable for NestedHashCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Ok(CliOutput {
                    success: true,
                    data: serde_json::json!({"transaction": {"hash": "0xnested_hash_def"}}),
                    raw: r#"{"transaction":{"hash":"0xnested_hash_def"}}"#.to_string(),
                    stderr: String::new(),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(NestedHashCli));

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x1234567890abcdef1234567890abcdef12345678", "amount": "25" }),
        ).unwrap();

        assert_eq!(result["status"], "confirmed");
        assert_eq!(result["chain_tx_hash"], "0xnested_hash_def");
    }

    /// BUG-4: CLI returns no hash at all -- should remain Pending.
    #[tokio::test]
    async fn test_send_payment_cli_no_hash() {
        use crate::cli::executor::{CliExecutable, CliOutput, CliError};

        struct NoHashCli;

        #[async_trait::async_trait]
        impl CliExecutable for NoHashCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Ok(CliOutput {
                    success: true,
                    data: serde_json::json!({"message": "submitted"}),
                    raw: r#"{"message":"submitted"}"#.to_string(),
                    stderr: String::new(),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(NoHashCli));

        let result = router.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x1234567890abcdef1234567890abcdef12345678", "amount": "25" }),
        ).unwrap();

        assert_eq!(result["status"], "pending", "No hash means status should be pending");
        assert!(result["chain_tx_hash"].is_null(), "chain_tx_hash should be null when no hash returned");
    }

    /// BUG-5: search_x402_services should filter results client-side.
    #[tokio::test]
    async fn test_search_x402_filters_results() {
        use crate::cli::executor::{CliExecutable, CliOutput, CliError};

        struct UnfilteredSearchCli;

        #[async_trait::async_trait]
        impl CliExecutable for UnfilteredSearchCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                // CLI returns 3 services but only 1 matches "weather"
                Ok(CliOutput {
                    success: true,
                    data: serde_json::json!({
                        "services": [
                            { "name": "Weather API", "description": "Real-time weather data", "resource": "https://weather.x402.org" },
                            { "name": "News Feed", "description": "Latest news articles", "resource": "https://news.x402.org" },
                            { "name": "Stock Prices", "description": "Market data service", "resource": "https://stocks.x402.org" }
                        ]
                    }),
                    raw: "{}".to_string(),
                    stderr: String::new(),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(UnfilteredSearchCli));

        let result = router.handle_tool_call(
            "search_x402_services",
            serde_json::json!({ "query": "weather" }),
        ).unwrap();

        let services = result["services"].as_array().expect("services should be an array");
        assert_eq!(services.len(), 1, "Only 1 of 3 services should match 'weather', got {}", services.len());
        assert_eq!(services[0]["name"], "Weather API");
        assert_eq!(result["total"], 1);
    }

    /// BUG-6: pay_x402 should include response_body, amount_paid, and response_status
    /// from the CLI output in the MCP response.
    #[tokio::test]
    async fn test_pay_x402_includes_response_body() {
        use crate::cli::executor::{CliExecutable, CliOutput, CliError};

        struct X402WithBodyCli;

        #[async_trait::async_trait]
        impl CliExecutable for X402WithBodyCli {
            async fn run(&self, _cmd: crate::cli::commands::AwalCommand) -> Result<CliOutput, CliError> {
                Ok(CliOutput {
                    success: true,
                    data: serde_json::json!({
                        "tx_hash": "0xx402_hash_123",
                        "response_body": {"data": "hello", "status": "ok"},
                        "amount_paid": "0.50",
                        "response_status": 200
                    }),
                    raw: "{}".to_string(),
                    stderr: String::new(),
                })
            }
        }

        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let router = McpRouter::new_with_cli(db.clone(), agent_id.clone(), Arc::new(X402WithBodyCli));

        let result = router.handle_tool_call(
            "pay_x402",
            serde_json::json!({ "url": "https://example.com/api", "max_amount": "10" }),
        ).unwrap();

        assert_eq!(result["status"], "confirmed");
        assert_eq!(result["chain_tx_hash"], "0xx402_hash_123");
        assert_eq!(result["response_body"]["data"], "hello", "response_body should be propagated");
        assert_eq!(result["amount_paid"], "0.50", "amount_paid should be propagated");
        assert_eq!(result["response_status"], 200, "response_status should be propagated");
    }
}
