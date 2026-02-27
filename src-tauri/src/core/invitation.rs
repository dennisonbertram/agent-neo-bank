use std::sync::Arc;

use rand::Rng;

use crate::db::models::InvitationCode;
use crate::db::queries;
use crate::db::schema::Database;
use crate::error::AppError;

/// Manages invitation code generation, validation, listing, and revocation.
pub struct InvitationManager {
    db: Arc<Database>,
    max_active_codes: usize,
}

impl InvitationManager {
    pub fn new(db: Arc<Database>, max_active_codes: usize) -> Self {
        Self {
            db,
            max_active_codes,
        }
    }

    /// Generate a new invitation code with format INV-[8 random lowercase alphanumeric].
    /// Stores it in the DB with max_uses=1.
    /// If `expires_in_hours` is Some, sets expires_at to now + hours.
    /// If None, expires_at is NULL (permanent code).
    pub fn generate(
        &self,
        label: &str,
        expires_in_hours: Option<u64>,
    ) -> Result<InvitationCode, AppError> {
        // Check active code limit
        let active_count = queries::count_active_invitation_codes(&self.db)?;
        if active_count >= self.max_active_codes {
            return Err(AppError::MaxActiveCodesReached);
        }

        let code = generate_code();
        let now = chrono::Utc::now().timestamp();
        let expires_at = expires_in_hours.map(|h| now + (h as i64) * 3600);

        let invitation = InvitationCode {
            code,
            created_at: now,
            expires_at,
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: label.to_string(),
        };

        queries::insert_invitation_code(&self.db, &invitation)?;
        Ok(invitation)
    }

    /// Validate an invitation code: checks existence, expiry, and use count.
    pub fn validate(&self, code: &str) -> Result<InvitationCode, AppError> {
        let invitation = match queries::get_invitation_code(&self.db, code) {
            Ok(inv) => inv,
            Err(AppError::NotFound(_)) => return Err(AppError::InvalidInvitationCode),
            Err(e) => return Err(e),
        };

        // Check if fully used
        if invitation.use_count >= invitation.max_uses {
            return Err(AppError::InvitationCodeExpired);
        }

        // Check expiry
        if let Some(expires_at) = invitation.expires_at {
            let now = chrono::Utc::now().timestamp();
            if now >= expires_at {
                return Err(AppError::InvitationCodeExpired);
            }
        }

        Ok(invitation)
    }

    /// List all active invitation codes (use_count < max_uses and not expired).
    pub fn list_active(&self) -> Result<Vec<InvitationCode>, AppError> {
        queries::list_active_invitation_codes(&self.db)
    }

    /// Revoke an invitation code by setting max_uses = use_count.
    pub fn revoke(&self, code: &str) -> Result<(), AppError> {
        queries::revoke_invitation_code(&self.db, code)
    }
}

/// Generate a random code in the format INV-[8 lowercase alphanumeric chars].
fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect();
    format!("INV-{}", chars.into_iter().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_db;
    use regex::Regex;

    #[test]
    fn test_invitation_code_generation_format() {
        let db = setup_test_db();
        let manager = InvitationManager::new(db, 50);

        let result = manager.generate("For Claude Code", Some(24));
        assert!(result.is_ok(), "generate should succeed");

        let invitation = result.unwrap();

        // Verify format: INV-[a-z0-9]{8}
        let re = Regex::new(r"^INV-[a-z0-9]{8}$").unwrap();
        assert!(
            re.is_match(&invitation.code),
            "Code '{}' should match INV-[a-z0-9]{{8}} format",
            invitation.code
        );

        // Verify max_uses is 1
        assert_eq!(invitation.max_uses, 1);

        // Verify expires_at is roughly 24h from now
        let now = chrono::Utc::now().timestamp();
        let expected_expiry = now + 24 * 3600;
        let expires_at = invitation.expires_at.expect("expires_at should be set");
        // Allow 5 seconds tolerance
        assert!(
            (expires_at - expected_expiry).abs() < 5,
            "expires_at should be ~24h from now"
        );

        // Verify label
        assert_eq!(invitation.label, "For Claude Code");
    }

    #[test]
    fn test_invitation_code_validation_valid() {
        let db = setup_test_db();
        let manager = InvitationManager::new(db, 50);

        let generated = manager.generate("Valid test", Some(24)).unwrap();
        let result = manager.validate(&generated.code);

        assert!(result.is_ok(), "validate should succeed for valid unused code");
        let validated = result.unwrap();
        assert_eq!(validated.code, generated.code);
    }

    #[test]
    fn test_invitation_code_validation_nonexistent() {
        let db = setup_test_db();
        let manager = InvitationManager::new(db, 50);

        let result = manager.validate("INV-nonexist");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInvitationCode => {} // expected
            other => panic!(
                "Expected InvalidInvitationCode, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_invitation_code_expiry_enforced() {
        let db = setup_test_db();
        let manager = InvitationManager::new(Arc::clone(&db), 50);

        // Insert a code that is already expired (expires_at in the past)
        let expired_code = InvitationCode {
            code: "INV-expired1".to_string(),
            created_at: 1000000,
            expires_at: Some(1000001), // far in the past
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "Expired test".to_string(),
        };
        queries::insert_invitation_code(&db, &expired_code).unwrap();

        let result = manager.validate("INV-expired1");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvitationCodeExpired => {} // expected
            other => panic!(
                "Expected InvitationCodeExpired, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_invitation_code_single_use_enforced() {
        let db = setup_test_db();
        let manager = InvitationManager::new(Arc::clone(&db), 50);

        // Insert a code that has been fully used (use_count == max_uses)
        // Note: used_by left as None to avoid FK constraint on agents table
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

        let result = manager.validate("INV-used0001");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvitationCodeExpired => {} // expected
            other => panic!(
                "Expected InvitationCodeExpired, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_invitation_code_max_active_codes_limit() {
        let db = setup_test_db();
        // Set max to 3 for easier testing
        let manager = InvitationManager::new(Arc::clone(&db), 3);

        // Generate 3 codes (should succeed)
        for i in 0..3 {
            let result = manager.generate(&format!("Code {}", i), None);
            assert!(
                result.is_ok(),
                "Generating code {} should succeed",
                i
            );
        }

        // The 4th should fail with MaxActiveCodesReached
        let result = manager.generate("One too many", None);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::MaxActiveCodesReached => {} // expected
            other => panic!(
                "Expected MaxActiveCodesReached, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_invitation_code_no_expiry_if_hours_not_set() {
        let db = setup_test_db();
        let manager = InvitationManager::new(db, 50);

        let result = manager.generate("Permanent", None);
        assert!(result.is_ok());

        let invitation = result.unwrap();
        assert!(
            invitation.expires_at.is_none(),
            "expires_at should be None when no hours specified"
        );
    }

    #[test]
    fn test_list_active_codes() {
        let db = setup_test_db();
        let manager = InvitationManager::new(Arc::clone(&db), 50);

        // Generate 2 active codes
        let code1 = manager.generate("Active 1", None).unwrap();
        let _code2 = manager.generate("Active 2", Some(24)).unwrap();

        // Insert an expired code directly
        let expired = InvitationCode {
            code: "INV-oldcode1".to_string(),
            created_at: 1000000,
            expires_at: Some(1000001),
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "Expired".to_string(),
        };
        queries::insert_invitation_code(&db, &expired).unwrap();

        let active = manager.list_active().unwrap();
        assert_eq!(active.len(), 2, "Should have 2 active codes");

        // Revoke one
        manager.revoke(&code1.code).unwrap();
        let active = manager.list_active().unwrap();
        assert_eq!(active.len(), 1, "Should have 1 active code after revoke");
    }

    #[test]
    fn test_revoke_nonexistent_code() {
        let db = setup_test_db();
        let manager = InvitationManager::new(db, 50);

        let result = manager.revoke("INV-nope0000");
        assert!(result.is_err());
    }
}
