use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use crate::error::AppError;

/// Database wraps an r2d2 connection pool for SQLite.
/// All DB calls should use `tokio::task::spawn_blocking` for async compatibility.
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Create a new Database backed by a file at `path`.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let manager = SqliteConnectionManager::file(path.as_ref());
        let pool = Pool::builder()
            .max_size(4)
            .build(manager)
            .map_err(|e| AppError::DatabaseError(format!("Failed to create pool: {}", e)))?;

        // Enable foreign keys on every connection
        let conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|e| AppError::DatabaseError(format!("Failed to set pragmas: {}", e)))?;

        Ok(Self { pool })
    }

    /// Create a new in-memory Database (useful for tests).
    pub fn new_in_memory() -> Result<Self, AppError> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(1) // in-memory DB must use single connection
            .build(manager)
            .map_err(|e| AppError::DatabaseError(format!("Failed to create pool: {}", e)))?;

        let conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| AppError::DatabaseError(format!("Failed to set pragmas: {}", e)))?;

        Ok(Self { pool })
    }

    /// Run all migrations. Currently only 001_initial.sql.
    /// Idempotent: uses CREATE TABLE IF NOT EXISTS. CREATE INDEX statements
    /// that fail with "already exists" are silently ignored.
    pub fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.get_connection()?;
        let migration_sql = include_str!("migrations/001_initial.sql");

        // Execute the full migration SQL as a batch.
        // But first, we need to handle CREATE INDEX without IF NOT EXISTS for idempotency.
        // Replace "CREATE INDEX" with "CREATE INDEX IF NOT EXISTS" for safe re-runs.
        let safe_sql = migration_sql.replace("CREATE INDEX ", "CREATE INDEX IF NOT EXISTS ");

        conn.execute_batch(&safe_sql)
            .map_err(|e| AppError::DatabaseError(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    /// Get a connection from the pool.
    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))
    }

    /// Check if a table exists in the database.
    pub fn table_exists(&self, table_name: &str) -> Result<bool, AppError> {
        let conn = self.get_connection()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                params![table_name],
                |row| row.get(0),
            )
            .map_err(|e| AppError::DatabaseError(format!("Failed to check table: {}", e)))?;
        Ok(count > 0)
    }

    /// Check if an index exists in the database.
    pub fn index_exists(&self, index_name: &str) -> Result<bool, AppError> {
        let conn = self.get_connection()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                params![index_name],
                |row| row.get(0),
            )
            .map_err(|e| AppError::DatabaseError(format!("Failed to check index: {}", e)))?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory_db() {
        let db = Database::new_in_memory();
        assert!(db.is_ok(), "Database::new_in_memory() should succeed");
    }

    #[test]
    fn test_run_migrations_creates_all_tables() {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();

        let expected_tables = [
            "app_config",
            "agents",
            "spending_policies",
            "global_policy",
            "global_spending_ledger",
            "transactions",
            "approval_requests",
            "invitation_codes",
            "token_delivery",
            "notification_preferences",
            "spending_ledger",
        ];

        for table in &expected_tables {
            assert!(
                db.table_exists(table).unwrap(),
                "Table '{}' should exist after migration",
                table
            );
        }
    }

    #[test]
    fn test_migrations_idempotent() {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        // Running migrations a second time should not error
        let result = db.run_migrations();
        assert!(result.is_ok(), "Running migrations twice should not error");
    }

    #[test]
    fn test_foreign_key_constraints_enforced() {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();

        let conn = db.get_connection().unwrap();

        // Inserting a spending_policy with a non-existent agent_id should fail
        let result = conn.execute(
            "INSERT INTO spending_policies (agent_id, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max, allowlist, updated_at)
             VALUES ('nonexistent_agent', '10', '100', '500', '2000', '5', '[]', 1000000)",
            [],
        );

        assert!(
            result.is_err(),
            "Foreign key constraint should reject non-existent agent_id"
        );
    }

    #[test]
    fn test_indexes_exist() {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();

        let expected_indexes = [
            "idx_agents_status",
            "idx_agents_token_hash",
            "idx_tx_agent",
            "idx_tx_status",
            "idx_tx_created",
            "idx_tx_type",
            "idx_approval_status",
            "idx_approval_agent",
            "idx_approval_expires",
        ];

        for index in &expected_indexes {
            assert!(
                db.index_exists(index).unwrap(),
                "Index '{}' should exist after migration",
                index
            );
        }
    }

    #[test]
    fn test_file_based_db() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let db_path = tmp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();
        db.run_migrations().unwrap();
        assert!(db.table_exists("agents").unwrap());
    }
}
