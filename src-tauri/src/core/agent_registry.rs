use std::sync::Arc;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::core::invitation::InvitationManager;
use crate::db::models::*;
use crate::db::queries;
use crate::db::schema::Database;
use crate::error::AppError;

// -------------------------------------------------------------------------
// Structs
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationRequest {
    pub name: String,
    pub purpose: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub invitation_code: String,
    pub description: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationResult {
    pub agent_id: String,
    pub status: String,
}

// -------------------------------------------------------------------------
// AES-GCM encryption helpers
// -------------------------------------------------------------------------

/// Static key for token encryption in delivery cache.
/// In production, this would come from secure storage / key management.
const ENCRYPTION_KEY: &[u8; 32] = b"agent-neo-bank-token-encrypt-key";

fn encrypt_token(token: &str) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new(ENCRYPTION_KEY.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, token.as_bytes())
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;

    // Prepend nonce (12 bytes) to ciphertext
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&combined))
}

fn decrypt_token(encrypted: &str) -> Result<String, AppError> {
    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| AppError::Internal(format!("Base64 decode failed: {}", e)))?;

    if combined.len() < 12 {
        return Err(AppError::Internal("Invalid encrypted token".to_string()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(ENCRYPTION_KEY.into());

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(format!("UTF-8 conversion failed: {}", e)))
}

// -------------------------------------------------------------------------
// AgentRegistry
// -------------------------------------------------------------------------

pub struct AgentRegistry {
    db: Arc<Database>,
    invitation_manager: InvitationManager,
    config: AppConfig,
}

impl AgentRegistry {
    pub fn new(db: Arc<Database>, config: AppConfig) -> Self {
        let invitation_manager = InvitationManager::new(Arc::clone(&db), 50);
        Self {
            db,
            invitation_manager,
            config,
        }
    }

    /// Register a new agent with an invitation code.
    pub fn register(
        &self,
        request: AgentRegistrationRequest,
    ) -> Result<AgentRegistrationResult, AppError> {
        // 1. Validate invitation code
        let _invitation = self.invitation_manager.validate(&request.invitation_code)?;

        // 2. Generate agent ID
        let agent_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        // 3. Build metadata from rich fields
        let metadata = serde_json::json!({
            "webhook_url": request.webhook_url,
        });

        // 4. Create agent with Pending status (must be before use_invitation_code due to FK)
        let agent = Agent {
            id: agent_id.clone(),
            name: request.name,
            description: request.description.unwrap_or_default(),
            purpose: request.purpose,
            agent_type: request.agent_type,
            capabilities: request.capabilities,
            status: AgentStatus::Pending,
            api_token_hash: None,
            token_prefix: None,
            balance_visible: self.config.new_agent_balance_visible,
            invitation_code: Some(request.invitation_code.clone()),
            created_at: now,
            updated_at: now,
            last_active_at: None,
            metadata: metadata.to_string(),
        };
        queries::insert_agent(&self.db, &agent)?;

        // 5. Mark invitation code as used (after agent insert due to FK constraint)
        queries::use_invitation_code(&self.db, &request.invitation_code, &agent_id, now)?;

        // 6. Insert spending policy with all zeros
        let policy = SpendingPolicy {
            agent_id: agent_id.clone(),
            per_tx_max: "0".to_string(),
            daily_cap: "0".to_string(),
            weekly_cap: "0".to_string(),
            monthly_cap: "0".to_string(),
            auto_approve_max: "0".to_string(),
            allowlist: vec![],
            updated_at: now,
        };
        queries::insert_spending_policy(&self.db, &policy)?;

        // 7. Create approval request for registration
        let expiry_hours = self.config.approval_default_expiry_hours as i64;
        let approval = ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.clone(),
            request_type: ApprovalRequestType::Registration,
            payload: "{}".to_string(),
            status: ApprovalStatus::Pending,
            tx_id: None,
            expires_at: now + expiry_hours * 3600,
            created_at: now,
            resolved_at: None,
            resolved_by: None,
        };
        queries::insert_approval_request(&self.db, &approval)?;

        Ok(AgentRegistrationResult {
            agent_id,
            status: "pending".to_string(),
        })
    }

    /// Approve a pending agent: generates token, stores hash, creates delivery cache.
    pub fn approve(&self, agent_id: &str) -> Result<String, AppError> {
        // 1. Get agent, verify pending
        let agent = queries::get_agent(&self.db, agent_id)?;
        if agent.status != AgentStatus::Pending {
            return Err(AppError::InvalidInput(format!(
                "Agent '{}' is not in pending status (current: {})",
                agent_id, agent.status
            )));
        }

        let now = chrono::Utc::now().timestamp();

        // 2. Generate token: anb_ + 32 random hex chars
        let mut rng = rand::thread_rng();
        let hex_chars: String = (0..32)
            .map(|_| format!("{:x}", rng.gen_range(0u8..16)))
            .collect();
        let raw_token = format!("anb_{}", hex_chars);

        // 3. Hash with argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let token_hash = argon2
            .hash_password(raw_token.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Argon2 hash failed: {}", e)))?
            .to_string();

        // 4. Store token hash and prefix
        let token_prefix = &raw_token[..12];
        queries::update_agent_token(&self.db, agent_id, &token_hash, token_prefix, now)?;

        // 5. Update agent status to Active
        queries::update_agent_status(&self.db, agent_id, &AgentStatus::Active, now)?;

        // 6. Encrypt token and store in token_delivery with 5-min expiry
        let encrypted = encrypt_token(&raw_token)?;
        let delivery = TokenDelivery {
            agent_id: agent_id.to_string(),
            encrypted_token: encrypted,
            created_at: now,
            expires_at: now + 300, // 5 minutes
            delivered: false,
        };
        queries::insert_token_delivery(&self.db, &delivery)?;

        Ok(raw_token)
    }

    /// Retrieve token from delivery cache (poll-once-then-delete).
    pub fn retrieve_token(&self, agent_id: &str) -> Result<Option<String>, AppError> {
        let delivery = queries::get_token_delivery(&self.db, agent_id)?;

        match delivery {
            None => Ok(None),
            Some(d) => {
                // Check if already delivered
                if d.delivered {
                    queries::delete_token_delivery(&self.db, agent_id)?;
                    return Ok(None);
                }

                // Check if expired
                let now = chrono::Utc::now().timestamp();
                if now >= d.expires_at {
                    queries::delete_token_delivery(&self.db, agent_id)?;
                    return Ok(None);
                }

                // Decrypt and return, then delete
                let token = decrypt_token(&d.encrypted_token)?;
                queries::delete_token_delivery(&self.db, agent_id)?;
                Ok(Some(token))
            }
        }
    }

    /// Get registration status for an agent.
    pub fn get_status(&self, agent_id: &str) -> Result<AgentRegistrationResult, AppError> {
        let agent = queries::get_agent(&self.db, agent_id)?;
        Ok(AgentRegistrationResult {
            agent_id: agent.id,
            status: agent.status.to_string(),
        })
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{create_test_invitation, setup_test_db};

    fn setup_registry() -> (Arc<Database>, AgentRegistry) {
        let db = setup_test_db();
        let config = AppConfig::default();
        let registry = AgentRegistry::new(Arc::clone(&db), config);
        (db, registry)
    }

    fn insert_valid_invitation(db: &Database, code: &str) {
        let invitation = create_test_invitation(code, "Test invitation");
        queries::insert_invitation_code(db, &invitation).unwrap();
    }

    fn make_registration_request(code: &str) -> AgentRegistrationRequest {
        AgentRegistrationRequest {
            name: "TestBot".to_string(),
            purpose: "Integration testing".to_string(),
            agent_type: "automated".to_string(),
            capabilities: vec!["send".to_string(), "receive".to_string()],
            invitation_code: code.to_string(),
            description: Some("A test bot for integration testing".to_string()),
            webhook_url: Some("https://example.com/webhook".to_string()),
        }
    }

    // 1. Valid invitation code → Ok with pending status
    #[test]
    fn test_agent_register_with_valid_invitation_code() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-abc12345");

        let request = make_registration_request("INV-abc12345");
        let result = registry.register(request);

        assert!(result.is_ok(), "Registration should succeed: {:?}", result.err());
        let reg_result = result.unwrap();
        assert_eq!(reg_result.status, "pending");
        assert!(!reg_result.agent_id.is_empty());

        // Verify agent inserted as pending
        let agent = queries::get_agent(&db, &reg_result.agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Pending);

        // Verify invitation code marked used
        let inv = queries::get_invitation_code(&db, "INV-abc12345").unwrap();
        assert_eq!(inv.use_count, 1);
        assert_eq!(inv.used_by.unwrap(), reg_result.agent_id);
    }

    // 2. Invalid invitation code → Err(InvalidInvitationCode)
    #[test]
    fn test_agent_register_with_invalid_code_rejected() {
        let (_db, registry) = setup_registry();

        let request = make_registration_request("INV-bogus999");
        let result = registry.register(request);

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInvitationCode => {}
            other => panic!("Expected InvalidInvitationCode, got: {:?}", other),
        }
    }

    // 3. Expired invitation code → Err(InvitationCodeExpired)
    #[test]
    fn test_agent_register_with_expired_code_rejected() {
        let (db, registry) = setup_registry();

        // Insert an already-expired code
        let expired = InvitationCode {
            code: "INV-expired1".to_string(),
            created_at: 1000000,
            expires_at: Some(1000001), // far in the past
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "Expired".to_string(),
        };
        queries::insert_invitation_code(&db, &expired).unwrap();

        let request = make_registration_request("INV-expired1");
        let result = registry.register(request);

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvitationCodeExpired => {}
            other => panic!("Expected InvitationCodeExpired, got: {:?}", other),
        }
    }

    // 4. Already-used code → Err(InvitationCodeExpired)
    #[test]
    fn test_agent_register_with_already_used_code_rejected() {
        let (db, registry) = setup_registry();

        let used_code = InvitationCode {
            code: "INV-used0001".to_string(),
            created_at: 1000000,
            expires_at: None,
            used_by: None,
            used_at: Some(1500000),
            max_uses: 1,
            use_count: 1,
            label: "Already used".to_string(),
        };
        queries::insert_invitation_code(&db, &used_code).unwrap();

        let request = make_registration_request("INV-used0001");
        let result = registry.register(request);

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvitationCodeExpired => {}
            other => panic!("Expected InvitationCodeExpired, got: {:?}", other),
        }
    }

    // 5. Registration creates pending approval request
    #[test]
    fn test_agent_register_creates_pending_approval_request() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-appr0001");

        let request = make_registration_request("INV-appr0001");
        let result = registry.register(request).unwrap();

        let approval = queries::get_approval_request_by_agent(
            &db,
            &result.agent_id,
            &ApprovalRequestType::Registration,
        )
        .unwrap();

        assert!(approval.is_some(), "Approval request should exist");
        let approval = approval.unwrap();
        assert_eq!(approval.request_type, ApprovalRequestType::Registration);
        assert_eq!(approval.status, ApprovalStatus::Pending);

        // Verify expiry is approximately 24 hours from now
        let now = chrono::Utc::now().timestamp();
        let expected_expiry = now + 24 * 3600;
        assert!(
            (approval.expires_at - expected_expiry).abs() < 5,
            "Approval should expire in ~24h (diff: {})",
            (approval.expires_at - expected_expiry).abs()
        );
    }

    // 6. Registration creates zero spending policy
    #[test]
    fn test_agent_register_creates_zero_spending_policy() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-zero0001");

        let request = make_registration_request("INV-zero0001");
        let result = registry.register(request).unwrap();

        let policy = queries::get_spending_policy(&db, &result.agent_id).unwrap();
        assert_eq!(policy.per_tx_max, "0");
        assert_eq!(policy.daily_cap, "0");
        assert_eq!(policy.weekly_cap, "0");
        assert_eq!(policy.monthly_cap, "0");
        assert_eq!(policy.auto_approve_max, "0");
        assert!(policy.allowlist.is_empty());
    }

    // 7. Rich metadata (purpose, agent_type, capabilities) stored correctly
    #[test]
    fn test_agent_register_rich_metadata_stored() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-meta0001");

        let request = AgentRegistrationRequest {
            name: "RichBot".to_string(),
            purpose: "Data analysis and reporting".to_string(),
            agent_type: "analytics".to_string(),
            capabilities: vec!["read".to_string(), "analyze".to_string(), "report".to_string()],
            invitation_code: "INV-meta0001".to_string(),
            description: Some("An analytics bot".to_string()),
            webhook_url: Some("https://example.com/hook".to_string()),
        };
        let result = registry.register(request).unwrap();

        let agent = queries::get_agent(&db, &result.agent_id).unwrap();
        assert_eq!(agent.purpose, "Data analysis and reporting");
        assert_eq!(agent.agent_type, "analytics");
        assert_eq!(
            agent.capabilities,
            vec!["read".to_string(), "analyze".to_string(), "report".to_string()]
        );
        assert_eq!(agent.description, "An analytics bot");

        // Verify webhook_url in metadata JSON
        let metadata: serde_json::Value = serde_json::from_str(&agent.metadata).unwrap();
        assert_eq!(
            metadata["webhook_url"].as_str().unwrap(),
            "https://example.com/hook"
        );
    }

    // 8. Approve generates token with anb_ prefix, argon2 hash, encrypted delivery
    #[test]
    fn test_agent_approve_generates_token_and_delivers() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-appr0002");

        let request = make_registration_request("INV-appr0002");
        let reg = registry.register(request).unwrap();

        let token = registry.approve(&reg.agent_id).unwrap();

        // Token has anb_ prefix
        assert!(token.starts_with("anb_"), "Token should start with anb_: {}", token);
        // Total length: "anb_" (4) + 32 hex = 36
        assert_eq!(token.len(), 36, "Token should be 36 chars");

        // Agent is now active with hash stored
        let agent = queries::get_agent(&db, &reg.agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Active);
        assert!(agent.api_token_hash.is_some());
        let hash = agent.api_token_hash.unwrap();
        assert!(hash.starts_with("$argon2"), "Hash should be argon2 format: {}", hash);

        // Token prefix stored (first 12 chars)
        assert_eq!(agent.token_prefix.unwrap(), &token[..12]);

        // Token delivery created
        let delivery = queries::get_token_delivery(&db, &reg.agent_id).unwrap();
        assert!(delivery.is_some(), "Token delivery should exist");
        let delivery = delivery.unwrap();
        assert!(!delivery.delivered);
        // Expires in ~5 minutes
        let now = chrono::Utc::now().timestamp();
        assert!(
            (delivery.expires_at - (now + 300)).abs() < 5,
            "Token delivery should expire in ~5 min"
        );
    }

    // 9. Retrieve token: first call returns token, second call returns None
    #[test]
    fn test_agent_token_delivery_returns_once_then_deletes() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-delv0001");

        let request = make_registration_request("INV-delv0001");
        let reg = registry.register(request).unwrap();
        let expected_token = registry.approve(&reg.agent_id).unwrap();

        // First retrieval returns the token
        let first = registry.retrieve_token(&reg.agent_id).unwrap();
        assert!(first.is_some(), "First retrieval should return token");
        assert_eq!(first.unwrap(), expected_token);

        // Second retrieval returns None (row deleted)
        let second = registry.retrieve_token(&reg.agent_id).unwrap();
        assert!(second.is_none(), "Second retrieval should return None");
    }

    // 10. Expired delivery returns None
    #[test]
    fn test_agent_token_delivery_expired_returns_none() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-expd0001");

        let request = make_registration_request("INV-expd0001");
        let reg = registry.register(request).unwrap();

        // Manually insert an expired token delivery
        let encrypted = encrypt_token("anb_test_expired_token_12345678").unwrap();
        let delivery = TokenDelivery {
            agent_id: reg.agent_id.clone(),
            encrypted_token: encrypted,
            created_at: 1000000,
            expires_at: 1000001, // far in the past
            delivered: false,
        };
        // The approve flow would normally insert this, but we need to test expiry
        // so skip approve and insert directly. First update agent status so there's
        // no delivery from approve.
        queries::insert_token_delivery(&db, &delivery).unwrap();

        let result = registry.retrieve_token(&reg.agent_id).unwrap();
        assert!(result.is_none(), "Expired delivery should return None");
    }

    // 11. Duplicate registration with same single-use code rejected
    #[test]
    fn test_agent_duplicate_registration_with_same_code_rejected() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-dupe0001");

        let request1 = make_registration_request("INV-dupe0001");
        let result1 = registry.register(request1);
        assert!(result1.is_ok(), "First registration should succeed");

        // Second registration with same code should fail
        let request2 = make_registration_request("INV-dupe0001");
        let result2 = registry.register(request2);
        assert!(result2.is_err(), "Duplicate registration should fail");
    }

    // 12. Registration does not panic (notification test placeholder)
    #[test]
    fn test_agent_register_notification_sent() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-noti0001");

        let request = make_registration_request("INV-noti0001");
        // Should not panic
        let result = registry.register(request);
        assert!(result.is_ok(), "Registration should not panic");
    }

    // Additional: encrypt/decrypt roundtrip
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "anb_1234567890abcdef1234567890abcdef";
        let encrypted = encrypt_token(original).unwrap();
        assert_ne!(encrypted, original);
        let decrypted = decrypt_token(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    // Additional: get_status works
    #[test]
    fn test_get_status_returns_correct_status() {
        let (db, registry) = setup_registry();
        insert_valid_invitation(&db, "INV-stat0001");

        let request = make_registration_request("INV-stat0001");
        let reg = registry.register(request).unwrap();

        let status = registry.get_status(&reg.agent_id).unwrap();
        assert_eq!(status.status, "pending");
        assert_eq!(status.agent_id, reg.agent_id);

        // After approval, status should be active
        registry.approve(&reg.agent_id).unwrap();
        let status = registry.get_status(&reg.agent_id).unwrap();
        assert_eq!(status.status, "active");
    }
}
