use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::executor::{CliError, CliOutput};

// -------------------------------------------------------------------------
// Balance types (real CLI format)
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub raw: String,
    pub formatted: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub address: String,
    pub chain: String,
    pub balances: HashMap<String, AssetBalance>,
    pub timestamp: String,
}

// -------------------------------------------------------------------------
// Auth status types (real CLI format)
// -------------------------------------------------------------------------

/// Result of parsing an auth status response.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthStatusResult {
    pub authenticated: bool,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
}

// -------------------------------------------------------------------------
// Login / Verify types (real CLI format)
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "flowId")]
    pub flow_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub success: bool,
    pub message: String,
}

// -------------------------------------------------------------------------
// Parsers
// -------------------------------------------------------------------------

/// Parse a balance response from the CLI.
/// Real CLI returns: `{ "address": "...", "chain": "...", "balances": { "USDC": { "raw": "...", "formatted": "...", "decimals": N }, ... }, "timestamp": "..." }`
pub fn parse_balance(output: &CliOutput) -> Result<BalanceResponse, CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: if output.stderr.is_empty() {
                output.raw.clone()
            } else {
                output.stderr.clone()
            },
            exit_code: None,
        });
    }

    serde_json::from_value(output.data.clone())
        .map_err(|e| CliError::ParseError(format!("Invalid balance response: {}", e)))
}

/// Parse a send result from the CLI.
/// Expects `data` to contain `{"tx_hash": "<string>"}`.
/// Returns the transaction hash.
pub fn parse_send_result(output: &CliOutput) -> Result<String, CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: output.stderr.clone(),
            exit_code: None,
        });
    }

    let tx_hash = output.data["tx_hash"]
        .as_str()
        .ok_or_else(|| CliError::ParseError("Missing 'tx_hash' field".into()))?;

    Ok(tx_hash.to_string())
}

/// Parse an auth status response from the CLI.
/// Real CLI returns: `{ "server": { "running": true, "pid": N }, "auth": { "authenticated": true, "email": "..." } }`
/// Also supports legacy flat format for backward compat.
pub fn parse_auth_status(output: &CliOutput) -> Result<AuthStatusResult, CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: output.stderr.clone(),
            exit_code: None,
        });
    }

    // Try nested format first (real CLI), then fall back to flat format (legacy)
    let authenticated = output.data["auth"]["authenticated"]
        .as_bool()
        .or_else(|| output.data["authenticated"].as_bool())
        .unwrap_or(false);

    if !authenticated {
        return Err(CliError::SessionExpired);
    }

    let email = output.data["auth"]["email"]
        .as_str()
        .or_else(|| output.data["email"].as_str())
        .map(|s| s.to_string());

    let wallet_address = output.data["wallet_address"]
        .as_str()
        .or_else(|| output.data["address"].as_str())
        .map(|s| s.to_string());

    Ok(AuthStatusResult {
        authenticated,
        email,
        wallet_address,
    })
}

/// Parse a bare address string from the CLI.
/// Real CLI returns: `"0x..."` (a bare JSON string, no object wrapper).
pub fn parse_address(output: &CliOutput) -> Result<String, CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: output.stderr.clone(),
            exit_code: None,
        });
    }

    // Try bare string first, then object format for backward compat
    output
        .data
        .as_str()
        .or_else(|| output.data["address"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| CliError::ParseError("Missing address in CLI response".into()))
}

/// Parse a login response from the CLI.
/// Real CLI returns: `{ "flowId": "...", "message": "..." }`
pub fn parse_login_response(output: &CliOutput) -> Result<LoginResponse, CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: output.stderr.clone(),
            exit_code: None,
        });
    }

    serde_json::from_value(output.data.clone())
        .map_err(|e| CliError::ParseError(format!("Invalid login response: {}", e)))
}

/// Parse a verify response from the CLI.
/// Real CLI returns: `{ "success": true, "message": "..." }`
pub fn parse_verify_response(output: &CliOutput) -> Result<VerifyResponse, CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: if output.stderr.is_empty() {
                output.raw.clone()
            } else {
                output.stderr.clone()
            },
            exit_code: None,
        });
    }

    serde_json::from_value(output.data.clone())
        .map_err(|e| CliError::ParseError(format!("Invalid verify response: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_output(success: bool, data: serde_json::Value, stderr: &str) -> CliOutput {
        let raw = serde_json::to_string(&data).unwrap_or_default();
        CliOutput {
            success,
            data,
            raw,
            stderr: stderr.to_string(),
        }
    }

    #[test]
    fn test_cli_parse_balance_output_success() {
        // Updated to use real CLI format
        let output = make_output(
            true,
            serde_json::json!({
                "address": "0xTest",
                "chain": "Base",
                "balances": {
                    "USDC": { "raw": "124783000000", "formatted": "1247.83", "decimals": 6 }
                },
                "timestamp": "2026-02-27T00:00:00.000Z"
            }),
            "",
        );
        let result = parse_balance(&output).unwrap();
        assert_eq!(result.balances["USDC"].formatted, "1247.83");
    }

    #[test]
    fn test_cli_parse_balance_missing_field() {
        let output = make_output(true, serde_json::json!({"something": "else"}), "");
        let result = parse_balance(&output);
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::ParseError(msg) => assert!(msg.contains("balance")),
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    #[test]
    fn test_cli_parse_send_output_with_tx_hash() {
        let output = make_output(
            true,
            serde_json::json!({"tx_hash": "0xabc123"}),
            "",
        );
        let tx_hash = parse_send_result(&output).unwrap();
        assert_eq!(tx_hash, "0xabc123");
    }

    #[test]
    fn test_cli_parse_send_missing_tx_hash() {
        let output = make_output(true, serde_json::json!({"status": "ok"}), "");
        let result = parse_send_result(&output);
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::ParseError(msg) => assert!(msg.contains("tx_hash")),
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    #[test]
    fn test_cli_parse_auth_status_authenticated() {
        // Updated to use real CLI nested format
        let output = make_output(
            true,
            serde_json::json!({
                "server": { "running": true, "pid": 12345 },
                "auth": { "authenticated": true, "email": "user@example.com" }
            }),
            "",
        );
        let result = parse_auth_status(&output).unwrap();
        assert!(result.authenticated);
        assert_eq!(result.email, Some("user@example.com".to_string()));
        assert_eq!(result.wallet_address, None);
    }

    #[test]
    fn test_cli_parse_auth_status_unauthenticated() {
        // Updated to use real CLI nested format
        let output = make_output(
            true,
            serde_json::json!({
                "server": { "running": true, "pid": 12345 },
                "auth": { "authenticated": false }
            }),
            "",
        );
        let result = parse_auth_status(&output);
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::SessionExpired => {}
            other => panic!("Expected SessionExpired, got: {:?}", other),
        }
    }

    #[test]
    fn test_cli_nonzero_exit_code_returns_error() {
        let output = make_output(
            false,
            serde_json::json!({}),
            "Error: command failed",
        );
        let result = parse_balance(&output);
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::CommandFailed { stderr, .. } => {
                assert_eq!(stderr, "Error: command failed");
            }
            other => panic!("Expected CommandFailed, got: {:?}", other),
        }
    }

    #[test]
    fn test_cli_session_expired_detected() {
        // When authenticated is false, parse_auth_status returns SessionExpired (nested format)
        let output = make_output(
            true,
            serde_json::json!({
                "server": { "running": true, "pid": 12345 },
                "auth": { "authenticated": false }
            }),
            "",
        );
        let result = parse_auth_status(&output);
        assert!(matches!(result, Err(CliError::SessionExpired)));
    }

    #[test]
    fn test_cli_parse_auth_status_with_wallet_address() {
        // Legacy flat format still supported for backward compat
        let output = make_output(
            true,
            serde_json::json!({
                "authenticated": true,
                "email": "user@example.com",
                "wallet_address": "0xWallet123"
            }),
            "",
        );
        let result = parse_auth_status(&output).unwrap();
        assert!(result.authenticated);
        assert_eq!(result.email, Some("user@example.com".to_string()));
        assert_eq!(result.wallet_address, Some("0xWallet123".to_string()));
    }

    #[test]
    fn test_cli_parse_balance_invalid_format() {
        // Invalid format: missing required fields for the real CLI format
        let output = make_output(
            true,
            serde_json::json!({"balance": "not_a_number", "asset": "USDC"}),
            "",
        );
        let result = parse_balance(&output);
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::ParseError(msg) => assert!(msg.contains("Invalid balance")),
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    // =====================================================================
    // NEW TDD TESTS: Real CLI format matching
    // =====================================================================

    #[test]
    fn test_parse_balance_real_format() {
        let data = serde_json::json!({
            "address": "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4",
            "chain": "Base",
            "balances": {
                "USDC": { "raw": "20000000", "formatted": "20.00", "decimals": 6 },
                "ETH": { "raw": "100000001000000000", "formatted": "0.10", "decimals": 18 },
                "WETH": { "raw": "100000001000000000", "formatted": "0.10", "decimals": 18 }
            },
            "timestamp": "2026-02-27T20:47:28.494Z"
        });
        let output = make_output(true, data, "");
        let result = parse_balance(&output).unwrap();
        assert_eq!(result.address, "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4");
        assert_eq!(result.chain, "Base");
        assert_eq!(result.balances.len(), 3);
        assert_eq!(result.balances["USDC"].formatted, "20.00");
        assert_eq!(result.balances["USDC"].decimals, 6);
        assert_eq!(result.balances["ETH"].formatted, "0.10");
        assert_eq!(result.balances["ETH"].decimals, 18);
    }

    #[test]
    fn test_parse_balance_zero_balances() {
        let data = serde_json::json!({
            "address": "0xABC",
            "chain": "Base Sepolia",
            "balances": {
                "USDC": { "raw": "0", "formatted": "0.00", "decimals": 6 },
                "ETH": { "raw": "0", "formatted": "0.00", "decimals": 18 },
                "WETH": { "raw": "0", "formatted": "0.00", "decimals": 18 }
            },
            "timestamp": "2026-02-27T20:48:10.932Z"
        });
        let output = make_output(true, data, "");
        let result = parse_balance(&output).unwrap();
        assert_eq!(result.balances["USDC"].formatted, "0.00");
        assert_eq!(result.balances["USDC"].raw, "0");
    }

    #[test]
    fn test_parse_balance_large_raw_values() {
        let data = serde_json::json!({
            "address": "0xABC",
            "chain": "Base",
            "balances": {
                "ETH": { "raw": "999999999999999999999", "formatted": "999.99", "decimals": 18 }
            },
            "timestamp": "2026-02-27T00:00:00.000Z"
        });
        let output = make_output(true, data, "");
        let result = parse_balance(&output).unwrap();
        assert_eq!(result.balances["ETH"].raw, "999999999999999999999");
    }

    #[test]
    fn test_parse_status_authenticated_nested() {
        let data = serde_json::json!({
            "server": { "running": true, "pid": 11705 },
            "auth": { "authenticated": true, "email": "user@example.com" }
        });
        let output = make_output(true, data, "");
        let result = parse_auth_status(&output).unwrap();
        assert!(result.authenticated);
        assert_eq!(result.email, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_parse_status_unauthenticated_nested() {
        let data = serde_json::json!({
            "server": { "running": true, "pid": 11705 },
            "auth": { "authenticated": false }
        });
        let output = make_output(true, data, "");
        let result = parse_auth_status(&output);
        assert!(matches!(result, Err(CliError::SessionExpired)));
    }

    #[test]
    fn test_parse_status_server_not_running() {
        let data = serde_json::json!({
            "server": { "running": false },
            "auth": { "authenticated": false }
        });
        let output = make_output(true, data, "");
        let result = parse_auth_status(&output);
        assert!(matches!(result, Err(CliError::SessionExpired)));
    }

    #[test]
    fn test_parse_address_bare_string() {
        let output = CliOutput {
            success: true,
            data: serde_json::Value::String("0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4".to_string()),
            raw: r#""0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4""#.to_string(),
            stderr: String::new(),
        };
        let addr = parse_address(&output).unwrap();
        assert_eq!(addr, "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4");
    }

    #[test]
    fn test_parse_login_response() {
        let data = serde_json::json!({
            "flowId": "c04d6529-2655-4e5f-bdd5-2d1f558b5d8f",
            "message": "Verification code sent to user@example.com..."
        });
        let output = make_output(true, data, "");
        let result = parse_login_response(&output).unwrap();
        assert_eq!(result.flow_id, "c04d6529-2655-4e5f-bdd5-2d1f558b5d8f");
        assert!(result.message.contains("Verification code"));
    }

    #[test]
    fn test_parse_verify_response_success() {
        let data = serde_json::json!({
            "success": true,
            "message": "Successfully signed in as user@example.com."
        });
        let output = make_output(true, data, "");
        let result = parse_verify_response(&output).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_parse_verify_response_failure() {
        let output = CliOutput {
            success: false,
            data: serde_json::json!({}),
            raw: "✖ Verification failed\nBridge communication error...".to_string(),
            stderr: String::new(),
        };
        let result = parse_verify_response(&output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_plain_text() {
        let output = CliOutput {
            success: false,
            data: serde_json::json!({"raw": "✖ Failed to fetch balances\nAuthentication required."}),
            raw: "✖ Failed to fetch balances\nAuthentication required.".to_string(),
            stderr: String::new(),
        };
        let result = parse_balance(&output);
        assert!(result.is_err());
        assert!(matches!(result, Err(CliError::CommandFailed { .. })));
    }
}
