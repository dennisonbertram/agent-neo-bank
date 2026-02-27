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
            description: "Send a payment to a recipient address".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to": { "type": "string", "description": "Recipient address" },
                    "amount": { "type": "string", "description": "Amount to send (decimal string)" },
                    "asset": { "type": "string", "description": "Asset (default: USDC)" },
                    "memo": { "type": "string", "description": "Optional memo" }
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
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tool_definitions_returns_all_tools() {
        let tools = get_tool_definitions();
        assert_eq!(tools.len(), 6);

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
        assert_eq!(json.as_array().unwrap().len(), 6);
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
