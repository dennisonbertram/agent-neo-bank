use rusqlite::params;

use crate::error::AppError;

use super::models::*;
use super::schema::Database;

// -------------------------------------------------------------------------
// Agent CRUD
// -------------------------------------------------------------------------

pub fn insert_agent(db: &Database, agent: &Agent) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let capabilities_json = serde_json::to_string(&agent.capabilities)
        .map_err(|e| AppError::DatabaseError(format!("Failed to serialize capabilities: {}", e)))?;
    conn.execute(
        "INSERT INTO agents (id, name, description, purpose, agent_type, capabilities, status,
         api_token_hash, token_prefix, balance_visible, invitation_code, created_at, updated_at,
         last_active_at, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            agent.id,
            agent.name,
            agent.description,
            agent.purpose,
            agent.agent_type,
            capabilities_json,
            agent.status.to_string(),
            agent.api_token_hash,
            agent.token_prefix,
            agent.balance_visible as i32,
            agent.invitation_code,
            agent.created_at,
            agent.updated_at,
            agent.last_active_at,
            agent.metadata,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to insert agent: {}", e)))?;
    Ok(())
}

pub fn get_agent(db: &Database, id: &str) -> Result<Agent, AppError> {
    let conn = db.get_connection()?;
    conn.query_row(
        "SELECT id, name, description, purpose, agent_type, capabilities, status,
         api_token_hash, token_prefix, balance_visible, invitation_code, created_at,
         updated_at, last_active_at, metadata
         FROM agents WHERE id = ?1",
        params![id],
        |row| {
            let capabilities_str: String = row.get(5)?;
            let capabilities: Vec<String> =
                serde_json::from_str(&capabilities_str).unwrap_or_default();
            let status_str: String = row.get(6)?;
            let status = match status_str.as_str() {
                "active" => AgentStatus::Active,
                "suspended" => AgentStatus::Suspended,
                "revoked" => AgentStatus::Revoked,
                _ => AgentStatus::Pending,
            };
            let balance_visible: i32 = row.get(9)?;
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                purpose: row.get(3)?,
                agent_type: row.get(4)?,
                capabilities,
                status,
                api_token_hash: row.get(7)?,
                token_prefix: row.get(8)?,
                balance_visible: balance_visible != 0,
                invitation_code: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                last_active_at: row.get(13)?,
                metadata: row.get(14)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Agent not found: {}", id))
        }
        _ => AppError::DatabaseError(format!("Failed to get agent: {}", e)),
    })
}

pub fn update_agent_status(
    db: &Database,
    id: &str,
    status: &AgentStatus,
    updated_at: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute(
            "UPDATE agents SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.to_string(), updated_at, id],
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to update agent status: {}", e)))?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Agent not found: {}", id)));
    }
    Ok(())
}

pub fn list_agents_by_status(db: &Database, status: &AgentStatus) -> Result<Vec<Agent>, AppError> {
    let conn = db.get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, purpose, agent_type, capabilities, status,
             api_token_hash, token_prefix, balance_visible, invitation_code, created_at,
             updated_at, last_active_at, metadata
             FROM agents WHERE status = ?1",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let agents = stmt
        .query_map(params![status.to_string()], |row| {
            let capabilities_str: String = row.get(5)?;
            let capabilities: Vec<String> =
                serde_json::from_str(&capabilities_str).unwrap_or_default();
            let status_str: String = row.get(6)?;
            let status = match status_str.as_str() {
                "active" => AgentStatus::Active,
                "suspended" => AgentStatus::Suspended,
                "revoked" => AgentStatus::Revoked,
                _ => AgentStatus::Pending,
            };
            let balance_visible: i32 = row.get(9)?;
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                purpose: row.get(3)?,
                agent_type: row.get(4)?,
                capabilities,
                status,
                api_token_hash: row.get(7)?,
                token_prefix: row.get(8)?,
                balance_visible: balance_visible != 0,
                invitation_code: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                last_active_at: row.get(13)?,
                metadata: row.get(14)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to query agents: {}", e)))?;

    agents
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect agents: {}", e)))
}

pub fn list_all_agents(db: &Database) -> Result<Vec<Agent>, AppError> {
    let conn = db.get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, purpose, agent_type, capabilities, status,
             api_token_hash, token_prefix, balance_visible, invitation_code, created_at,
             updated_at, last_active_at, metadata
             FROM agents ORDER BY name ASC",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let agents = stmt
        .query_map([], |row| {
            let capabilities_str: String = row.get(5)?;
            let capabilities: Vec<String> =
                serde_json::from_str(&capabilities_str).unwrap_or_default();
            let status_str: String = row.get(6)?;
            let status = match status_str.as_str() {
                "active" => AgentStatus::Active,
                "suspended" => AgentStatus::Suspended,
                "revoked" => AgentStatus::Revoked,
                _ => AgentStatus::Pending,
            };
            let balance_visible: i32 = row.get(9)?;
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                purpose: row.get(3)?,
                agent_type: row.get(4)?,
                capabilities,
                status,
                api_token_hash: row.get(7)?,
                token_prefix: row.get(8)?,
                balance_visible: balance_visible != 0,
                invitation_code: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                last_active_at: row.get(13)?,
                metadata: row.get(14)?,
            })
        })
        .map_err(|e| AppError::DatabaseError(format!("Failed to query agents: {}", e)))?;

    agents
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect agents: {}", e)))
}

pub fn delete_agent(db: &Database, id: &str) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute("DELETE FROM agents WHERE id = ?1", params![id])
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete agent: {}", e)))?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Agent not found: {}", id)));
    }
    Ok(())
}

// -------------------------------------------------------------------------
// Transaction CRUD
// -------------------------------------------------------------------------

pub fn insert_transaction(db: &Database, tx: &Transaction) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "INSERT INTO transactions (id, agent_id, tx_type, amount, asset, recipient, sender,
         chain_tx_hash, status, category, memo, description, service_name, service_url,
         reason, webhook_url, error_message, period_daily, period_weekly, period_monthly,
         created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            tx.id,
            tx.agent_id,
            tx.tx_type.to_string(),
            tx.amount,
            tx.asset,
            tx.recipient,
            tx.sender,
            tx.chain_tx_hash,
            tx.status.to_string(),
            tx.category,
            tx.memo,
            tx.description,
            tx.service_name,
            tx.service_url,
            tx.reason,
            tx.webhook_url,
            tx.error_message,
            tx.period_daily,
            tx.period_weekly,
            tx.period_monthly,
            tx.created_at,
            tx.updated_at,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to insert transaction: {}", e)))?;
    Ok(())
}

pub fn get_transaction(db: &Database, id: &str) -> Result<Transaction, AppError> {
    let conn = db.get_connection()?;
    conn.query_row(
        "SELECT id, agent_id, tx_type, amount, asset, recipient, sender, chain_tx_hash,
         status, category, memo, description, service_name, service_url, reason,
         webhook_url, error_message, period_daily, period_weekly, period_monthly,
         created_at, updated_at
         FROM transactions WHERE id = ?1",
        params![id],
        |row| row_to_transaction(row),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Transaction not found: {}", id))
        }
        _ => AppError::DatabaseError(format!("Failed to get transaction: {}", e)),
    })
}

pub fn list_transactions_for_agent(
    db: &Database,
    agent_id: &str,
    limit: i64,
) -> Result<Vec<Transaction>, AppError> {
    let conn = db.get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, tx_type, amount, asset, recipient, sender, chain_tx_hash,
             status, category, memo, description, service_name, service_url, reason,
             webhook_url, error_message, period_daily, period_weekly, period_monthly,
             created_at, updated_at
             FROM transactions WHERE agent_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let txs = stmt
        .query_map(params![agent_id, limit], |row| row_to_transaction(row))
        .map_err(|e| AppError::DatabaseError(format!("Failed to query transactions: {}", e)))?;

    txs.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect transactions: {}", e)))
}

pub fn list_transactions_by_agent(
    db: &Database,
    agent_id: &str,
) -> Result<Vec<Transaction>, AppError> {
    let conn = db.get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, tx_type, amount, asset, recipient, sender, chain_tx_hash,
             status, category, memo, description, service_name, service_url, reason,
             webhook_url, error_message, period_daily, period_weekly, period_monthly,
             created_at, updated_at
             FROM transactions WHERE agent_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let txs = stmt
        .query_map(params![agent_id], |row| row_to_transaction(row))
        .map_err(|e| AppError::DatabaseError(format!("Failed to query transactions: {}", e)))?;

    txs.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect transactions: {}", e)))
}

pub fn list_transactions_by_status(
    db: &Database,
    status: &TxStatus,
) -> Result<Vec<Transaction>, AppError> {
    let conn = db.get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, tx_type, amount, asset, recipient, sender, chain_tx_hash,
             status, category, memo, description, service_name, service_url, reason,
             webhook_url, error_message, period_daily, period_weekly, period_monthly,
             created_at, updated_at
             FROM transactions WHERE status = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let txs = stmt
        .query_map(params![status.to_string()], |row| row_to_transaction(row))
        .map_err(|e| AppError::DatabaseError(format!("Failed to query transactions: {}", e)))?;

    txs.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect transactions: {}", e)))
}

pub fn list_transactions_paginated(
    db: &Database,
    agent_id: Option<&str>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Transaction>, i64), AppError> {
    let conn = db.get_connection()?;

    // Build WHERE clause dynamically
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(aid) = agent_id {
        conditions.push(format!("agent_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(aid.to_string()));
    }
    if let Some(s) = status {
        conditions.push(format!("status = ?{}", param_values.len() + 1));
        param_values.push(Box::new(s.to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    // Get total count
    let count_sql = format!("SELECT COUNT(*) FROM transactions{}", where_clause);
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn
        .query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))
        .map_err(|e| AppError::DatabaseError(format!("Failed to count transactions: {}", e)))?;

    // Get paginated results
    let select_sql = format!(
        "SELECT id, agent_id, tx_type, amount, asset, recipient, sender, chain_tx_hash,
         status, category, memo, description, service_name, service_url, reason,
         webhook_url, error_message, period_daily, period_weekly, period_monthly,
         created_at, updated_at
         FROM transactions{} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
        where_clause,
        param_values.len() + 1,
        param_values.len() + 2,
    );

    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = param_values;
    all_params.push(Box::new(limit));
    all_params.push(Box::new(offset));
    let all_params_ref: Vec<&dyn rusqlite::types::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&select_sql)
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let txs = stmt
        .query_map(all_params_ref.as_slice(), |row| row_to_transaction(row))
        .map_err(|e| AppError::DatabaseError(format!("Failed to query transactions: {}", e)))?;

    let results = txs
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect transactions: {}", e)))?;

    Ok((results, total))
}

fn row_to_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<Transaction> {
    let tx_type_str: String = row.get(2)?;
    let tx_type = match tx_type_str.as_str() {
        "receive" => TxType::Receive,
        "earn" => TxType::Earn,
        _ => TxType::Send,
    };
    let status_str: String = row.get(8)?;
    let status = match status_str.as_str() {
        "approved" => TxStatus::Approved,
        "awaiting_approval" => TxStatus::AwaitingApproval,
        "executing" => TxStatus::Executing,
        "confirmed" => TxStatus::Confirmed,
        "failed" => TxStatus::Failed,
        "denied" => TxStatus::Denied,
        _ => TxStatus::Pending,
    };
    Ok(Transaction {
        id: row.get(0)?,
        agent_id: row.get(1)?,
        tx_type,
        amount: row.get(3)?,
        asset: row.get(4)?,
        recipient: row.get(5)?,
        sender: row.get(6)?,
        chain_tx_hash: row.get(7)?,
        status,
        category: row.get(9)?,
        memo: row.get(10)?,
        description: row.get(11)?,
        service_name: row.get(12)?,
        service_url: row.get(13)?,
        reason: row.get(14)?,
        webhook_url: row.get(15)?,
        error_message: row.get(16)?,
        period_daily: row.get(17)?,
        period_weekly: row.get(18)?,
        period_monthly: row.get(19)?,
        created_at: row.get(20)?,
        updated_at: row.get(21)?,
    })
}

// -------------------------------------------------------------------------
// Spending Policy CRUD
// -------------------------------------------------------------------------

pub fn insert_spending_policy(db: &Database, policy: &SpendingPolicy) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let allowlist_json = serde_json::to_string(&policy.allowlist)
        .map_err(|e| AppError::DatabaseError(format!("Failed to serialize allowlist: {}", e)))?;
    conn.execute(
        "INSERT INTO spending_policies (agent_id, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max, allowlist, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            policy.agent_id,
            policy.per_tx_max,
            policy.daily_cap,
            policy.weekly_cap,
            policy.monthly_cap,
            policy.auto_approve_max,
            allowlist_json,
            policy.updated_at,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to insert spending policy: {}", e)))?;
    Ok(())
}

pub fn get_spending_policy(db: &Database, agent_id: &str) -> Result<SpendingPolicy, AppError> {
    let conn = db.get_connection()?;
    conn.query_row(
        "SELECT agent_id, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max, allowlist, updated_at
         FROM spending_policies WHERE agent_id = ?1",
        params![agent_id],
        |row| {
            let allowlist_str: String = row.get(6)?;
            let allowlist: Vec<String> =
                serde_json::from_str(&allowlist_str).unwrap_or_default();
            Ok(SpendingPolicy {
                agent_id: row.get(0)?,
                per_tx_max: row.get(1)?,
                daily_cap: row.get(2)?,
                weekly_cap: row.get(3)?,
                monthly_cap: row.get(4)?,
                auto_approve_max: row.get(5)?,
                allowlist,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Spending policy not found for agent: {}", agent_id))
        }
        _ => AppError::DatabaseError(format!("Failed to get spending policy: {}", e)),
    })
}

pub fn update_spending_policy(db: &Database, policy: &SpendingPolicy) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let allowlist_json = serde_json::to_string(&policy.allowlist)
        .map_err(|e| AppError::DatabaseError(format!("Failed to serialize allowlist: {}", e)))?;
    conn.execute(
        "UPDATE spending_policies SET per_tx_max = ?2, daily_cap = ?3, weekly_cap = ?4, monthly_cap = ?5, auto_approve_max = ?6, allowlist = ?7, updated_at = ?8 WHERE agent_id = ?1",
        params![
            policy.agent_id,
            policy.per_tx_max,
            policy.daily_cap,
            policy.weekly_cap,
            policy.monthly_cap,
            policy.auto_approve_max,
            allowlist_json,
            policy.updated_at,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to update spending policy: {}", e)))?;
    Ok(())
}

// -------------------------------------------------------------------------
// Spending Ledger (BEGIN EXCLUSIVE)
// -------------------------------------------------------------------------

pub fn upsert_spending_ledger(
    db: &Database,
    agent_id: &str,
    period: &str,
    amount: &str,
    updated_at: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute_batch("BEGIN EXCLUSIVE")
        .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

    let result = conn.execute(
        "INSERT INTO spending_ledger (agent_id, period, total, tx_count, updated_at)
         VALUES (?1, ?2, ?3, 1, ?4)
         ON CONFLICT(agent_id, period) DO UPDATE SET
           total = CAST((CAST(spending_ledger.total AS REAL) + CAST(?3 AS REAL)) AS TEXT),
           tx_count = spending_ledger.tx_count + 1,
           updated_at = ?4",
        params![agent_id, period, amount, updated_at],
    );

    match result {
        Ok(_) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::DatabaseError(format!("Failed to commit: {}", e)))?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(AppError::DatabaseError(format!(
                "Failed to upsert spending ledger: {}",
                e
            )))
        }
    }
}

pub fn get_spending_for_period(
    db: &Database,
    agent_id: &str,
    period: &str,
) -> Result<Option<SpendingLedger>, AppError> {
    let conn = db.get_connection()?;
    match conn.query_row(
        "SELECT agent_id, period, total, tx_count, updated_at
         FROM spending_ledger WHERE agent_id = ?1 AND period = ?2",
        params![agent_id, period],
        |row| {
            Ok(SpendingLedger {
                agent_id: row.get(0)?,
                period: row.get(1)?,
                total: row.get(2)?,
                tx_count: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    ) {
        Ok(ledger) => Ok(Some(ledger)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(format!(
            "Failed to get spending ledger: {}",
            e
        ))),
    }
}

// -------------------------------------------------------------------------
// Invitation Code CRUD
// -------------------------------------------------------------------------

pub fn insert_invitation_code(db: &Database, invitation: &InvitationCode) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "INSERT INTO invitation_codes (code, created_at, expires_at, used_by, used_at, max_uses, use_count, label)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            invitation.code,
            invitation.created_at,
            invitation.expires_at,
            invitation.used_by,
            invitation.used_at,
            invitation.max_uses,
            invitation.use_count,
            invitation.label,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to insert invitation code: {}", e)))?;
    Ok(())
}

pub fn get_invitation_code(db: &Database, code: &str) -> Result<InvitationCode, AppError> {
    let conn = db.get_connection()?;
    conn.query_row(
        "SELECT code, created_at, expires_at, used_by, used_at, max_uses, use_count, label
         FROM invitation_codes WHERE code = ?1",
        params![code],
        |row| {
            Ok(InvitationCode {
                code: row.get(0)?,
                created_at: row.get(1)?,
                expires_at: row.get(2)?,
                used_by: row.get(3)?,
                used_at: row.get(4)?,
                max_uses: row.get(5)?,
                use_count: row.get(6)?,
                label: row.get(7)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Invitation code not found: {}", code))
        }
        _ => AppError::DatabaseError(format!("Failed to get invitation code: {}", e)),
    })
}

pub fn use_invitation_code(
    db: &Database,
    code: &str,
    agent_id: &str,
    used_at: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute(
            "UPDATE invitation_codes SET used_by = ?1, used_at = ?2, use_count = use_count + 1
             WHERE code = ?3 AND use_count < max_uses",
            params![agent_id, used_at, code],
        )
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to use invitation code: {}", e))
        })?;
    if rows == 0 {
        return Err(AppError::InvalidInput(format!(
            "Invitation code '{}' is not available (not found or already used)",
            code
        )));
    }
    Ok(())
}

pub fn count_active_invitation_codes(db: &Database) -> Result<usize, AppError> {
    let conn = db.get_connection()?;
    let now = chrono::Utc::now().timestamp();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invitation_codes
             WHERE use_count < max_uses AND (expires_at IS NULL OR expires_at > ?1)",
            params![now],
            |row| row.get(0),
        )
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to count active invitation codes: {}", e))
        })?;
    Ok(count as usize)
}

pub fn revoke_invitation_code(db: &Database, code: &str) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute(
            "UPDATE invitation_codes SET max_uses = use_count WHERE code = ?1",
            params![code],
        )
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to revoke invitation code: {}", e))
        })?;
    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Invitation code not found: {}",
            code
        )));
    }
    Ok(())
}

pub fn list_active_invitation_codes(db: &Database) -> Result<Vec<InvitationCode>, AppError> {
    let conn = db.get_connection()?;
    let now = chrono::Utc::now().timestamp();
    let mut stmt = conn
        .prepare(
            "SELECT code, created_at, expires_at, used_by, used_at, max_uses, use_count, label
             FROM invitation_codes
             WHERE use_count < max_uses AND (expires_at IS NULL OR expires_at > ?1)",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let codes = stmt
        .query_map(params![now], |row| {
            Ok(InvitationCode {
                code: row.get(0)?,
                created_at: row.get(1)?,
                expires_at: row.get(2)?,
                used_by: row.get(3)?,
                used_at: row.get(4)?,
                max_uses: row.get(5)?,
                use_count: row.get(6)?,
                label: row.get(7)?,
            })
        })
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to query active invitation codes: {}", e))
        })?;

    codes
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect invitation codes: {}", e)))
}

pub fn list_all_invitation_codes(db: &Database) -> Result<Vec<InvitationCode>, AppError> {
    let conn = db.get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT code, created_at, expires_at, used_by, used_at, max_uses, use_count, label
             FROM invitation_codes
             ORDER BY created_at DESC",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;

    let codes = stmt
        .query_map([], |row| {
            Ok(InvitationCode {
                code: row.get(0)?,
                created_at: row.get(1)?,
                expires_at: row.get(2)?,
                used_by: row.get(3)?,
                used_at: row.get(4)?,
                max_uses: row.get(5)?,
                use_count: row.get(6)?,
                label: row.get(7)?,
            })
        })
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to query invitation codes: {}", e))
        })?;

    codes
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect invitation codes: {}", e)))
}

pub fn delete_invitation_code(db: &Database, code: &str) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute(
            "DELETE FROM invitation_codes WHERE code = ?1",
            params![code],
        )
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to delete invitation code: {}", e))
        })?;
    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Invitation code not found: {}",
            code
        )));
    }
    Ok(())
}

// -------------------------------------------------------------------------
// Global Policy CRUD
// -------------------------------------------------------------------------

pub fn upsert_global_policy(db: &Database, policy: &GlobalPolicy) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "INSERT INTO global_policy (id, daily_cap, weekly_cap, monthly_cap, min_reserve_balance,
         kill_switch_active, kill_switch_reason, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
           daily_cap = ?2, weekly_cap = ?3, monthly_cap = ?4,
           min_reserve_balance = ?5, kill_switch_active = ?6,
           kill_switch_reason = ?7, updated_at = ?8",
        params![
            policy.id,
            policy.daily_cap,
            policy.weekly_cap,
            policy.monthly_cap,
            policy.min_reserve_balance,
            policy.kill_switch_active as i32,
            policy.kill_switch_reason,
            policy.updated_at,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to upsert global policy: {}", e)))?;
    Ok(())
}

pub fn get_global_policy(db: &Database) -> Result<Option<GlobalPolicy>, AppError> {
    let conn = db.get_connection()?;
    match conn.query_row(
        "SELECT id, daily_cap, weekly_cap, monthly_cap, min_reserve_balance,
         kill_switch_active, kill_switch_reason, updated_at
         FROM global_policy WHERE id = 'default'",
        [],
        |row| {
            let kill_switch: i32 = row.get(5)?;
            Ok(GlobalPolicy {
                id: row.get(0)?,
                daily_cap: row.get(1)?,
                weekly_cap: row.get(2)?,
                monthly_cap: row.get(3)?,
                min_reserve_balance: row.get(4)?,
                kill_switch_active: kill_switch != 0,
                kill_switch_reason: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    ) {
        Ok(policy) => Ok(Some(policy)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(format!(
            "Failed to get global policy: {}",
            e
        ))),
    }
}

// -------------------------------------------------------------------------
// Global Spending Ledger CRUD
// -------------------------------------------------------------------------

pub fn upsert_global_spending_ledger(
    db: &Database,
    period: &str,
    amount: &str,
    updated_at: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute_batch("BEGIN EXCLUSIVE")
        .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

    let result = conn.execute(
        "INSERT INTO global_spending_ledger (period, total, tx_count, updated_at)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(period) DO UPDATE SET
           total = CAST((CAST(global_spending_ledger.total AS REAL) + CAST(?2 AS REAL)) AS TEXT),
           tx_count = global_spending_ledger.tx_count + 1,
           updated_at = ?3",
        params![period, amount, updated_at],
    );

    match result {
        Ok(_) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::DatabaseError(format!("Failed to commit: {}", e)))?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(AppError::DatabaseError(format!(
                "Failed to upsert global spending ledger: {}",
                e
            )))
        }
    }
}

pub fn get_global_spending_for_period(
    db: &Database,
    period: &str,
) -> Result<Option<GlobalSpendingLedger>, AppError> {
    let conn = db.get_connection()?;
    match conn.query_row(
        "SELECT period, total, tx_count, updated_at
         FROM global_spending_ledger WHERE period = ?1",
        params![period],
        |row| {
            Ok(GlobalSpendingLedger {
                period: row.get(0)?,
                total: row.get(1)?,
                tx_count: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    ) {
        Ok(ledger) => Ok(Some(ledger)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(format!(
            "Failed to get global spending ledger: {}",
            e
        ))),
    }
}

// -------------------------------------------------------------------------
// Transaction Status Updates
// -------------------------------------------------------------------------

pub fn update_transaction_status(
    db: &Database,
    tx_id: &str,
    status: &TxStatus,
    chain_tx_hash: Option<&str>,
    error_message: Option<&str>,
    updated_at: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute(
            "UPDATE transactions SET status = ?1, chain_tx_hash = ?2, error_message = ?3, updated_at = ?4
             WHERE id = ?5",
            params![status.to_string(), chain_tx_hash, error_message, updated_at, tx_id],
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to update transaction status: {}", e)))?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Transaction not found: {}", tx_id)));
    }
    Ok(())
}

/// Atomically update transaction to confirmed and upsert both spending ledgers.
/// Uses BEGIN EXCLUSIVE to ensure atomicity.
pub fn update_transaction_and_ledgers_atomic(
    db: &Database,
    tx_id: &str,
    chain_tx_hash: &str,
    agent_id: &str,
    amount: &str,
    period_daily: &str,
    period_weekly: &str,
    period_monthly: &str,
    updated_at: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute_batch("BEGIN EXCLUSIVE")
        .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

    // Update transaction status to confirmed
    let result = (|| -> Result<(), AppError> {
        conn.execute(
            "UPDATE transactions SET status = 'confirmed', chain_tx_hash = ?1, updated_at = ?2
             WHERE id = ?3",
            params![chain_tx_hash, updated_at, tx_id],
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to update tx status: {}", e)))?;

        // Upsert agent spending ledger for all three periods
        for period in &[period_daily, period_weekly, period_monthly] {
            conn.execute(
                "INSERT INTO spending_ledger (agent_id, period, total, tx_count, updated_at)
                 VALUES (?1, ?2, ?3, 1, ?4)
                 ON CONFLICT(agent_id, period) DO UPDATE SET
                   total = CAST((CAST(spending_ledger.total AS REAL) + CAST(?3 AS REAL)) AS TEXT),
                   tx_count = spending_ledger.tx_count + 1,
                   updated_at = ?4",
                params![agent_id, period, amount, updated_at],
            )
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to upsert spending ledger: {}", e))
            })?;
        }

        // Upsert global spending ledger for all three periods
        for period in &[period_daily, period_weekly, period_monthly] {
            conn.execute(
                "INSERT INTO global_spending_ledger (period, total, tx_count, updated_at)
                 VALUES (?1, ?2, 1, ?3)
                 ON CONFLICT(period) DO UPDATE SET
                   total = CAST((CAST(global_spending_ledger.total AS REAL) + CAST(?2 AS REAL)) AS TEXT),
                   tx_count = global_spending_ledger.tx_count + 1,
                   updated_at = ?3",
                params![period, amount, updated_at],
            )
            .map_err(|e| {
                AppError::DatabaseError(format!(
                    "Failed to upsert global spending ledger: {}",
                    e
                ))
            })?;
        }

        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::DatabaseError(format!("Failed to commit: {}", e)))?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

// -------------------------------------------------------------------------
// Notification Preferences CRUD
// -------------------------------------------------------------------------

pub fn upsert_notification_preferences(
    db: &Database,
    prefs: &NotificationPreferences,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "INSERT INTO notification_preferences (id, enabled, on_all_tx, on_large_tx,
         large_tx_threshold, on_errors, on_limit_requests, on_agent_registration)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
           enabled = ?2, on_all_tx = ?3, on_large_tx = ?4,
           large_tx_threshold = ?5, on_errors = ?6, on_limit_requests = ?7,
           on_agent_registration = ?8",
        params![
            prefs.id,
            prefs.enabled as i32,
            prefs.on_all_tx as i32,
            prefs.on_large_tx as i32,
            prefs.large_tx_threshold,
            prefs.on_errors as i32,
            prefs.on_limit_requests as i32,
            prefs.on_agent_registration as i32,
        ],
    )
    .map_err(|e| {
        AppError::DatabaseError(format!(
            "Failed to upsert notification preferences: {}",
            e
        ))
    })?;
    Ok(())
}

pub fn get_notification_preferences(
    db: &Database,
) -> Result<Option<NotificationPreferences>, AppError> {
    let conn = db.get_connection()?;
    match conn.query_row(
        "SELECT id, enabled, on_all_tx, on_large_tx, large_tx_threshold, on_errors,
         on_limit_requests, on_agent_registration
         FROM notification_preferences WHERE id = 'default'",
        [],
        |row| {
            let enabled: i32 = row.get(1)?;
            let on_all_tx: i32 = row.get(2)?;
            let on_large_tx: i32 = row.get(3)?;
            let on_errors: i32 = row.get(5)?;
            let on_limit_requests: i32 = row.get(6)?;
            let on_agent_registration: i32 = row.get(7)?;
            Ok(NotificationPreferences {
                id: row.get(0)?,
                enabled: enabled != 0,
                on_all_tx: on_all_tx != 0,
                on_large_tx: on_large_tx != 0,
                large_tx_threshold: row.get(4)?,
                on_errors: on_errors != 0,
                on_limit_requests: on_limit_requests != 0,
                on_agent_registration: on_agent_registration != 0,
            })
        },
    ) {
        Ok(prefs) => Ok(Some(prefs)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(format!(
            "Failed to get notification preferences: {}",
            e
        ))),
    }
}

// -------------------------------------------------------------------------
// App Config CRUD
// -------------------------------------------------------------------------

pub fn set_app_config(db: &Database, key: &str, value: &str) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "INSERT INTO app_config (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to set app config: {}", e)))?;
    Ok(())
}

pub fn get_app_config(db: &Database, key: &str) -> Result<Option<String>, AppError> {
    let conn = db.get_connection()?;
    match conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        params![key],
        |row| row.get(0),
    ) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(format!(
            "Failed to get app config: {}",
            e
        ))),
    }
}

pub fn delete_app_config(db: &Database, key: &str) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute("DELETE FROM app_config WHERE key = ?1", params![key])
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete app config: {}", e)))?;
    Ok(())
}

// -------------------------------------------------------------------------
// Approval Request CRUD
// -------------------------------------------------------------------------

pub fn insert_approval_request(
    db: &Database,
    request: &ApprovalRequest,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "INSERT INTO approval_requests (id, agent_id, request_type, payload, status, tx_id,
         expires_at, created_at, resolved_at, resolved_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            request.id,
            request.agent_id,
            request.request_type.to_string(),
            request.payload,
            request.status.to_string(),
            request.tx_id,
            request.expires_at,
            request.created_at,
            request.resolved_at,
            request.resolved_by,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to insert approval request: {}", e)))?;
    Ok(())
}

pub fn get_approval_request_by_agent(
    db: &Database,
    agent_id: &str,
    request_type: &ApprovalRequestType,
) -> Result<Option<ApprovalRequest>, AppError> {
    let conn = db.get_connection()?;
    match conn.query_row(
        "SELECT id, agent_id, request_type, payload, status, tx_id, expires_at,
         created_at, resolved_at, resolved_by
         FROM approval_requests WHERE agent_id = ?1 AND request_type = ?2
         ORDER BY created_at DESC LIMIT 1",
        params![agent_id, request_type.to_string()],
        |row| row_to_approval_request(row),
    ) {
        Ok(req) => Ok(Some(req)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(format!(
            "Failed to get approval request: {}",
            e
        ))),
    }
}

fn row_to_approval_request(row: &rusqlite::Row<'_>) -> rusqlite::Result<ApprovalRequest> {
    let request_type_str: String = row.get(2)?;
    let request_type = match request_type_str.as_str() {
        "transaction" => ApprovalRequestType::Transaction,
        "limit_increase" => ApprovalRequestType::LimitIncrease,
        _ => ApprovalRequestType::Registration,
    };
    let status_str: String = row.get(4)?;
    let status = match status_str.as_str() {
        "approved" => ApprovalStatus::Approved,
        "denied" => ApprovalStatus::Denied,
        "expired" => ApprovalStatus::Expired,
        _ => ApprovalStatus::Pending,
    };
    Ok(ApprovalRequest {
        id: row.get(0)?,
        agent_id: row.get(1)?,
        request_type,
        payload: row.get(3)?,
        status,
        tx_id: row.get(5)?,
        expires_at: row.get(6)?,
        created_at: row.get(7)?,
        resolved_at: row.get(8)?,
        resolved_by: row.get(9)?,
    })
}

pub fn get_approval_request(db: &Database, id: &str) -> Result<ApprovalRequest, AppError> {
    let conn = db.get_connection()?;
    conn.query_row(
        "SELECT id, agent_id, request_type, payload, status, tx_id, expires_at,
         created_at, resolved_at, resolved_by
         FROM approval_requests WHERE id = ?1",
        params![id],
        |row| row_to_approval_request(row),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Approval request not found: {}", id))
        }
        _ => AppError::DatabaseError(format!("Failed to get approval request: {}", e)),
    })
}

pub fn list_pending_approvals(
    db: &Database,
    agent_id: Option<&str>,
) -> Result<Vec<ApprovalRequest>, AppError> {
    let conn = db.get_connection()?;
    match agent_id {
        Some(aid) => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, agent_id, request_type, payload, status, tx_id, expires_at,
                     created_at, resolved_at, resolved_by
                     FROM approval_requests WHERE status = 'pending' AND agent_id = ?1
                     ORDER BY created_at DESC",
                )
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to prepare statement: {}", e))
                })?;
            let rows = stmt
                .query_map(params![aid], |row| row_to_approval_request(row))
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to query approvals: {}", e))
                })?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::DatabaseError(format!("Failed to collect approvals: {}", e)))
        }
        None => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, agent_id, request_type, payload, status, tx_id, expires_at,
                     created_at, resolved_at, resolved_by
                     FROM approval_requests WHERE status = 'pending'
                     ORDER BY created_at DESC",
                )
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to prepare statement: {}", e))
                })?;
            let rows = stmt
                .query_map([], |row| row_to_approval_request(row))
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to query approvals: {}", e))
                })?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::DatabaseError(format!("Failed to collect approvals: {}", e)))
        }
    }
}

pub fn update_approval_status(
    db: &Database,
    id: &str,
    status: &ApprovalStatus,
    resolved_at: Option<i64>,
    resolved_by: Option<&str>,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute(
            "UPDATE approval_requests SET status = ?1, resolved_at = ?2, resolved_by = ?3 WHERE id = ?4",
            params![status.to_string(), resolved_at, resolved_by, id],
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to update approval status: {}", e)))?;
    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Approval request not found: {}",
            id
        )));
    }
    Ok(())
}

pub fn list_expired_approvals(
    db: &Database,
    now_timestamp: i64,
) -> Result<Vec<ApprovalRequest>, AppError> {
    let conn = db.get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, request_type, payload, status, tx_id, expires_at,
             created_at, resolved_at, resolved_by
             FROM approval_requests WHERE status = 'pending' AND expires_at < ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to prepare statement: {}", e)))?;
    let rows = stmt
        .query_map(params![now_timestamp], |row| row_to_approval_request(row))
        .map_err(|e| AppError::DatabaseError(format!("Failed to query expired approvals: {}", e)))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::DatabaseError(format!("Failed to collect expired approvals: {}", e)))
}

pub fn list_approvals(
    db: &Database,
    status: Option<&ApprovalStatus>,
) -> Result<Vec<ApprovalRequest>, AppError> {
    let conn = db.get_connection()?;
    match status {
        Some(s) => {
            let status_str = s.to_string();
            let mut stmt = conn
                .prepare(
                    "SELECT id, agent_id, request_type, payload, status, tx_id, expires_at,
                     created_at, resolved_at, resolved_by
                     FROM approval_requests WHERE status = ?1
                     ORDER BY created_at DESC",
                )
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to prepare statement: {}", e))
                })?;
            let rows = stmt
                .query_map(params![status_str], |row| row_to_approval_request(row))
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to query approvals: {}", e))
                })?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::DatabaseError(format!("Failed to collect approvals: {}", e)))
        }
        None => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, agent_id, request_type, payload, status, tx_id, expires_at,
                     created_at, resolved_at, resolved_by
                     FROM approval_requests
                     ORDER BY created_at DESC",
                )
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to prepare statement: {}", e))
                })?;
            let rows = stmt
                .query_map([], |row| row_to_approval_request(row))
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to query approvals: {}", e))
                })?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::DatabaseError(format!("Failed to collect approvals: {}", e)))
        }
    }
}

// -------------------------------------------------------------------------
// Token Delivery CRUD
// -------------------------------------------------------------------------

pub fn insert_token_delivery(db: &Database, delivery: &TokenDelivery) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "INSERT INTO token_delivery (agent_id, encrypted_token, created_at, expires_at, delivered)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            delivery.agent_id,
            delivery.encrypted_token,
            delivery.created_at,
            delivery.expires_at,
            delivery.delivered as i32,
        ],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to insert token delivery: {}", e)))?;
    Ok(())
}

pub fn get_token_delivery(db: &Database, agent_id: &str) -> Result<Option<TokenDelivery>, AppError> {
    let conn = db.get_connection()?;
    match conn.query_row(
        "SELECT agent_id, encrypted_token, created_at, expires_at, delivered
         FROM token_delivery WHERE agent_id = ?1",
        params![agent_id],
        |row| {
            let delivered: i32 = row.get(4)?;
            Ok(TokenDelivery {
                agent_id: row.get(0)?,
                encrypted_token: row.get(1)?,
                created_at: row.get(2)?,
                expires_at: row.get(3)?,
                delivered: delivered != 0,
            })
        },
    ) {
        Ok(delivery) => Ok(Some(delivery)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(format!(
            "Failed to get token delivery: {}",
            e
        ))),
    }
}

pub fn delete_token_delivery(db: &Database, agent_id: &str) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute(
        "DELETE FROM token_delivery WHERE agent_id = ?1",
        params![agent_id],
    )
    .map_err(|e| AppError::DatabaseError(format!("Failed to delete token delivery: {}", e)))?;
    Ok(())
}

// -------------------------------------------------------------------------
// Agent Token Update
// -------------------------------------------------------------------------

pub fn update_agent_token(
    db: &Database,
    agent_id: &str,
    token_hash: &str,
    token_prefix: &str,
    updated_at: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    let rows = conn
        .execute(
            "UPDATE agents SET api_token_hash = ?1, token_prefix = ?2, updated_at = ?3 WHERE id = ?4",
            params![token_hash, token_prefix, updated_at, agent_id],
        )
        .map_err(|e| AppError::DatabaseError(format!("Failed to update agent token: {}", e)))?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Agent not found: {}", agent_id)));
    }
    Ok(())
}

// -------------------------------------------------------------------------
// Atomic Policy Check + Reserve (TOCTOU fix)
// -------------------------------------------------------------------------

use std::str::FromStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Result of an atomic policy check + reservation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AtomicPolicyResult {
    AutoApproved,
    RequiresApproval { reason: String },
    Denied { reason: String },
}

/// Atomically: read ledger + check policy + reserve spend in a single BEGIN EXCLUSIVE.
///
/// For AutoApproved: reserves the amount in the ledger immediately.
/// For RequiresApproval: reserves the amount (will be rolled back if denied/expired).
/// For Denied: no ledger changes.
///
/// This eliminates the TOCTOU race condition where concurrent requests could all
/// pass policy checks before any ledger updates occur.
pub fn check_policy_and_reserve_atomic(
    db: &Database,
    agent_id: &str,
    amount: &str,
    recipient: &str,
    current_balance: &str,
    period_daily: &str,
    period_weekly: &str,
    period_monthly: &str,
    now_ts: i64,
) -> Result<AtomicPolicyResult, AppError> {
    let conn = db.get_connection()?;

    // Parse the amount
    let amount_dec = Decimal::from_str(amount)
        .map_err(|e| AppError::Internal(format!("Invalid amount: {}", e)))?;
    let balance_dec = Decimal::from_str(current_balance)
        .map_err(|e| AppError::Internal(format!("Invalid balance: {}", e)))?;

    // BEGIN EXCLUSIVE — blocks all other writers until we COMMIT/ROLLBACK
    conn.execute_batch("BEGIN EXCLUSIVE")
        .map_err(|e| AppError::DatabaseError(format!("Failed to begin exclusive: {}", e)))?;

    let result = (|| -> Result<AtomicPolicyResult, AppError> {
        // 1. Read spending policy for this agent
        let policy = conn.query_row(
            "SELECT agent_id, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max, allowlist, updated_at
             FROM spending_policies WHERE agent_id = ?1",
            params![agent_id],
            |row| {
                let allowlist_str: String = row.get(6)?;
                let allowlist: Vec<String> =
                    serde_json::from_str(&allowlist_str).unwrap_or_default();
                Ok(SpendingPolicy {
                    agent_id: row.get(0)?,
                    per_tx_max: row.get(1)?,
                    daily_cap: row.get(2)?,
                    weekly_cap: row.get(3)?,
                    monthly_cap: row.get(4)?,
                    auto_approve_max: row.get(5)?,
                    allowlist,
                    updated_at: row.get(7)?,
                })
            },
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Spending policy not found for agent: {}", agent_id))
            }
            _ => AppError::DatabaseError(format!("Failed to get spending policy: {}", e)),
        })?;

        let per_tx_max = Decimal::from_str(&policy.per_tx_max)
            .map_err(|e| AppError::Internal(format!("Invalid per_tx_max: {}", e)))?;
        let daily_cap = Decimal::from_str(&policy.daily_cap)
            .map_err(|e| AppError::Internal(format!("Invalid daily_cap: {}", e)))?;
        let weekly_cap = Decimal::from_str(&policy.weekly_cap)
            .map_err(|e| AppError::Internal(format!("Invalid weekly_cap: {}", e)))?;
        let monthly_cap = Decimal::from_str(&policy.monthly_cap)
            .map_err(|e| AppError::Internal(format!("Invalid monthly_cap: {}", e)))?;
        let auto_approve_max = Decimal::from_str(&policy.auto_approve_max)
            .map_err(|e| AppError::Internal(format!("Invalid auto_approve_max: {}", e)))?;

        // 2. Check per_tx_max
        if amount_dec > per_tx_max {
            return Ok(AtomicPolicyResult::Denied {
                reason: format!("Amount {} exceeds per-tx limit of {}", amount_dec, per_tx_max),
            });
        }

        // 3. Check allowlist
        if !policy.allowlist.is_empty()
            && !policy.allowlist.iter().any(|a| a == recipient)
        {
            return Ok(AtomicPolicyResult::Denied {
                reason: "Recipient not in allowlist".to_string(),
            });
        }

        // 4. Read agent spending ledger for daily period (inside the exclusive tx)
        let daily_spent = read_ledger_total(&conn, agent_id, period_daily)?;
        if daily_spent + amount_dec > daily_cap {
            return Ok(AtomicPolicyResult::Denied {
                reason: format!(
                    "Amount {} would exceed daily cap of {} (already spent {})",
                    amount_dec, daily_cap, daily_spent
                ),
            });
        }

        // 5. Read agent spending ledger for weekly period
        let weekly_spent = read_ledger_total(&conn, agent_id, period_weekly)?;
        if weekly_spent + amount_dec > weekly_cap {
            return Ok(AtomicPolicyResult::Denied {
                reason: format!(
                    "Amount {} would exceed weekly cap of {} (already spent {})",
                    amount_dec, weekly_cap, weekly_spent
                ),
            });
        }

        // 6. Read agent spending ledger for monthly period
        let monthly_spent = read_ledger_total(&conn, agent_id, period_monthly)?;
        if monthly_spent + amount_dec > monthly_cap {
            return Ok(AtomicPolicyResult::Denied {
                reason: format!(
                    "Amount {} would exceed monthly cap of {} (already spent {})",
                    amount_dec, monthly_cap, monthly_spent
                ),
            });
        }

        // 7. Read global policy (optional)
        let global_policy = match conn.query_row(
            "SELECT id, daily_cap, weekly_cap, monthly_cap, min_reserve_balance,
             kill_switch_active, kill_switch_reason, updated_at
             FROM global_policy WHERE id = 'default'",
            [],
            |row| {
                let kill_switch: i32 = row.get(5)?;
                Ok(GlobalPolicy {
                    id: row.get(0)?,
                    daily_cap: row.get(1)?,
                    weekly_cap: row.get(2)?,
                    monthly_cap: row.get(3)?,
                    min_reserve_balance: row.get(4)?,
                    kill_switch_active: kill_switch != 0,
                    kill_switch_reason: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        ) {
            Ok(p) => Some(p),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(AppError::DatabaseError(format!("Failed to get global policy: {}", e))),
        };

        // 8. Evaluate global policy if it exists
        if let Some(gp) = &global_policy {
            // Kill switch
            if gp.kill_switch_active {
                return Ok(AtomicPolicyResult::Denied {
                    reason: format!("Emergency kill switch active: {}", gp.kill_switch_reason),
                });
            }

            // Min reserve balance
            let min_reserve: Decimal = gp.min_reserve_balance.parse()
                .map_err(|e| AppError::Internal(format!("Invalid min_reserve_balance: {}", e)))?;
            if min_reserve > Decimal::ZERO && balance_dec - amount_dec < min_reserve {
                return Ok(AtomicPolicyResult::Denied {
                    reason: format!("Would drop balance below minimum reserve of {}", min_reserve),
                });
            }

            // Global daily cap
            let g_daily_cap: Decimal = gp.daily_cap.parse()
                .map_err(|e| AppError::Internal(format!("Invalid global daily_cap: {}", e)))?;
            if g_daily_cap > Decimal::ZERO {
                let g_daily_spent = read_global_ledger_total(&conn, period_daily)?;
                if g_daily_spent + amount_dec > g_daily_cap {
                    return Ok(AtomicPolicyResult::Denied {
                        reason: format!("Global daily spending cap of {} would be exceeded", g_daily_cap),
                    });
                }
            }

            // Global weekly cap
            let g_weekly_cap: Decimal = gp.weekly_cap.parse()
                .map_err(|e| AppError::Internal(format!("Invalid global weekly_cap: {}", e)))?;
            if g_weekly_cap > Decimal::ZERO {
                let g_weekly_spent = read_global_ledger_total(&conn, period_weekly)?;
                if g_weekly_spent + amount_dec > g_weekly_cap {
                    return Ok(AtomicPolicyResult::Denied {
                        reason: format!("Global weekly spending cap of {} would be exceeded", g_weekly_cap),
                    });
                }
            }

            // Global monthly cap
            let g_monthly_cap: Decimal = gp.monthly_cap.parse()
                .map_err(|e| AppError::Internal(format!("Invalid global monthly_cap: {}", e)))?;
            if g_monthly_cap > Decimal::ZERO {
                let g_monthly_spent = read_global_ledger_total(&conn, period_monthly)?;
                if g_monthly_spent + amount_dec > g_monthly_cap {
                    return Ok(AtomicPolicyResult::Denied {
                        reason: format!("Global monthly spending cap of {} would be exceeded", g_monthly_cap),
                    });
                }
            }
        }

        // 9. Determine auto-approve vs requires-approval
        let decision = if amount_dec <= auto_approve_max {
            AtomicPolicyResult::AutoApproved
        } else {
            AtomicPolicyResult::RequiresApproval {
                reason: format!(
                    "Amount {} exceeds auto-approve threshold of {}",
                    amount_dec, auto_approve_max
                ),
            }
        };

        // 10. Reserve: UPSERT agent spending ledger for all three periods
        for period in &[period_daily, period_weekly, period_monthly] {
            conn.execute(
                "INSERT INTO spending_ledger (agent_id, period, total, tx_count, updated_at)
                 VALUES (?1, ?2, ?3, 1, ?4)
                 ON CONFLICT(agent_id, period) DO UPDATE SET
                   total = CAST((CAST(spending_ledger.total AS REAL) + CAST(?3 AS REAL)) AS TEXT),
                   tx_count = spending_ledger.tx_count + 1,
                   updated_at = ?4",
                params![agent_id, period, amount, now_ts],
            ).map_err(|e| {
                AppError::DatabaseError(format!("Failed to upsert spending ledger: {}", e))
            })?;
        }

        // 11. Reserve: UPSERT global spending ledger for all three periods
        for period in &[period_daily, period_weekly, period_monthly] {
            conn.execute(
                "INSERT INTO global_spending_ledger (period, total, tx_count, updated_at)
                 VALUES (?1, ?2, 1, ?3)
                 ON CONFLICT(period) DO UPDATE SET
                   total = CAST((CAST(global_spending_ledger.total AS REAL) + CAST(?2 AS REAL)) AS TEXT),
                   tx_count = global_spending_ledger.tx_count + 1,
                   updated_at = ?3",
                params![period, amount, now_ts],
            ).map_err(|e| {
                AppError::DatabaseError(format!("Failed to upsert global spending ledger: {}", e))
            })?;
        }

        Ok(decision)
    })();

    match &result {
        Ok(AtomicPolicyResult::Denied { .. }) => {
            // Denied — rollback (no ledger changes)
            let _ = conn.execute_batch("ROLLBACK");
        }
        Ok(_) => {
            // AutoApproved or RequiresApproval — commit the reservation
            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::DatabaseError(format!("Failed to commit reservation: {}", e)))?;
        }
        Err(_) => {
            // Error — rollback
            let _ = conn.execute_batch("ROLLBACK");
        }
    }

    result
}

/// Roll back a spending reservation when CLI fails or approval is denied.
/// Decrements the spending ledger entries that were reserved.
pub fn rollback_reservation(
    db: &Database,
    agent_id: &str,
    amount: &str,
    period_daily: &str,
    period_weekly: &str,
    period_monthly: &str,
    _now_ts: i64,
) -> Result<(), AppError> {
    let conn = db.get_connection()?;
    conn.execute_batch("BEGIN EXCLUSIVE")
        .map_err(|e| AppError::DatabaseError(format!("Failed to begin exclusive: {}", e)))?;

    let result = (|| -> Result<(), AppError> {
        // Decrement agent spending ledger for all three periods
        for period in &[period_daily, period_weekly, period_monthly] {
            conn.execute(
                "UPDATE spending_ledger SET
                   total = CAST((CAST(spending_ledger.total AS REAL) - CAST(?3 AS REAL)) AS TEXT),
                   tx_count = spending_ledger.tx_count - 1
                 WHERE agent_id = ?1 AND period = ?2",
                params![agent_id, period, amount],
            ).map_err(|e| {
                AppError::DatabaseError(format!("Failed to decrement spending ledger: {}", e))
            })?;
        }

        // Decrement global spending ledger for all three periods
        for period in &[period_daily, period_weekly, period_monthly] {
            conn.execute(
                "UPDATE global_spending_ledger SET
                   total = CAST((CAST(global_spending_ledger.total AS REAL) - CAST(?2 AS REAL)) AS TEXT),
                   tx_count = global_spending_ledger.tx_count - 1
                 WHERE period = ?1",
                params![period, amount],
            ).map_err(|e| {
                AppError::DatabaseError(format!("Failed to decrement global spending ledger: {}", e))
            })?;
        }

        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::DatabaseError(format!("Failed to commit rollback: {}", e)))?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

/// Helper: read agent spending ledger total for a period within an existing transaction.
fn read_ledger_total(
    conn: &r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>,
    agent_id: &str,
    period: &str,
) -> Result<Decimal, AppError> {
    match conn.query_row(
        "SELECT total FROM spending_ledger WHERE agent_id = ?1 AND period = ?2",
        params![agent_id, period],
        |row| {
            let total: String = row.get(0)?;
            Ok(total)
        },
    ) {
        Ok(total_str) => Decimal::from_str(&total_str)
            .map_err(|e| AppError::Internal(format!("Invalid ledger total: {}", e))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Decimal::ZERO),
        Err(e) => Err(AppError::DatabaseError(format!("Failed to read spending ledger: {}", e))),
    }
}

/// Helper: read global spending ledger total for a period within an existing transaction.
fn read_global_ledger_total(
    conn: &r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>,
    period: &str,
) -> Result<Decimal, AppError> {
    match conn.query_row(
        "SELECT total FROM global_spending_ledger WHERE period = ?1",
        params![period],
        |row| {
            let total: String = row.get(0)?;
            Ok(total)
        },
    ) {
        Ok(total_str) => Decimal::from_str(&total_str)
            .map_err(|e| AppError::Internal(format!("Invalid global ledger total: {}", e))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Decimal::ZERO),
        Err(e) => Err(AppError::DatabaseError(format!("Failed to read global spending ledger: {}", e))),
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::Database;

    fn setup_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn make_agent(name: &str, status: AgentStatus) -> Agent {
        Agent {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: format!("Test agent: {}", name),
            purpose: "Testing".to_string(),
            agent_type: "test".to_string(),
            capabilities: vec!["send".to_string()],
            status,
            api_token_hash: None,
            token_prefix: None,
            balance_visible: true,
            invitation_code: None,
            created_at: 1000000,
            updated_at: 1000000,
            last_active_at: None,
            metadata: "{}".to_string(),
        }
    }

    fn make_transaction(agent_id: &str, amount: &str, status: TxStatus) -> Transaction {
        Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: Some(agent_id.to_string()),
            tx_type: TxType::Send,
            amount: amount.to_string(),
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

    #[test]
    fn test_insert_and_get_agent() {
        let db = setup_db();
        let agent = make_agent("TestBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let fetched = get_agent(&db, &agent.id).unwrap();
        assert_eq!(fetched.name, "TestBot");
        assert_eq!(fetched.status, AgentStatus::Active);
        assert_eq!(fetched.capabilities, vec!["send".to_string()]);
    }

    #[test]
    fn test_update_agent_status() {
        let db = setup_db();
        let agent = make_agent("StatusBot", AgentStatus::Pending);
        insert_agent(&db, &agent).unwrap();

        update_agent_status(&db, &agent.id, &AgentStatus::Active, 2000000).unwrap();
        let fetched = get_agent(&db, &agent.id).unwrap();
        assert_eq!(fetched.status, AgentStatus::Active);
        assert_eq!(fetched.updated_at, 2000000);
    }

    #[test]
    fn test_list_agents_by_status() {
        let db = setup_db();
        let a1 = make_agent("Active1", AgentStatus::Active);
        let a2 = make_agent("Active2", AgentStatus::Active);
        let a3 = make_agent("Pending1", AgentStatus::Pending);
        insert_agent(&db, &a1).unwrap();
        insert_agent(&db, &a2).unwrap();
        insert_agent(&db, &a3).unwrap();

        let active = list_agents_by_status(&db, &AgentStatus::Active).unwrap();
        assert_eq!(active.len(), 2);

        let pending = list_agents_by_status(&db, &AgentStatus::Pending).unwrap();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_delete_agent_cascades_spending_policy() {
        let db = setup_db();
        let agent = make_agent("CascadeBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let policy = SpendingPolicy {
            agent_id: agent.id.clone(),
            per_tx_max: "10".to_string(),
            daily_cap: "100".to_string(),
            weekly_cap: "500".to_string(),
            monthly_cap: "2000".to_string(),
            auto_approve_max: "5".to_string(),
            allowlist: vec![],
            updated_at: 1000000,
        };
        insert_spending_policy(&db, &policy).unwrap();

        // Verify policy exists
        assert!(get_spending_policy(&db, &agent.id).is_ok());

        // Delete agent -- should cascade to spending_policies
        delete_agent(&db, &agent.id).unwrap();

        // Spending policy should be gone
        assert!(get_spending_policy(&db, &agent.id).is_err());
    }

    #[test]
    fn test_insert_and_get_transaction() {
        let db = setup_db();
        let agent = make_agent("TxBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let tx = make_transaction(&agent.id, "25.50", TxStatus::Pending);
        insert_transaction(&db, &tx).unwrap();

        let fetched = get_transaction(&db, &tx.id).unwrap();
        assert_eq!(fetched.amount, "25.50");
        assert_eq!(fetched.status, TxStatus::Pending);
        assert_eq!(fetched.tx_type, TxType::Send);
    }

    #[test]
    fn test_list_transactions_by_agent() {
        let db = setup_db();
        let agent = make_agent("TxListBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let tx1 = make_transaction(&agent.id, "10.00", TxStatus::Pending);
        let tx2 = make_transaction(&agent.id, "20.00", TxStatus::Confirmed);
        insert_transaction(&db, &tx1).unwrap();
        insert_transaction(&db, &tx2).unwrap();

        let txs = list_transactions_by_agent(&db, &agent.id).unwrap();
        assert_eq!(txs.len(), 2);
    }

    #[test]
    fn test_list_transactions_by_status() {
        let db = setup_db();
        let agent = make_agent("TxStatusBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let tx1 = make_transaction(&agent.id, "10.00", TxStatus::Pending);
        let tx2 = make_transaction(&agent.id, "20.00", TxStatus::Pending);
        let tx3 = make_transaction(&agent.id, "30.00", TxStatus::Confirmed);
        insert_transaction(&db, &tx1).unwrap();
        insert_transaction(&db, &tx2).unwrap();
        insert_transaction(&db, &tx3).unwrap();

        let pending = list_transactions_by_status(&db, &TxStatus::Pending).unwrap();
        assert_eq!(pending.len(), 2);

        let confirmed = list_transactions_by_status(&db, &TxStatus::Confirmed).unwrap();
        assert_eq!(confirmed.len(), 1);
    }

    #[test]
    fn test_insert_and_get_spending_policy() {
        let db = setup_db();
        let agent = make_agent("PolicyBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let policy = SpendingPolicy {
            agent_id: agent.id.clone(),
            per_tx_max: "50".to_string(),
            daily_cap: "500".to_string(),
            weekly_cap: "2000".to_string(),
            monthly_cap: "8000".to_string(),
            auto_approve_max: "10".to_string(),
            allowlist: vec!["0xAllowed".to_string()],
            updated_at: 1000000,
        };
        insert_spending_policy(&db, &policy).unwrap();

        let fetched = get_spending_policy(&db, &agent.id).unwrap();
        assert_eq!(fetched.per_tx_max, "50");
        assert_eq!(fetched.allowlist, vec!["0xAllowed".to_string()]);
    }

    #[test]
    fn test_upsert_spending_ledger() {
        let db = setup_db();
        let agent = make_agent("LedgerBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        let period = "daily:2026-02-27";

        // First insert
        upsert_spending_ledger(&db, &agent.id, period, "10.00", 1000000).unwrap();
        let ledger = get_spending_for_period(&db, &agent.id, period)
            .unwrap()
            .unwrap();
        assert_eq!(ledger.total, "10.00");
        assert_eq!(ledger.tx_count, 1);

        // Upsert (add to existing)
        upsert_spending_ledger(&db, &agent.id, period, "5.50", 2000000).unwrap();
        let ledger = get_spending_for_period(&db, &agent.id, period)
            .unwrap()
            .unwrap();
        assert_eq!(ledger.total, "15.5");
        assert_eq!(ledger.tx_count, 2);
    }

    #[test]
    fn test_get_spending_for_period() {
        let db = setup_db();
        let agent = make_agent("SpendBot", AgentStatus::Active);
        insert_agent(&db, &agent).unwrap();

        // Non-existent period returns None
        let result = get_spending_for_period(&db, &agent.id, "daily:2099-01-01").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_insert_and_get_invitation_code() {
        let db = setup_db();
        let invitation = InvitationCode {
            code: "INV-TEST-001".to_string(),
            created_at: 1000000,
            expires_at: Some(2000000),
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "For testing".to_string(),
        };
        insert_invitation_code(&db, &invitation).unwrap();

        let fetched = get_invitation_code(&db, "INV-TEST-001").unwrap();
        assert_eq!(fetched.label, "For testing");
        assert_eq!(fetched.use_count, 0);
        assert!(fetched.used_by.is_none());
    }

    #[test]
    fn test_use_invitation_code() {
        let db = setup_db();
        let invitation = InvitationCode {
            code: "INV-USE-001".to_string(),
            created_at: 1000000,
            expires_at: None,
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: "One-time code".to_string(),
        };
        insert_invitation_code(&db, &invitation).unwrap();

        let agent = make_agent("InvBot", AgentStatus::Pending);
        insert_agent(&db, &agent).unwrap();

        // Use the code
        use_invitation_code(&db, "INV-USE-001", &agent.id, 1500000).unwrap();

        let fetched = get_invitation_code(&db, "INV-USE-001").unwrap();
        assert_eq!(fetched.use_count, 1);
        assert_eq!(fetched.used_by.unwrap(), agent.id);

        // Using it again should fail (max_uses = 1, use_count now = 1)
        let result = use_invitation_code(&db, "INV-USE-001", "other-agent", 1600000);
        assert!(result.is_err());
    }

    #[test]
    fn test_global_policy_crud() {
        let db = setup_db();

        // Initially no global policy
        let result = get_global_policy(&db).unwrap();
        assert!(result.is_none());

        // Insert
        let policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "10000".to_string(),
            weekly_cap: "50000".to_string(),
            monthly_cap: "200000".to_string(),
            min_reserve_balance: "100".to_string(),
            kill_switch_active: false,
            kill_switch_reason: "".to_string(),
            updated_at: 1000000,
        };
        upsert_global_policy(&db, &policy).unwrap();

        let fetched = get_global_policy(&db).unwrap().unwrap();
        assert_eq!(fetched.daily_cap, "10000");
        assert!(!fetched.kill_switch_active);

        // Update (upsert)
        let updated_policy = GlobalPolicy {
            id: "default".to_string(),
            daily_cap: "5000".to_string(),
            weekly_cap: "25000".to_string(),
            monthly_cap: "100000".to_string(),
            min_reserve_balance: "200".to_string(),
            kill_switch_active: true,
            kill_switch_reason: "Emergency".to_string(),
            updated_at: 2000000,
        };
        upsert_global_policy(&db, &updated_policy).unwrap();

        let fetched = get_global_policy(&db).unwrap().unwrap();
        assert_eq!(fetched.daily_cap, "5000");
        assert!(fetched.kill_switch_active);
        assert_eq!(fetched.kill_switch_reason, "Emergency");
    }

    #[test]
    fn test_notification_preferences_crud() {
        let db = setup_db();

        // Initially no preferences
        let result = get_notification_preferences(&db).unwrap();
        assert!(result.is_none());

        // Insert
        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: false,
            on_large_tx: true,
            large_tx_threshold: "25.00".to_string(),
            on_errors: true,
            on_limit_requests: true,
            on_agent_registration: true,
        };
        upsert_notification_preferences(&db, &prefs).unwrap();

        let fetched = get_notification_preferences(&db).unwrap().unwrap();
        assert!(fetched.enabled);
        assert!(!fetched.on_all_tx);
        assert_eq!(fetched.large_tx_threshold, "25.00");

        // Update (upsert)
        let updated = NotificationPreferences {
            id: "default".to_string(),
            enabled: false,
            on_all_tx: true,
            on_large_tx: false,
            large_tx_threshold: "50.00".to_string(),
            on_errors: false,
            on_limit_requests: false,
            on_agent_registration: false,
        };
        upsert_notification_preferences(&db, &updated).unwrap();

        let fetched = get_notification_preferences(&db).unwrap().unwrap();
        assert!(!fetched.enabled);
        assert!(fetched.on_all_tx);
    }

    #[test]
    fn test_app_config_crud() {
        let db = setup_db();

        // Initially empty
        let result = get_app_config(&db, "network").unwrap();
        assert!(result.is_none());

        // Set
        set_app_config(&db, "network", "base-sepolia").unwrap();
        let value = get_app_config(&db, "network").unwrap().unwrap();
        assert_eq!(value, "base-sepolia");

        // Update (upsert)
        set_app_config(&db, "network", "base-mainnet").unwrap();
        let value = get_app_config(&db, "network").unwrap().unwrap();
        assert_eq!(value, "base-mainnet");

        // Delete
        delete_app_config(&db, "network").unwrap();
        let result = get_app_config(&db, "network").unwrap();
        assert!(result.is_none());
    }
}
