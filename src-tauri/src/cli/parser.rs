use rust_decimal::Decimal;
use std::str::FromStr;

use super::executor::{CliError, CliOutput};

/// Result of parsing an auth status response.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthStatusResult {
    pub authenticated: bool,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
}

/// Parse a balance response from the CLI.
/// Expects `data` to contain `{"balance": "<decimal>", "asset": "<string>"}`.
pub fn parse_balance(output: &CliOutput) -> Result<(Decimal, String), CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: output.stderr.clone(),
            exit_code: None,
        });
    }

    let balance_str = output.data["balance"]
        .as_str()
        .ok_or_else(|| CliError::ParseError("Missing 'balance' field".into()))?;

    let balance = Decimal::from_str(balance_str)
        .map_err(|e| CliError::ParseError(format!("Invalid balance decimal: {}", e)))?;

    let asset = output.data["asset"]
        .as_str()
        .unwrap_or("USDC")
        .to_string();

    Ok((balance, asset))
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
/// Returns authentication state; detects session expiry.
pub fn parse_auth_status(output: &CliOutput) -> Result<AuthStatusResult, CliError> {
    if !output.success {
        return Err(CliError::CommandFailed {
            stderr: output.stderr.clone(),
            exit_code: None,
        });
    }

    let authenticated = output.data["authenticated"]
        .as_bool()
        .unwrap_or(false);

    if !authenticated {
        return Err(CliError::SessionExpired);
    }

    let email = output.data["email"].as_str().map(|s| s.to_string());
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
        let output = make_output(
            true,
            serde_json::json!({"balance": "1247.83", "asset": "USDC"}),
            "",
        );
        let (balance, asset) = parse_balance(&output).unwrap();
        assert_eq!(balance, Decimal::from_str("1247.83").unwrap());
        assert_eq!(asset, "USDC");
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
        let output = make_output(
            true,
            serde_json::json!({"authenticated": true, "email": "user@example.com"}),
            "",
        );
        let result = parse_auth_status(&output).unwrap();
        assert!(result.authenticated);
        assert_eq!(result.email, Some("user@example.com".to_string()));
        assert_eq!(result.wallet_address, None);
    }

    #[test]
    fn test_cli_parse_auth_status_unauthenticated() {
        let output = make_output(
            true,
            serde_json::json!({"authenticated": false}),
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
        // When authenticated is false, parse_auth_status returns SessionExpired
        let output = make_output(
            true,
            serde_json::json!({"authenticated": false}),
            "",
        );
        let result = parse_auth_status(&output);
        assert!(matches!(result, Err(CliError::SessionExpired)));
    }

    #[test]
    fn test_cli_parse_auth_status_with_wallet_address() {
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
    fn test_cli_parse_balance_invalid_decimal() {
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
}
