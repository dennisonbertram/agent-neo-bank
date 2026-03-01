use serde::{Deserialize, Serialize};

/// Represents a tool that can be invoked via the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Returns the full list of tool definitions available to MCP agents.
pub fn get_tool_definitions() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "send_payment".to_string(),
            description: "Send USDC to an Ethereum address on Base".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to": { "type": "string", "description": "Recipient address" },
                    "amount": { "type": "string", "description": "Amount to send (decimal string)" }
                },
                "required": ["to", "amount"]
            }),
        },
        McpTool {
            name: "check_balance".to_string(),
            description: "Check wallet balance".to_string(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "get_spending_limits".to_string(),
            description: "Get current spending limits for this agent".to_string(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "request_limit_increase".to_string(),
            description: "Request an increase to spending limits".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "new_per_tx_max": { "type": "string" },
                    "new_daily_cap": { "type": "string" },
                    "reason": { "type": "string" }
                },
                "required": ["reason"]
            }),
        },
        McpTool {
            name: "get_transactions".to_string(),
            description: "Get recent transactions for this agent".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "Max results (default 10)" },
                    "status": { "type": "string", "description": "Filter by status" }
                }
            }),
        },
        McpTool {
            name: "register_agent".to_string(),
            description: "Register a new agent (requires invitation code)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "purpose": { "type": "string" },
                    "invitation_code": { "type": "string" }
                },
                "required": ["name", "purpose", "invitation_code"]
            }),
        },
        McpTool {
            name: "get_address".to_string(),
            description: "Get the wallet's public address. Use this to receive payments or verify identity.".to_string(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "trade_tokens".to_string(),
            description: "Swap tokens on Base network. Subject to your spending policy based on the source amount.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "from_asset": { "type": "string", "enum": ["ETH", "USDC", "WETH"], "description": "Source token" },
                    "to_asset": { "type": "string", "enum": ["ETH", "USDC", "WETH"], "description": "Destination token" },
                    "amount": { "type": "string", "description": "Amount of source token to swap" },
                    "slippage": { "type": "integer", "description": "Slippage tolerance in basis points (default: 100 = 1%)" }
                },
                "required": ["from_asset", "to_asset", "amount"]
            }),
        },
        McpTool {
            name: "pay_x402".to_string(),
            description: "Pay for an X402 service. The URL will be called and payment made automatically. Subject to spending policy.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "X402-enabled service URL" },
                    "max_amount": { "type": "string", "description": "Maximum amount willing to pay (required safety cap)" },
                    "method": { "type": "string", "description": "HTTP method (GET, POST, PUT, DELETE, PATCH). Default: GET", "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"] },
                    "data": { "type": "string", "description": "Request body as JSON string (for POST/PUT requests)" },
                    "headers": { "type": "string", "description": "Custom headers as JSON string" }
                },
                "required": ["url", "max_amount"]
            }),
        },
        McpTool {
            name: "list_x402_services".to_string(),
            description: "Browse available X402 services in the bazaar.".to_string(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        McpTool {
            name: "search_x402_services".to_string(),
            description: "Search the X402 bazaar for services matching a query.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search terms" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "get_x402_details".to_string(),
            description: "Get payment details for an X402 service before paying.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "X402 service URL" }
                },
                "required": ["url"]
            }),
        },
        McpTool {
            name: "get_agent_info".to_string(),
            description: "Get your agent profile information — name, status, and when you were created.".to_string(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tool_definitions_returns_all_tools() {
        let tools = get_tool_definitions();
        assert_eq!(tools.len(), 13);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"send_payment"));
        assert!(names.contains(&"check_balance"));
        assert!(names.contains(&"get_spending_limits"));
        assert!(names.contains(&"request_limit_increase"));
        assert!(names.contains(&"get_transactions"));
        assert!(names.contains(&"register_agent"));
    }

    #[test]
    fn test_tool_definitions_serialize_to_json() {
        let tools = get_tool_definitions();
        let json = serde_json::to_value(&tools).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 13);
    }

    #[test]
    fn test_send_payment_has_required_fields() {
        let tools = get_tool_definitions();
        let send = tools.iter().find(|t| t.name == "send_payment").unwrap();
        let required = send.input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("to")));
        assert!(required.contains(&serde_json::json!("amount")));
    }
}
