use serde::Serialize;
use tauri::State;

use crate::db::models::Transaction;
use crate::db::queries;
use crate::error::AppError;
use crate::state::app_state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct ListTransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub total: i64,
}

#[tauri::command]
pub async fn list_transactions(
    limit: i64,
    offset: i64,
    status: Option<String>,
    agent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<ListTransactionsResponse, AppError> {
    let db = state.db.clone();
    let (transactions, total) = tokio::task::spawn_blocking(move || {
        queries::list_transactions_paginated(
            &db,
            agent_id.as_deref(),
            status.as_deref(),
            limit,
            offset,
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    Ok(ListTransactionsResponse {
        transactions,
        total,
    })
}

#[tauri::command]
pub async fn get_transaction(
    tx_id: String,
    state: State<'_, AppState>,
) -> Result<Transaction, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || queries::get_transaction(&db, &tx_id))
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Agent, AgentStatus, Transaction, TxStatus, TxType};
    use crate::db::queries::{insert_agent, insert_transaction};
    use crate::db::schema::Database;

    fn create_test_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn create_test_agent(id: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: "Test Agent".to_string(),
            description: "Test".to_string(),
            purpose: "Testing".to_string(),
            agent_type: "test".to_string(),
            capabilities: vec!["send".to_string()],
            status: AgentStatus::Active,
            api_token_hash: None,
            token_prefix: None,
            balance_visible: true,
            invitation_code: None,
            created_at: 1740700800,
            updated_at: 1740700800,
            last_active_at: None,
            metadata: "{}".to_string(),
        }
    }

    fn create_test_transaction(id: &str, status: TxStatus) -> Transaction {
        Transaction {
            id: id.to_string(),
            agent_id: Some("agent-1".to_string()),
            tx_type: TxType::Send,
            amount: "10.00".to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0xRecipient".to_string()),
            sender: None,
            chain_tx_hash: None,
            status,
            category: "test".to_string(),
            memo: "Test".to_string(),
            description: "Test transaction".to_string(),
            service_name: "Test".to_string(),
            service_url: "https://test.example.com".to_string(),
            reason: "Testing".to_string(),
            webhook_url: None,
            error_message: None,
            period_daily: "daily:2026-02-27".to_string(),
            period_weekly: "weekly:2026-W09".to_string(),
            period_monthly: "monthly:2026-02".to_string(),
            created_at: 1740700800,
            updated_at: 1740700800,
        }
    }

    #[test]
    fn test_list_transactions_command() {
        let db = create_test_db();

        // Insert agent first (foreign key requirement)
        let agent = create_test_agent("agent-1");
        insert_agent(&db, &agent).unwrap();

        // Insert test transactions
        let tx1 = create_test_transaction("tx-1", TxStatus::Confirmed);
        let tx2 = create_test_transaction("tx-2", TxStatus::Pending);
        let tx3 = create_test_transaction("tx-3", TxStatus::Failed);

        insert_transaction(&db, &tx1).unwrap();
        insert_transaction(&db, &tx2).unwrap();
        insert_transaction(&db, &tx3).unwrap();

        // Test paginated query - all transactions
        let (transactions, total) =
            queries::list_transactions_paginated(&db, None, None, 20, 0).unwrap();
        assert_eq!(total, 3);
        assert_eq!(transactions.len(), 3);

        // Test with status filter
        let (transactions, total) =
            queries::list_transactions_paginated(&db, None, Some("confirmed"), 20, 0).unwrap();
        assert_eq!(total, 1);
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id, "tx-1");

        // Test with pagination
        let (transactions, total) =
            queries::list_transactions_paginated(&db, None, None, 2, 0).unwrap();
        assert_eq!(total, 3);
        assert_eq!(transactions.len(), 2);

        let (transactions, _) =
            queries::list_transactions_paginated(&db, None, None, 2, 2).unwrap();
        assert_eq!(transactions.len(), 1);
    }
}
