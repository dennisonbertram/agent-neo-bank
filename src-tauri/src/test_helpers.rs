use std::sync::Arc;

use crate::db::models::*;
use crate::db::schema::Database;

/// Create an in-memory database with all migrations applied.
pub fn setup_test_db() -> Arc<Database> {
    let db = Database::new_in_memory().expect("Failed to create in-memory DB");
    db.run_migrations().expect("Failed to run migrations");
    Arc::new(db)
}

/// Create a file-based database with all migrations applied.
/// Returns the database and a TempDir that must be kept alive for the DB to persist.
pub fn setup_test_db_file() -> (Arc<Database>, tempfile::TempDir) {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = tmp_dir.path().join("test.db");
    let db = Database::new(db_path).expect("Failed to create file DB");
    db.run_migrations().expect("Failed to run migrations");
    (Arc::new(db), tmp_dir)
}

/// Create a test agent with the given name and status.
pub fn create_test_agent(name: &str, status: AgentStatus) -> Agent {
    Agent {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: format!("Test agent: {}", name),
        purpose: "Integration testing".to_string(),
        agent_type: "test".to_string(),
        capabilities: vec!["send".to_string()],
        status,
        api_token_hash: None,
        token_prefix: None,
        balance_visible: true,
        invitation_code: Some("INV-test".to_string()),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
        last_active_at: None,
        metadata: "{}".to_string(),
    }
}

/// Create a test agent with a mock token hash. Returns (agent, raw_token).
pub fn create_test_agent_with_token(name: &str) -> (Agent, String) {
    let raw_token = format!(
        "anb_test_{}",
        &uuid::Uuid::new_v4().to_string().replace('-', "")[..16]
    );
    let token_hash = format!("argon2_hash_of_{}", raw_token);
    let mut agent = create_test_agent(name, AgentStatus::Active);
    agent.api_token_hash = Some(token_hash);
    agent.token_prefix = Some(raw_token[..12].to_string());
    (agent, raw_token)
}

/// Create a test spending policy.
pub fn create_test_spending_policy(
    agent_id: &str,
    per_tx_max: &str,
    daily_cap: &str,
    weekly_cap: &str,
    monthly_cap: &str,
    auto_approve_max: &str,
) -> SpendingPolicy {
    SpendingPolicy {
        agent_id: agent_id.to_string(),
        per_tx_max: per_tx_max.to_string(),
        daily_cap: daily_cap.to_string(),
        weekly_cap: weekly_cap.to_string(),
        monthly_cap: monthly_cap.to_string(),
        auto_approve_max: auto_approve_max.to_string(),
        allowlist: vec![],
        updated_at: chrono::Utc::now().timestamp(),
    }
}

/// Create a test invitation code.
pub fn create_test_invitation(code: &str, label: &str) -> InvitationCode {
    InvitationCode {
        code: code.to_string(),
        created_at: chrono::Utc::now().timestamp(),
        expires_at: None,
        used_by: None,
        used_at: None,
        max_uses: 1,
        use_count: 0,
        label: label.to_string(),
    }
}
