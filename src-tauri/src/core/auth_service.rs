use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;

use crate::cli::commands::AwalCommand;
use crate::cli::executor::CliExecutable;
use crate::db::models::AgentStatus;
use crate::db::queries::list_agents_by_status;
use crate::db::schema::Database;
use crate::error::AppError;

// -------------------------------------------------------------------------
// Types
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthResult {
    OtpSent { flow_id: String },
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub email: Option<String>,
}

struct CachedToken {
    agent_id: String,
    cached_at: Instant,
}

// -------------------------------------------------------------------------
// AuthService
// -------------------------------------------------------------------------

pub struct AuthService {
    cli: Arc<dyn CliExecutable>,
    db: Arc<Database>,
    token_cache: RwLock<HashMap<String, CachedToken>>,
    cache_ttl: Duration,
    current_flow_id: RwLock<Option<String>>,
    current_email: RwLock<Option<String>>,
}

impl AuthService {
    pub fn new(cli: Arc<dyn CliExecutable>, db: Arc<Database>, cache_ttl: Duration) -> Self {
        Self {
            cli,
            db,
            token_cache: RwLock::new(HashMap::new()),
            cache_ttl,
            current_flow_id: RwLock::new(None),
            current_email: RwLock::new(None),
        }
    }

    /// Start OTP login flow. Calls CLI `auth login <email>` and returns flow_id.
    pub async fn login(&self, email: &str) -> Result<AuthResult, AppError> {
        let output = self
            .cli
            .run(AwalCommand::AuthLogin {
                email: email.to_string(),
            })
            .await
            .map_err(|e| AppError::CliError(e.to_string()))?;

        if !output.success {
            return Err(AppError::AuthError(format!(
                "Login failed: {}",
                output.stderr
            )));
        }

        let flow_id = output.data["flowId"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // Store flow_id and email for the verify step
        *self.current_flow_id.write().await = Some(flow_id.clone());
        *self.current_email.write().await = Some(email.to_string());

        Ok(AuthResult::OtpSent { flow_id })
    }

    /// Verify OTP code. Uses the stored flow_id from the login step.
    /// Real CLI expects: `awal auth verify <flowId> <otp> --json`
    pub async fn verify(&self, otp: &str) -> Result<AuthResult, AppError> {
        let flow_id = self
            .current_flow_id
            .read()
            .await
            .clone()
            .ok_or_else(|| AppError::AuthError("No active login flow".to_string()))?;

        let output = self
            .cli
            .run(AwalCommand::AuthVerify {
                flow_id,
                otp: otp.to_string(),
            })
            .await
            .map_err(|e| AppError::CliError(e.to_string()))?;

        if !output.success {
            return Err(AppError::InvalidOtp);
        }

        // Check for explicit verification failure in the response data
        if let Some(success) = output.data.get("success") {
            if success == false {
                return Err(AppError::InvalidOtp);
            }
        }

        // Clear the flow_id after successful verification
        *self.current_flow_id.write().await = None;

        Ok(AuthResult::Verified)
    }

    /// Check current auth status by calling CLI `status`.
    /// Real CLI returns nested format: `{ "server": {...}, "auth": { "authenticated": ..., "email": ... } }`
    pub async fn check_status(&self) -> Result<AuthStatus, AppError> {
        let output = self
            .cli
            .run(AwalCommand::AuthStatus)
            .await
            .map_err(|e| AppError::CliError(e.to_string()))?;

        // Try nested format first (real CLI), then fall back to flat format (legacy)
        let authenticated = output
            .data
            .get("auth")
            .and_then(|a| a.get("authenticated"))
            .and_then(|v| v.as_bool())
            .or_else(|| output.data.get("authenticated").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        let email = output
            .data
            .get("auth")
            .and_then(|a| a.get("email"))
            .and_then(|v| v.as_str())
            .or_else(|| output.data.get("email").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        Ok(AuthStatus {
            authenticated,
            email,
        })
    }

    /// Validate an agent bearer token using two-tier lookup:
    /// 1. SHA-256 hash -> in-memory cache (fast path)
    /// 2. Cache miss -> argon2 verify against all active agents in DB (slow path)
    pub async fn validate_agent_token(&self, token: &str) -> Result<String, AppError> {
        let sha256_hex = sha256_hex(token);

        // Fast path: check cache
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = cache.get(&sha256_hex) {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    return Ok(cached.agent_id.clone());
                }
                // Expired -- fall through to argon2
            }
        }

        // Slow path: query DB for active agents with token hashes
        let db = self.db.clone();
        let agents = tokio::task::spawn_blocking(move || {
            list_agents_by_status(&db, &AgentStatus::Active)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Check each agent's argon2 hash
        let token_bytes = token.as_bytes().to_vec();
        for agent in &agents {
            if let Some(ref hash_str) = agent.api_token_hash {
                let hash_str = hash_str.clone();
                let token_bytes = token_bytes.clone();
                let valid = tokio::task::spawn_blocking(move || {
                    use argon2::password_hash::PasswordHash;
                    use argon2::Argon2;
                    use argon2::PasswordVerifier;
                    match PasswordHash::new(&hash_str) {
                        Ok(parsed) => Argon2::default()
                            .verify_password(&token_bytes, &parsed)
                            .is_ok(),
                        Err(_) => false,
                    }
                })
                .await
                .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?;

                if valid {
                    // Check agent is not suspended/revoked (already filtered to Active, but defensive)
                    if agent.status == AgentStatus::Suspended {
                        return Err(AppError::AgentSuspended(format!(
                            "Agent {} is suspended",
                            agent.id
                        )));
                    }
                    if agent.status == AgentStatus::Revoked {
                        return Err(AppError::InvalidToken);
                    }

                    // Add to cache
                    let mut cache = self.token_cache.write().await;
                    cache.insert(
                        sha256_hex,
                        CachedToken {
                            agent_id: agent.id.clone(),
                            cached_at: Instant::now(),
                        },
                    );

                    return Ok(agent.id.clone());
                }
            }
        }

        Err(AppError::InvalidToken)
    }

    /// Logout by calling CLI `auth logout` and clearing local state.
    pub async fn logout(&self) -> Result<(), AppError> {
        self.cli
            .run(AwalCommand::AuthLogout)
            .await
            .map_err(|e| AppError::CliError(e.to_string()))?;

        *self.current_flow_id.write().await = None;
        *self.current_email.write().await = None;

        Ok(())
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::executor::{CliOutput, MockCliExecutor};
    use crate::db::queries::insert_agent;
    use crate::test_helpers::{create_test_agent, setup_test_db};

    fn make_auth_service(
        mock: Arc<MockCliExecutor>,
        db: Arc<Database>,
    ) -> AuthService {
        AuthService::new(mock, db, Duration::from_secs(300))
    }

    #[tokio::test]
    async fn test_auth_otp_login_calls_cli() {
        let mock = Arc::new(MockCliExecutor::new());
        mock.set_response(
            "auth_login",
            CliOutput {
                success: true,
                data: serde_json::json!({"flowId": "flow-abc-123"}),
                raw: r#"{"flowId": "flow-abc-123"}"#.to_string(),
                stderr: String::new(),
            },
        );
        let db = setup_test_db();
        let auth = make_auth_service(mock, db);

        let result = auth.login("user@example.com").await.unwrap();
        match result {
            AuthResult::OtpSent { flow_id } => {
                assert_eq!(flow_id, "flow-abc-123");
            }
            _ => panic!("Expected OtpSent"),
        }
    }

    #[tokio::test]
    async fn test_auth_otp_verify_success() {
        let mock = Arc::new(MockCliExecutor::new());
        mock.set_response(
            "auth_login",
            CliOutput {
                success: true,
                data: serde_json::json!({"flowId": "flow-123", "message": "Verification code sent..."}),
                raw: "{}".to_string(),
                stderr: String::new(),
            },
        );
        mock.set_response(
            "auth_verify",
            CliOutput {
                success: true,
                data: serde_json::json!({"success": true, "message": "Successfully signed in."}),
                raw: r#"{"success": true, "message": "Successfully signed in."}"#.to_string(),
                stderr: String::new(),
            },
        );
        let db = setup_test_db();
        let auth = make_auth_service(mock, db);

        // Must login first to set flow_id
        auth.login("user@example.com").await.unwrap();

        let result = auth.verify("123456").await.unwrap();
        match result {
            AuthResult::Verified => {}
            _ => panic!("Expected Verified"),
        }
    }

    #[tokio::test]
    async fn test_auth_otp_verify_invalid_code() {
        let mock = Arc::new(MockCliExecutor::new());
        mock.set_response(
            "auth_login",
            CliOutput {
                success: true,
                data: serde_json::json!({"flowId": "flow-123", "message": "Verification code sent..."}),
                raw: "{}".to_string(),
                stderr: String::new(),
            },
        );
        mock.set_response(
            "auth_verify",
            CliOutput {
                success: false,
                data: serde_json::json!({}),
                raw: String::new(),
                stderr: "Invalid OTP code".to_string(),
            },
        );
        let db = setup_test_db();
        let auth = make_auth_service(mock, db);

        auth.login("user@example.com").await.unwrap();

        let result = auth.verify("000000").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidOtp => {}
            other => panic!("Expected InvalidOtp, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_check_status_authenticated() {
        let mock = Arc::new(MockCliExecutor::new());
        mock.set_response(
            "auth_status",
            CliOutput {
                success: true,
                data: serde_json::json!({
                    "server": { "running": true, "pid": 12345 },
                    "auth": { "authenticated": true, "email": "test@example.com" }
                }),
                raw: r#"{"server":{"running":true},"auth":{"authenticated":true,"email":"test@example.com"}}"#.to_string(),
                stderr: String::new(),
            },
        );
        let db = setup_test_db();
        let auth = make_auth_service(mock, db);

        let status = auth.check_status().await.unwrap();
        assert!(status.authenticated);
        assert_eq!(status.email.unwrap(), "test@example.com");
    }

    #[tokio::test]
    async fn test_auth_check_status_unauthenticated() {
        let mock = Arc::new(MockCliExecutor::new());
        mock.set_response(
            "auth_status",
            CliOutput {
                success: true,
                data: serde_json::json!({
                    "server": { "running": true, "pid": 12345 },
                    "auth": { "authenticated": false }
                }),
                raw: r#"{"server":{"running":true},"auth":{"authenticated":false}}"#.to_string(),
                stderr: String::new(),
            },
        );
        let db = setup_test_db();
        let auth = make_auth_service(mock, db);

        let status = auth.check_status().await.unwrap();
        assert!(!status.authenticated);
        assert!(status.email.is_none());
    }

    #[tokio::test]
    async fn test_auth_token_validation_sha256_cache_hit() {
        let mock = Arc::new(MockCliExecutor::new());
        let db = setup_test_db();
        let auth = make_auth_service(mock, db);

        // Manually populate cache
        let token = "anb_test_token_123";
        let sha_hex = sha256_hex(token);
        {
            let mut cache = auth.token_cache.write().await;
            cache.insert(
                sha_hex,
                CachedToken {
                    agent_id: "agent-001".to_string(),
                    cached_at: Instant::now(),
                },
            );
        }

        let result = auth.validate_agent_token(token).await.unwrap();
        assert_eq!(result, "agent-001");
    }

    #[tokio::test]
    async fn test_auth_token_validation_sha256_cache_miss_argon2_fallback() {
        let mock = Arc::new(MockCliExecutor::new());
        let db = setup_test_db();

        // Create an agent with a real argon2 hash
        let token = "anb_test_fallback_token";
        let argon2_hash = hash_token_argon2(token);
        let mut agent = create_test_agent("Argon2Bot", AgentStatus::Active);
        agent.api_token_hash = Some(argon2_hash);
        insert_agent(&db, &agent).unwrap();

        let auth = make_auth_service(mock, db);

        // Cache is empty, should fall through to argon2
        let result = auth.validate_agent_token(token).await.unwrap();
        assert_eq!(result, agent.id);

        // Verify it was cached
        let sha_hex = sha256_hex(token);
        let cache = auth.token_cache.read().await;
        assert!(cache.contains_key(&sha_hex));
    }

    #[tokio::test]
    async fn test_auth_token_validation_cache_expired_triggers_argon2() {
        let mock = Arc::new(MockCliExecutor::new());
        let db = setup_test_db();

        // Create an agent with a real argon2 hash
        let token = "anb_test_expired_cache_token";
        let argon2_hash = hash_token_argon2(token);
        let mut agent = create_test_agent("ExpiredCacheBot", AgentStatus::Active);
        agent.api_token_hash = Some(argon2_hash);
        insert_agent(&db, &agent).unwrap();

        // Use a very short TTL so cache expires immediately
        let auth = AuthService::new(
            mock,
            db,
            Duration::from_millis(0), // 0ms TTL = always expired
        );

        // Populate cache with an entry that will be expired
        let sha_hex = sha256_hex(token);
        {
            let mut cache = auth.token_cache.write().await;
            cache.insert(
                sha_hex,
                CachedToken {
                    agent_id: "stale-agent-id".to_string(),
                    cached_at: Instant::now() - Duration::from_secs(1),
                },
            );
        }

        // Should bypass expired cache and use argon2 to find the real agent
        let result = auth.validate_agent_token(token).await.unwrap();
        assert_eq!(result, agent.id);
    }

    #[tokio::test]
    async fn test_auth_token_validation_invalid_token() {
        let mock = Arc::new(MockCliExecutor::new());
        let db = setup_test_db();

        // Create an agent with a valid hash, but we'll use a wrong token
        let mut agent = create_test_agent("ValidHashBot", AgentStatus::Active);
        let correct_token = "anb_correct_token";
        agent.api_token_hash = Some(hash_token_argon2(correct_token));
        insert_agent(&db, &agent).unwrap();

        let auth = make_auth_service(mock, db);

        let result = auth.validate_agent_token("anb_wrong_token").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidToken => {}
            other => panic!("Expected InvalidToken, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_token_validation_suspended_agent_rejected() {
        let mock = Arc::new(MockCliExecutor::new());
        let db = setup_test_db();

        // Note: list_agents_by_status filters to Active only, so suspended
        // agents won't be returned. This means their tokens simply won't match.
        let token = "anb_suspended_token";
        let mut agent = create_test_agent("SuspendedBot", AgentStatus::Suspended);
        agent.api_token_hash = Some(hash_token_argon2(token));
        insert_agent(&db, &agent).unwrap();

        let auth = make_auth_service(mock, db);

        // Suspended agent's token should fail (not in Active list)
        let result = auth.validate_agent_token(token).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidToken => {}
            other => panic!("Expected InvalidToken, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_logout_clears_session() {
        let mock = Arc::new(MockCliExecutor::new());
        mock.set_response(
            "auth_login",
            CliOutput {
                success: true,
                data: serde_json::json!({"flowId": "flow-logout-test", "message": "Verification code sent..."}),
                raw: "{}".to_string(),
                stderr: String::new(),
            },
        );
        mock.set_response(
            "auth_logout",
            CliOutput {
                success: true,
                data: serde_json::json!({}),
                raw: "{}".to_string(),
                stderr: String::new(),
            },
        );
        let db = setup_test_db();
        let auth = make_auth_service(mock, db);

        // Login sets flow_id
        auth.login("user@example.com").await.unwrap();
        assert!(auth.current_flow_id.read().await.is_some());

        // Logout clears it
        auth.logout().await.unwrap();
        assert!(auth.current_flow_id.read().await.is_none());
        assert!(auth.current_email.read().await.is_none());
    }

    /// Helper: hash a token with argon2 for test purposes
    fn hash_token_argon2(token: &str) -> String {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::SaltString;
        use argon2::{Argon2, PasswordHasher};

        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(token.as_bytes(), &salt)
            .expect("Failed to hash token")
            .to_string()
    }
}
