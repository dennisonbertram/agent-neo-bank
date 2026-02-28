//! Transport-agnostic MCP request router.
//!
//! `McpRouter` holds references to the shared database and dispatches
//! tool calls to the correct handler. Both the stdio-based `McpServer`
//! and the future HTTP-based MCP server use this router, avoiding code
//! duplication.

use std::sync::Arc;

use serde_json::Value;

use crate::core::spending_policy::{daily_period_key, monthly_period_key, weekly_period_key};
use crate::db::models::*;
use crate::db::queries;
use crate::db::queries::AtomicPolicyResult;
use crate::db::schema::Database;
use crate::error::AppError;

use super::mcp_tools::{get_tool_definitions, McpTool};

/// Transport-agnostic router that handles MCP tool calls.
///
/// The router is bound to a specific agent via `agent_id` and enforces
/// per-agent isolation for all queries and policy checks.
pub struct McpRouter {
    db: Arc<Database>,
    agent_id: String,
}

impl McpRouter {
    /// Create a new router bound to the given agent.
    ///
    /// The caller is responsible for authenticating the agent beforehand.
    pub fn new(db: Arc<Database>, agent_id: String) -> Self {
        Self { db, agent_id }
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
            _ => Err(AppError::NotFound(format!("Unknown tool: {}", tool_name))),
        }
    }

    // -----------------------------------------------------------------
    // Tool handlers
    // -----------------------------------------------------------------

    fn handle_send_payment(&self, arguments: Value) -> Result<Value, AppError> {
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

        let policy_result = queries::check_policy_and_reserve_atomic(
            &self.db,
            &self.agent_id,
            &amount,
            &to,
            "0",
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

    fn handle_register_agent(&self, arguments: Value) -> Result<Value, AppError> {
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
        assert_eq!(tools.len(), 6);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"send_payment"));
        assert!(names.contains(&"check_balance"));
        assert!(names.contains(&"register_agent"));
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
                    "to": "0x1234567890abcdef",
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
        assert_eq!(result["to"], "0x1234567890abcdef");

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
                        "to": format!("0x{:04}", i),
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
                serde_json::json!({ "to": "0xaaa", "amount": "10" }),
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
}
