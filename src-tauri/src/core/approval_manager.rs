use std::sync::Arc;

use chrono::Utc;

use crate::db::models::{ApprovalRequest, ApprovalRequestType, ApprovalStatus};
use crate::db::queries;
use crate::db::schema::Database;
use crate::error::AppError;

/// Manages approval request lifecycle: creating, resolving, listing, and
/// cleaning up stale (expired) approvals.
pub struct ApprovalManager {
    db: Arc<Database>,
}

impl ApprovalManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Create a new approval request. Returns the created request.
    pub fn create_request(
        &self,
        agent_id: &str,
        request_type: ApprovalRequestType,
        payload: serde_json::Value,
        tx_id: Option<&str>,
        expires_in_seconds: Option<i64>,
    ) -> Result<ApprovalRequest, AppError> {
        let now = Utc::now().timestamp();
        let expires_at = now + expires_in_seconds.unwrap_or(86400); // 24h default

        let approval = ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            request_type,
            payload: payload.to_string(),
            status: ApprovalStatus::Pending,
            tx_id: tx_id.map(String::from),
            expires_at,
            created_at: now,
            resolved_at: None,
            resolved_by: None,
        };

        queries::insert_approval_request(&self.db, &approval)?;
        Ok(approval)
    }

    /// List all approvals, optionally filtered by status.
    pub fn list_all(
        &self,
        status: Option<&ApprovalStatus>,
    ) -> Result<Vec<ApprovalRequest>, AppError> {
        queries::list_approvals(&self.db, status)
    }

    /// Resolve a pending approval (approve or deny).
    ///
    /// Returns the updated approval request on success.
    /// Fails if the approval does not exist or is already resolved.
    pub fn resolve(
        &self,
        approval_id: &str,
        decision: ApprovalStatus,
        resolved_by: &str,
    ) -> Result<ApprovalRequest, AppError> {
        // Validate the decision is either Approved or Denied
        match &decision {
            ApprovalStatus::Approved | ApprovalStatus::Denied => {}
            _ => {
                return Err(AppError::InvalidInput(
                    "Decision must be Approved or Denied".to_string(),
                ));
            }
        }

        // Fetch the current approval
        let existing = queries::get_approval_request(&self.db, approval_id)?;

        // Only pending approvals can be resolved
        if existing.status != ApprovalStatus::Pending {
            return Err(AppError::InvalidInput(format!(
                "Approval {} is already resolved with status: {}",
                approval_id, existing.status
            )));
        }

        let now = Utc::now().timestamp();
        queries::update_approval_status(
            &self.db,
            approval_id,
            &decision,
            Some(now),
            Some(resolved_by),
        )?;

        // Return the updated approval
        queries::get_approval_request(&self.db, approval_id)
    }

    /// Clean up stale approvals whose expires_at is in the past.
    ///
    /// For each expired approval:
    /// 1. Sets status to "expired", resolved_at = now, resolved_by = "auto"
    /// 2. If the approval has an associated tx_id, marks that transaction as "failed"
    ///
    /// Returns the count of approvals that were expired.
    pub fn cleanup_expired(&self) -> Result<usize, AppError> {
        let now = Utc::now().timestamp();
        let expired = queries::list_expired_approvals(&self.db, now)?;
        let count = expired.len();

        for approval in &expired {
            queries::update_approval_status(
                &self.db,
                &approval.id,
                &ApprovalStatus::Expired,
                Some(now),
                Some("auto"),
            )?;

            // If there's an associated transaction, rollback the reservation and mark it as failed
            if let Some(ref tx_id) = approval.tx_id {
                // Rollback the spending reservation that was made during check_policy_and_reserve_atomic
                if let Ok(tx) = queries::get_transaction(&self.db, tx_id) {
                    let _ = queries::rollback_reservation(
                        &self.db,
                        &approval.agent_id,
                        &tx.amount,
                        &tx.period_daily,
                        &tx.period_weekly,
                        &tx.period_monthly,
                        now,
                    );
                }
                queries::update_transaction_status(
                    &self.db,
                    tx_id,
                    &crate::db::models::TxStatus::Failed,
                    None, // chain_tx_hash
                    Some("Approval expired"),
                    now,
                )?;
            }
        }

        Ok(count)
    }

    /// List pending approvals, optionally filtered by agent_id.
    pub fn list_pending(
        &self,
        agent_id: Option<&str>,
    ) -> Result<Vec<ApprovalRequest>, AppError> {
        queries::list_pending_approvals(&self.db, agent_id)
    }

    /// Get a specific approval by id.
    pub fn get(&self, approval_id: &str) -> Result<ApprovalRequest, AppError> {
        queries::get_approval_request(&self.db, approval_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::*;
    use crate::db::queries::{insert_agent, insert_approval_request, insert_transaction};
    use crate::test_helpers::{create_test_agent, setup_test_db};

    fn make_approval(
        agent_id: &str,
        request_type: ApprovalRequestType,
        tx_id: Option<&str>,
        expires_at: i64,
    ) -> ApprovalRequest {
        ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            request_type,
            payload: "{}".to_string(),
            status: ApprovalStatus::Pending,
            tx_id: tx_id.map(|s| s.to_string()),
            expires_at,
            created_at: Utc::now().timestamp(),
            resolved_at: None,
            resolved_by: None,
        }
    }

    fn make_test_transaction(agent_id: &str, status: TxStatus) -> Transaction {
        Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: Some(agent_id.to_string()),
            tx_type: TxType::Send,
            amount: "10.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0xRecipient".to_string()),
            sender: None,
            chain_tx_hash: None,
            status,
            category: "test".to_string(),
            memo: "test memo".to_string(),
            description: "test description".to_string(),
            service_name: "Test".to_string(),
            service_url: "".to_string(),
            reason: "testing".to_string(),
            webhook_url: None,
            error_message: None,
            period_daily: "daily:2026-02-27".to_string(),
            period_weekly: "weekly:2026-W09".to_string(),
            period_monthly: "monthly:2026-02".to_string(),
            created_at: 1000000,
            updated_at: 1000000,
        }
    }

    // -----------------------------------------------------------------------
    // Test 1: Resolve approval as approved
    // -----------------------------------------------------------------------
    #[test]
    fn test_resolve_approval_approved() {
        let db = setup_test_db();
        let agent = create_test_agent("ApprovalBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let approval = make_approval(
            &agent.id,
            ApprovalRequestType::Transaction,
            None,
            Utc::now().timestamp() + 86400,
        );
        insert_approval_request(&db, &approval).unwrap();

        let manager = ApprovalManager::new(db);
        let result = manager
            .resolve(&approval.id, ApprovalStatus::Approved, "user")
            .unwrap();

        assert_eq!(result.status, ApprovalStatus::Approved);
        assert!(result.resolved_at.is_some());
        assert_eq!(result.resolved_by.as_deref(), Some("user"));
    }

    // -----------------------------------------------------------------------
    // Test 2: Resolve approval as denied
    // -----------------------------------------------------------------------
    #[test]
    fn test_resolve_approval_denied() {
        let db = setup_test_db();
        let agent = create_test_agent("DenyBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let approval = make_approval(
            &agent.id,
            ApprovalRequestType::LimitIncrease,
            None,
            Utc::now().timestamp() + 86400,
        );
        insert_approval_request(&db, &approval).unwrap();

        let manager = ApprovalManager::new(db);
        let result = manager
            .resolve(&approval.id, ApprovalStatus::Denied, "admin")
            .unwrap();

        assert_eq!(result.status, ApprovalStatus::Denied);
        assert!(result.resolved_at.is_some());
        assert_eq!(result.resolved_by.as_deref(), Some("admin"));
    }

    // -----------------------------------------------------------------------
    // Test 3: Resolve nonexistent approval fails
    // -----------------------------------------------------------------------
    #[test]
    fn test_resolve_nonexistent_approval_fails() {
        let db = setup_test_db();
        let manager = ApprovalManager::new(db);

        let result = manager.resolve("nonexistent-id", ApprovalStatus::Approved, "user");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(_) => {} // expected
            other => panic!("Expected NotFound, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // Test 4: Resolve already resolved approval fails
    // -----------------------------------------------------------------------
    #[test]
    fn test_resolve_already_resolved_fails() {
        let db = setup_test_db();
        let agent = create_test_agent("ResolvedBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let approval = make_approval(
            &agent.id,
            ApprovalRequestType::Registration,
            None,
            Utc::now().timestamp() + 86400,
        );
        insert_approval_request(&db, &approval).unwrap();

        let manager = ApprovalManager::new(db);

        // Resolve it once
        manager
            .resolve(&approval.id, ApprovalStatus::Approved, "user")
            .unwrap();

        // Try to resolve again
        let result = manager.resolve(&approval.id, ApprovalStatus::Denied, "user");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput(_) => {} // expected
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // Test 5: Cleanup expires stale approvals
    // -----------------------------------------------------------------------
    #[test]
    fn test_cleanup_expires_stale_approvals() {
        let db = setup_test_db();
        let agent = create_test_agent("StaleBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        // Create an approval that expired 1 hour ago
        let approval = make_approval(
            &agent.id,
            ApprovalRequestType::LimitIncrease,
            None,
            Utc::now().timestamp() - 3600,
        );
        insert_approval_request(&db, &approval).unwrap();

        let manager = ApprovalManager::new(db.clone());
        let count = manager.cleanup_expired().unwrap();

        assert_eq!(count, 1);

        // Verify it was marked expired
        let updated = queries::get_approval_request(&db, &approval.id).unwrap();
        assert_eq!(updated.status, ApprovalStatus::Expired);
        assert!(updated.resolved_at.is_some());
        assert_eq!(updated.resolved_by.as_deref(), Some("auto"));
    }

    // -----------------------------------------------------------------------
    // Test 6: Cleanup does not affect non-expired approvals
    // -----------------------------------------------------------------------
    #[test]
    fn test_cleanup_does_not_affect_non_expired() {
        let db = setup_test_db();
        let agent = create_test_agent("FreshBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        // Create an approval that expires in the future
        let approval = make_approval(
            &agent.id,
            ApprovalRequestType::Transaction,
            None,
            Utc::now().timestamp() + 86400,
        );
        insert_approval_request(&db, &approval).unwrap();

        let manager = ApprovalManager::new(db.clone());
        let count = manager.cleanup_expired().unwrap();

        assert_eq!(count, 0);

        // Verify it's still pending
        let updated = queries::get_approval_request(&db, &approval.id).unwrap();
        assert_eq!(updated.status, ApprovalStatus::Pending);
        assert!(updated.resolved_at.is_none());
    }

    // -----------------------------------------------------------------------
    // Test 7: Cleanup fails associated transactions
    // -----------------------------------------------------------------------
    #[test]
    fn test_cleanup_fails_associated_transactions() {
        let db = setup_test_db();
        let agent = create_test_agent("TxExpireBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        // Create a pending transaction
        let tx = make_test_transaction(&agent.id, TxStatus::Pending);
        insert_transaction(&db, &tx).unwrap();

        // Create an expired approval linked to the transaction
        let approval = make_approval(
            &agent.id,
            ApprovalRequestType::Transaction,
            Some(&tx.id),
            Utc::now().timestamp() - 3600,
        );
        insert_approval_request(&db, &approval).unwrap();

        let manager = ApprovalManager::new(db.clone());
        let count = manager.cleanup_expired().unwrap();

        assert_eq!(count, 1);

        // Verify the transaction was marked as failed
        let updated_tx = crate::db::queries::get_transaction(&db, &tx.id).unwrap();
        assert_eq!(updated_tx.status, TxStatus::Failed);
        assert_eq!(
            updated_tx.error_message.as_deref(),
            Some("Approval expired")
        );
    }

    // -----------------------------------------------------------------------
    // Test 8: List pending approvals
    // -----------------------------------------------------------------------
    #[test]
    fn test_list_pending_approvals() {
        let db = setup_test_db();
        let agent1 = create_test_agent("PendBot1", AgentStatus::Active);
        let agent2 = create_test_agent("PendBot2", AgentStatus::Active);
        insert_agent(&db, &agent1).unwrap();
        insert_agent(&db, &agent2).unwrap();

        // Create two pending approvals for agent1
        let a1 = make_approval(
            &agent1.id,
            ApprovalRequestType::Transaction,
            None,
            Utc::now().timestamp() + 86400,
        );
        let a2 = make_approval(
            &agent1.id,
            ApprovalRequestType::LimitIncrease,
            None,
            Utc::now().timestamp() + 86400,
        );
        // One pending for agent2
        let a3 = make_approval(
            &agent2.id,
            ApprovalRequestType::Registration,
            None,
            Utc::now().timestamp() + 86400,
        );
        insert_approval_request(&db, &a1).unwrap();
        insert_approval_request(&db, &a2).unwrap();
        insert_approval_request(&db, &a3).unwrap();

        // Resolve one of agent1's approvals so it's no longer pending
        queries::update_approval_status(
            &db,
            &a1.id,
            &ApprovalStatus::Approved,
            Some(Utc::now().timestamp()),
            Some("user"),
        )
        .unwrap();

        let manager = ApprovalManager::new(db);

        // All pending
        let all_pending = manager.list_pending(None).unwrap();
        assert_eq!(all_pending.len(), 2); // a2 + a3

        // Filtered by agent1
        let agent1_pending = manager.list_pending(Some(&agent1.id)).unwrap();
        assert_eq!(agent1_pending.len(), 1);
        assert_eq!(agent1_pending[0].id, a2.id);

        // Filtered by agent2
        let agent2_pending = manager.list_pending(Some(&agent2.id)).unwrap();
        assert_eq!(agent2_pending.len(), 1);
        assert_eq!(agent2_pending[0].id, a3.id);
    }

    // -----------------------------------------------------------------------
    // Test 9: Get approval by id
    // -----------------------------------------------------------------------
    #[test]
    fn test_get_approval_by_id() {
        let db = setup_test_db();
        let agent = create_test_agent("GetBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let approval = make_approval(
            &agent.id,
            ApprovalRequestType::Transaction,
            None,
            Utc::now().timestamp() + 86400,
        );
        insert_approval_request(&db, &approval).unwrap();

        let manager = ApprovalManager::new(db);
        let fetched = manager.get(&approval.id).unwrap();

        assert_eq!(fetched.id, approval.id);
        assert_eq!(fetched.agent_id, agent.id);
        assert_eq!(fetched.request_type, ApprovalRequestType::Transaction);
        assert_eq!(fetched.status, ApprovalStatus::Pending);

        // Nonexistent returns error
        let result = manager.get("nonexistent-id");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(_) => {}
            other => panic!("Expected NotFound, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // Test 10: Create request via create_request()
    // -----------------------------------------------------------------------
    #[test]
    fn test_approval_create_request() {
        let db = setup_test_db();
        let agent = create_test_agent("CreateBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        // Create a transaction so the FK constraint is satisfied
        let tx = make_test_transaction(&agent.id, TxStatus::Pending);
        insert_transaction(&db, &tx).unwrap();

        let manager = ApprovalManager::new(db.clone());
        let payload = serde_json::json!({"amount": "50.00", "recipient": "0xABC"});
        let result = manager
            .create_request(
                &agent.id,
                ApprovalRequestType::Transaction,
                payload.clone(),
                Some(&tx.id),
                None,
            )
            .unwrap();

        assert_eq!(result.agent_id, agent.id);
        assert_eq!(result.request_type, ApprovalRequestType::Transaction);
        assert_eq!(result.status, ApprovalStatus::Pending);
        assert_eq!(result.tx_id.as_deref(), Some(tx.id.as_str()));
        assert!(result.resolved_at.is_none());
        assert!(result.resolved_by.is_none());

        // Verify it is persisted in the DB
        let fetched = queries::get_approval_request(&db, &result.id).unwrap();
        assert_eq!(fetched.id, result.id);
        assert_eq!(fetched.agent_id, agent.id);
        assert_eq!(fetched.payload, payload.to_string());
    }

    // -----------------------------------------------------------------------
    // Test 11: Create request with custom expiry
    // -----------------------------------------------------------------------
    #[test]
    fn test_approval_create_request_with_custom_expiry() {
        let db = setup_test_db();
        let agent = create_test_agent("ExpiryBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let manager = ApprovalManager::new(db);
        let now = Utc::now().timestamp();

        let result = manager
            .create_request(
                &agent.id,
                ApprovalRequestType::LimitIncrease,
                serde_json::json!({"new_limit": "1000"}),
                None,
                Some(3600), // 1 hour
            )
            .unwrap();

        // expires_at should be approximately now + 3600
        let diff = result.expires_at - now;
        assert!(
            diff >= 3599 && diff <= 3601,
            "Expected expires_at ~1h from now, got diff={}",
            diff
        );
    }

    // -----------------------------------------------------------------------
    // Test 12: List all with status filter
    // -----------------------------------------------------------------------
    #[test]
    fn test_approval_list_all_with_status_filter() {
        let db = setup_test_db();
        let agent = create_test_agent("ListAllBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let manager = ApprovalManager::new(db.clone());

        // Create two pending approvals
        let a1 = manager
            .create_request(
                &agent.id,
                ApprovalRequestType::Transaction,
                serde_json::json!({}),
                None,
                None,
            )
            .unwrap();
        let _a2 = manager
            .create_request(
                &agent.id,
                ApprovalRequestType::LimitIncrease,
                serde_json::json!({}),
                None,
                None,
            )
            .unwrap();

        // Approve one
        manager
            .resolve(&a1.id, ApprovalStatus::Approved, "user")
            .unwrap();

        // List all: should be 2
        let all = manager.list_all(None).unwrap();
        assert_eq!(all.len(), 2);

        // List pending only: should be 1
        let pending = manager.list_all(Some(&ApprovalStatus::Pending)).unwrap();
        assert_eq!(pending.len(), 1);

        // List approved only: should be 1
        let approved = manager.list_all(Some(&ApprovalStatus::Approved)).unwrap();
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].id, a1.id);
    }

    // -----------------------------------------------------------------------
    // Test 13: Create and resolve flow
    // -----------------------------------------------------------------------
    #[test]
    fn test_approval_create_and_resolve_flow() {
        let db = setup_test_db();
        let agent = create_test_agent("FlowBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let manager = ApprovalManager::new(db);

        // Create
        let created = manager
            .create_request(
                &agent.id,
                ApprovalRequestType::Registration,
                serde_json::json!({"name": "NewAgent"}),
                None,
                None,
            )
            .unwrap();
        assert_eq!(created.status, ApprovalStatus::Pending);

        // Resolve as denied
        let resolved = manager
            .resolve(&created.id, ApprovalStatus::Denied, "admin")
            .unwrap();
        assert_eq!(resolved.status, ApprovalStatus::Denied);
        assert!(resolved.resolved_at.is_some());
        assert_eq!(resolved.resolved_by.as_deref(), Some("admin"));

        // Cannot resolve again
        let err = manager
            .resolve(&created.id, ApprovalStatus::Approved, "user")
            .unwrap_err();
        match err {
            AppError::InvalidInput(_) => {} // expected
            other => panic!("Expected InvalidInput, got: {:?}", other),
        }
    }
}
