use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::spending_policy::{daily_period_key, weekly_period_key, monthly_period_key};
use crate::db::models::*;
use crate::db::queries;
use crate::db::queries::AtomicPolicyResult;
use crate::db::schema::Database;
use crate::error::AppError;

use super::mcp_tools::get_tool_definitions;

// -------------------------------------------------------------------------
// JSON-RPC types
// -------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

// -------------------------------------------------------------------------
// McpServer
// -------------------------------------------------------------------------

pub struct McpServer {
    db: Arc<Database>,
    agent_id: String,
}

impl std::fmt::Debug for McpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServer")
            .field("agent_id", &self.agent_id)
            .finish()
    }
}

impl McpServer {
    /// Validate an agent token by looking up agents in the DB and checking
    /// the token hash. For MCP, we use a simplified SHA-256 comparison
    /// (the token passed as CLI arg is the agent_id directly for simplicity,
    /// or we look up agents by iterating). In production, the MCP spawner
    /// passes a pre-validated agent_id.
    ///
    /// This constructor validates that the agent_id exists and is active.
    pub fn new_with_agent_id(db: Arc<Database>, agent_id: String) -> Result<Self, AppError> {
        // Verify the agent exists and is active
        let agent = queries::get_agent(&db, &agent_id)?;
        if agent.status != AgentStatus::Active {
            return Err(AppError::AuthError(format!(
                "Agent {} is not active (status: {})",
                agent_id, agent.status
            )));
        }
        Ok(Self { db, agent_id })
    }

    /// Validate a raw API token against agents in the DB.
    /// Uses SHA-256 hash comparison against stored token hashes.
    /// Returns an McpServer bound to the validated agent.
    pub fn validate_token(db: Arc<Database>, token: &str) -> Result<Self, AppError> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        // Look up active agents and find one whose stored hash matches
        let agents = queries::list_agents_by_status(&db, &AgentStatus::Active)?;
        for agent in &agents {
            if let Some(ref stored_hash) = agent.api_token_hash {
                // For MCP we compare SHA-256 hashes directly.
                // The auth_service uses argon2, but for the stdio MCP path
                // we support a simpler SHA-256 prefix check or exact match.
                if stored_hash == &token_hash {
                    return Ok(Self {
                        db,
                        agent_id: agent.id.clone(),
                    });
                }
            }
        }

        Err(AppError::InvalidToken)
    }

    /// Get the bound agent_id for this server instance.
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Handle a parsed JSON-RPC request and return a response.
    pub fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "agent-neo-bank-mcp",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            "tools/list" => {
                let tools = get_tool_definitions();
                Ok(serde_json::json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                match self.handle_tool_call(tool_name, arguments) {
                    Ok(content) => Ok(serde_json::json!({
                        "content": [{ "type": "text", "text": content.to_string() }]
                    })),
                    Err(e) => Err(e),
                }
            }
            _ => Err(AppError::NotFound(format!(
                "Unknown method: {}",
                request.method
            ))),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(value),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: error_code(&e),
                    message: e.to_string(),
                }),
            },
        }
    }

    /// Dispatch a tool call to the appropriate handler.
    pub fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        match tool_name {
            "send_payment" => self.handle_send_payment(arguments),
            "check_balance" => self.handle_check_balance(arguments),
            "get_spending_limits" => self.handle_get_spending_limits(arguments),
            "request_limit_increase" => self.handle_request_limit_increase(arguments),
            "get_transactions" => self.handle_get_transactions(arguments),
            "register_agent" => self.handle_register_agent(arguments),
            _ => Err(AppError::NotFound(format!("Unknown tool: {}", tool_name))),
        }
    }

    // ---------------------------------------------------------------------
    // Tool handlers
    // ---------------------------------------------------------------------

    fn handle_send_payment(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        let to = arguments
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'to' field".to_string()))?
            .to_string();
        let amount = arguments
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Missing 'amount' field".to_string()))?
            .to_string();
        let asset = arguments
            .get("asset")
            .and_then(|v| v.as_str())
            .unwrap_or("USDC")
            .to_string();
        let memo = arguments
            .get("memo")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let now = chrono::Utc::now();
        let now_ts = now.timestamp();
        let tx_id = uuid::Uuid::new_v4().to_string();

        let period_d = daily_period_key(&now);
        let period_w = weekly_period_key(&now);
        let period_m = monthly_period_key(&now);

        // Run atomic policy check + ledger reservation
        let policy_result = queries::check_policy_and_reserve_atomic(
            &self.db,
            &self.agent_id,
            &amount,
            &to,
            "0", // MCP doesn't track wallet balance; use 0 (min_reserve_balance check will be lenient)
            &period_d,
            &period_w,
            &period_m,
            now_ts,
        )?;

        match policy_result {
            AtomicPolicyResult::Denied { reason } => {
                // Insert a denied transaction for audit trail
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
                // Ledger already reserved; insert transaction as Pending (ready for execution)
                let tx = Transaction {
                    id: tx_id.clone(),
                    agent_id: Some(self.agent_id.clone()),
                    tx_type: TxType::Send,
                    amount: amount.clone(),
                    asset: asset.clone(),
                    recipient: Some(to.clone()),
                    sender: None,
                    chain_tx_hash: None,
                    status: TxStatus::Pending,
                    category: "payment".to_string(),
                    memo,
                    description: format!("MCP send {} {} to {}", amount, asset, to),
                    service_name: "mcp".to_string(),
                    service_url: String::new(),
                    reason: String::new(),
                    webhook_url: None,
                    error_message: None,
                    period_daily: period_d,
                    period_weekly: period_w,
                    period_monthly: period_m,
                    created_at: now_ts,
                    updated_at: now_ts,
                };
                queries::insert_transaction(&self.db, &tx)?;

                Ok(serde_json::json!({
                    "tx_id": tx_id,
                    "status": "pending",
                    "amount": amount,
                    "asset": asset,
                    "to": to
                }))
            }
            AtomicPolicyResult::RequiresApproval { reason } => {
                // Ledger already reserved; insert transaction as AwaitingApproval
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

                // Create an approval request
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

    fn handle_check_balance(
        &self,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        // Query the agent to check balance_visible flag
        let agent = queries::get_agent(&self.db, &self.agent_id)?;
        if !agent.balance_visible {
            return Ok(serde_json::json!({
                "balance": "hidden",
                "asset": "USDC",
                "message": "Balance visibility is disabled for this agent"
            }));
        }

        // Return a balance from the DB or a default.
        // In a full implementation this would query the wallet service.
        Ok(serde_json::json!({
            "balance": "0.00",
            "asset": "USDC"
        }))
    }

    fn handle_get_spending_limits(
        &self,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
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

    fn handle_request_limit_increase(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
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
            expires_at: now + 86400, // 24 hours
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

    fn handle_get_transactions(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
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

        let tx_summaries: Vec<serde_json::Value> = txs
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

    fn handle_register_agent(
        &self,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
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
            .ok_or_else(|| AppError::InvalidInput("Missing 'invitation_code' field".to_string()))?
            .to_string();

        // Validate the invitation code exists and has uses remaining
        let code = queries::get_invitation_code(&self.db, &invitation_code)?;
        if code.use_count >= code.max_uses {
            return Err(AppError::InvalidInvitationCode);
        }

        let now = chrono::Utc::now().timestamp();
        let agent_id = uuid::Uuid::new_v4().to_string();

        let agent = Agent {
            id: agent_id.clone(),
            name: name.clone(),
            description: String::new(),
            purpose,
            agent_type: "mcp".to_string(),
            capabilities: vec!["send".to_string()],
            status: AgentStatus::Pending,
            api_token_hash: None,
            token_prefix: None,
            balance_visible: true,
            invitation_code: Some(invitation_code),
            created_at: now,
            updated_at: now,
            last_active_at: None,
            metadata: "{}".to_string(),
        };

        queries::insert_agent(&self.db, &agent)?;

        Ok(serde_json::json!({
            "agent_id": agent_id,
            "name": name,
            "status": "pending",
            "message": "Agent registration submitted, pending approval"
        }))
    }

    /// Main stdio loop. Reads JSON-RPC requests from stdin line-by-line,
    /// processes them, and writes JSON-RPC responses to stdout.
    pub fn run(&self) -> Result<(), AppError> {
        use std::io::{self, BufRead, Write};

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line.map_err(|e| AppError::Internal(format!("stdin read error: {}", e)))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(req) => req,
                Err(e) => {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                        }),
                    };
                    let json = serde_json::to_string(&error_response)
                        .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"}}"#.to_string());
                    writeln!(stdout_lock, "{}", json)
                        .map_err(|e| AppError::Internal(format!("stdout write error: {}", e)))?;
                    stdout_lock
                        .flush()
                        .map_err(|e| AppError::Internal(format!("stdout flush error: {}", e)))?;
                    continue;
                }
            };

            let response = self.handle_request(&request);
            let json = serde_json::to_string(&response)
                .map_err(|e| AppError::Internal(format!("Serialization error: {}", e)))?;
            writeln!(stdout_lock, "{}", json)
                .map_err(|e| AppError::Internal(format!("stdout write error: {}", e)))?;
            stdout_lock
                .flush()
                .map_err(|e| AppError::Internal(format!("stdout flush error: {}", e)))?;
        }

        Ok(())
    }
}

/// Map AppError variants to JSON-RPC error codes.
fn error_code(err: &AppError) -> i32 {
    match err {
        AppError::NotFound(_) => -32601, // Method not found
        AppError::InvalidInput(_) => -32602, // Invalid params
        AppError::InvalidToken => -32000, // Server error: auth
        AppError::AuthError(_) => -32000,
        AppError::PolicyViolation(_) => -32001,
        AppError::KillSwitchActive(_) => -32002,
        AppError::AgentSuspended(_) => -32003,
        AppError::InvalidInvitationCode => -32004,
        AppError::RateLimited => -32005,
        _ => -32603, // Internal error
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

    fn make_test_server(db: Arc<Database>, agent_id: &str) -> McpServer {
        McpServer {
            db,
            agent_id: agent_id.to_string(),
        }
    }

    fn setup_agent_with_policy(db: &Arc<Database>) -> String {
        let agent = create_test_agent("TestBot", AgentStatus::Active);
        let agent_id = agent.id.clone();
        insert_agent(db, &agent).unwrap();

        let policy = create_test_spending_policy(&agent_id, "100", "1000", "5000", "20000", "50");
        insert_spending_policy(db, &policy).unwrap();

        agent_id
    }

    #[test]
    fn test_mcp_send_payment_tool_dispatch() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db.clone(), &agent_id);

        let result = server
            .handle_tool_call(
                "send_payment",
                serde_json::json!({
                    "to": "0x1234567890abcdef",
                    "amount": "25.50",
                    "asset": "USDC",
                    "memo": "test payment"
                }),
            )
            .unwrap();

        assert!(result.get("tx_id").is_some(), "Result should contain tx_id");
        assert_eq!(result["status"], "pending");
        assert_eq!(result["amount"], "25.50");
        assert_eq!(result["asset"], "USDC");
        assert_eq!(result["to"], "0x1234567890abcdef");

        // Verify the transaction was persisted
        let tx_id = result["tx_id"].as_str().unwrap();
        let tx = crate::db::queries::get_transaction(&db, tx_id).unwrap();
        assert_eq!(tx.agent_id.as_deref(), Some(agent_id.as_str()));
        assert_eq!(tx.amount, "25.50");
    }

    #[test]
    fn test_mcp_check_balance_response_shape() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let result = server
            .handle_tool_call("check_balance", serde_json::json!({}))
            .unwrap();

        assert!(
            result.get("balance").is_some(),
            "Result should contain balance"
        );
        assert!(result.get("asset").is_some(), "Result should contain asset");
        assert_eq!(result["asset"], "USDC");
    }

    #[test]
    fn test_mcp_per_agent_auth_validation() {
        let db = setup_test_db();

        // Create two agents
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

        // Server bound to agent_a should see agent_a's policy
        let server_a = make_test_server(db.clone(), &id_a);
        let limits_a = server_a
            .handle_tool_call("get_spending_limits", serde_json::json!({}))
            .unwrap();
        assert_eq!(limits_a["per_tx_max"], "100");
        assert_eq!(limits_a["daily_cap"], "1000");

        // Server bound to agent_b should see agent_b's policy
        let server_b = make_test_server(db.clone(), &id_b);
        let limits_b = server_b
            .handle_tool_call("get_spending_limits", serde_json::json!({}))
            .unwrap();
        assert_eq!(limits_b["per_tx_max"], "200");
        assert_eq!(limits_b["daily_cap"], "2000");

        // Transactions created by agent_a should not appear for agent_b
        server_a
            .handle_tool_call(
                "send_payment",
                serde_json::json!({ "to": "0xaaa", "amount": "10" }),
            )
            .unwrap();

        let txs_b = server_b
            .handle_tool_call("get_transactions", serde_json::json!({}))
            .unwrap();
        assert_eq!(
            txs_b["transactions"].as_array().unwrap().len(),
            0,
            "Agent B should not see Agent A's transactions"
        );
    }

    #[test]
    fn test_mcp_unknown_tool_returns_error() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let result = server.handle_tool_call("nonexistent_tool", serde_json::json!({}));
        assert!(result.is_err(), "Unknown tool should return an error");

        match result.unwrap_err() {
            AppError::NotFound(msg) => {
                assert!(msg.contains("nonexistent_tool"));
            }
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_mcp_invalid_token_fails_on_startup() {
        let db = setup_test_db();

        // No agents in DB -- any token should fail
        let result = McpServer::validate_token(db, "anb_invalid_token_xyz");
        assert!(result.is_err(), "Invalid token should fail validation");

        match result.unwrap_err() {
            AppError::InvalidToken => {}
            other => panic!("Expected InvalidToken, got: {:?}", other),
        }
    }

    #[test]
    fn test_mcp_get_spending_limits_response_shape() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let result = server
            .handle_tool_call("get_spending_limits", serde_json::json!({}))
            .unwrap();

        assert!(
            result.get("per_tx_max").is_some(),
            "Should have per_tx_max"
        );
        assert!(result.get("daily_cap").is_some(), "Should have daily_cap");
        assert!(
            result.get("weekly_cap").is_some(),
            "Should have weekly_cap"
        );
        assert!(
            result.get("monthly_cap").is_some(),
            "Should have monthly_cap"
        );
        assert!(
            result.get("auto_approve_max").is_some(),
            "Should have auto_approve_max"
        );
        assert!(
            result.get("allowlist").is_some(),
            "Should have allowlist"
        );

        assert_eq!(result["per_tx_max"], "100");
        assert_eq!(result["daily_cap"], "1000");
        assert_eq!(result["weekly_cap"], "5000");
        assert_eq!(result["monthly_cap"], "20000");
        assert_eq!(result["auto_approve_max"], "50");
    }

    #[test]
    fn test_mcp_handle_request_initialize() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let response = server.handle_request(&request);
        assert!(response.error.is_none(), "initialize should not error");
        let result = response.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "agent-neo-bank-mcp");
        assert!(result["capabilities"]["tools"].is_object());
    }

    #[test]
    fn test_mcp_handle_request_tools_list() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(&request);
        assert!(response.error.is_none(), "tools/list should not error");
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);
    }

    #[test]
    fn test_mcp_handle_request_tools_call() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(3)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "check_balance",
                "arguments": {}
            })),
        };

        let response = server.handle_request(&request);
        assert!(response.error.is_none(), "tools/call should not error");
        let result = response.result.unwrap();
        let content = result["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "text");
    }

    #[test]
    fn test_mcp_handle_request_unknown_method() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(99)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(&request);
        assert!(response.error.is_some(), "Unknown method should return error");
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[test]
    fn test_mcp_new_with_agent_id_validates_active() {
        let db = setup_test_db();

        // Active agent should succeed
        let agent = create_test_agent("ActiveBot", AgentStatus::Active);
        let agent_id = agent.id.clone();
        insert_agent(&db, &agent).unwrap();
        assert!(McpServer::new_with_agent_id(db.clone(), agent_id).is_ok());

        // Suspended agent should fail
        let suspended = create_test_agent("SuspendedBot", AgentStatus::Suspended);
        let suspended_id = suspended.id.clone();
        insert_agent(&db, &suspended).unwrap();
        let result = McpServer::new_with_agent_id(db.clone(), suspended_id);
        assert!(result.is_err());

        // Non-existent agent should fail
        let result = McpServer::new_with_agent_id(db, "nonexistent-id".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_get_transactions_with_limit() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db.clone(), &agent_id);

        // Create 5 transactions
        for i in 0..5 {
            server
                .handle_tool_call(
                    "send_payment",
                    serde_json::json!({
                        "to": format!("0x{:04}", i),
                        "amount": format!("{}.00", i + 1)
                    }),
                )
                .unwrap();
        }

        // Get with limit 3
        let result = server
            .handle_tool_call(
                "get_transactions",
                serde_json::json!({ "limit": 3 }),
            )
            .unwrap();

        let txs = result["transactions"].as_array().unwrap();
        assert_eq!(txs.len(), 3, "Should return at most 3 transactions");
        assert_eq!(result["total"], 5, "Total should be 5");
    }

    #[test]
    fn test_mcp_request_limit_increase() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db.clone(), &agent_id);

        let result = server
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
    fn test_mcp_register_agent_with_valid_code() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db.clone(), &agent_id);

        // Create an invitation code
        let invitation = create_test_invitation("INV-MCP-001", "MCP test invite");
        crate::db::queries::insert_invitation_code(&db, &invitation).unwrap();

        let result = server
            .handle_tool_call(
                "register_agent",
                serde_json::json!({
                    "name": "NewAgent",
                    "purpose": "Automated payments",
                    "invitation_code": "INV-MCP-001"
                }),
            )
            .unwrap();

        assert!(result.get("agent_id").is_some());
        assert_eq!(result["status"], "pending");
        assert_eq!(result["name"], "NewAgent");
    }

    #[test]
    fn test_mcp_send_payment_missing_required_fields() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        // Missing 'to' field
        let result = server.handle_tool_call(
            "send_payment",
            serde_json::json!({ "amount": "10" }),
        );
        assert!(result.is_err());

        // Missing 'amount' field
        let result = server.handle_tool_call(
            "send_payment",
            serde_json::json!({ "to": "0x123" }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_json_rpc_response_format() {
        let db = setup_test_db();
        let agent_id = setup_agent_with_policy(&db);
        let server = make_test_server(db, &agent_id);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(42)),
            method: "initialize".to_string(),
            params: None,
        };

        let response = server.handle_request(&request);
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(serde_json::json!(42)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        // Serialize and verify shape
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("jsonrpc").is_some());
        assert!(json.get("id").is_some());
        assert!(json.get("result").is_some());
        // error should be absent (skip_serializing_if)
        assert!(json.get("error").is_none());
    }
}
