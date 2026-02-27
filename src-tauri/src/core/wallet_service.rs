use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::sync::RwLock;

use crate::cli::commands::AwalCommand;
use crate::cli::executor::{CliError, CliExecutable};
use crate::cli::parser::AssetBalance;
use crate::db::queries::get_agent;
use crate::db::schema::Database;
use crate::error::AppError;

// -------------------------------------------------------------------------
// CachedBalance
// -------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CachedBalance {
    pub address: String,
    pub chain: String,
    pub balances: HashMap<String, AssetBalance>,
    pub timestamp: String,
    pub fetched_at: Instant,
}

// -------------------------------------------------------------------------
// BalanceCache
// -------------------------------------------------------------------------

pub struct BalanceCache {
    cache: RwLock<Option<CachedBalance>>,
    ttl: Duration,
}

impl BalanceCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(None),
            ttl,
        }
    }

    /// Get the cached balance if it exists and is within TTL, otherwise fetch
    /// from CLI, cache it, and return the fresh value.
    ///
    /// Uses a read-lock-first, write-lock-with-double-check pattern to avoid
    /// thundering herd: if multiple callers arrive at an empty/expired cache,
    /// only the first one to acquire the write lock will call the CLI.
    pub async fn get_or_fetch(
        &self,
        cli: &dyn CliExecutable,
    ) -> Result<CachedBalance, AppError> {
        // 1. Fast path — read lock
        {
            let guard = self.cache.read().await;
            if let Some(ref cached) = *guard {
                if cached.fetched_at.elapsed() < self.ttl {
                    return Ok(cached.clone());
                }
            }
        }

        // 2. Slow path — write lock with double-check
        {
            let mut guard = self.cache.write().await;
            // Double-check: another task may have populated the cache while we waited
            if let Some(ref cached) = *guard {
                if cached.fetched_at.elapsed() < self.ttl {
                    return Ok(cached.clone());
                }
            }

            // Actually call the CLI
            let output = cli
                .run(AwalCommand::GetBalance { chain: None })
                .await
                .map_err(|e| match e {
                    CliError::Timeout => AppError::CliTimeout,
                    CliError::SessionExpired => AppError::CliSessionExpired,
                    CliError::NotFound(msg) => AppError::CliNotFound(msg),
                    CliError::CommandFailed { stderr, .. } => AppError::CliError(stderr),
                    CliError::ParseError(msg) => AppError::CliError(msg),
                })?;

            let parsed = crate::cli::parser::parse_balance(&output)
                .map_err(|e| AppError::CliError(format!("Balance parse error: {}", e)))?;

            let cached = CachedBalance {
                address: parsed.address,
                chain: parsed.chain,
                balances: parsed.balances,
                timestamp: parsed.timestamp,
                fetched_at: Instant::now(),
            };
            *guard = Some(cached.clone());
            Ok(cached)
        }
    }

    /// Invalidate the cache, forcing a fresh CLI call on the next request.
    pub async fn invalidate(&self) {
        let mut guard = self.cache.write().await;
        *guard = None;
    }
}

// -------------------------------------------------------------------------
// BalanceResponse
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct BalanceResponse {
    pub balance: Option<String>,
    pub asset: Option<String>,
    pub balances: Option<HashMap<String, AssetBalance>>,
    pub balance_visible: bool,
    pub cached: bool,
}

// -------------------------------------------------------------------------
// WalletService
// -------------------------------------------------------------------------

pub struct WalletService {
    cli: Arc<dyn CliExecutable>,
    db: Arc<Database>,
    cache: BalanceCache,
}

impl WalletService {
    pub fn new(cli: Arc<dyn CliExecutable>, db: Arc<Database>, ttl: Duration) -> Self {
        Self {
            cli,
            db,
            cache: BalanceCache::new(ttl),
        }
    }

    /// Get the wallet balance (cache-aware). Does not check per-agent visibility.
    pub async fn get_balance(&self) -> Result<CachedBalance, AppError> {
        self.cache.get_or_fetch(self.cli.as_ref()).await
    }

    /// Get the balance for a specific agent, respecting `balance_visible`.
    pub async fn get_balance_for_agent(
        &self,
        agent_id: &str,
    ) -> Result<BalanceResponse, AppError> {
        let db = self.db.clone();
        let aid = agent_id.to_string();
        let agent = tokio::task::spawn_blocking(move || get_agent(&db, &aid))
            .await
            .map_err(|e| AppError::Internal(format!("Spawn blocking failed: {}", e)))??;

        if !agent.balance_visible {
            return Ok(BalanceResponse {
                balance: None,
                asset: None,
                balances: None,
                balance_visible: false,
                cached: false,
            });
        }

        let cached = self.get_balance().await?;
        // Pull USDC from the balances map for backward compat
        let usdc_balance = cached
            .balances
            .get("USDC")
            .map(|b| b.formatted.clone());
        Ok(BalanceResponse {
            balance: usdc_balance,
            asset: Some("USDC".to_string()),
            balances: Some(cached.balances),
            balance_visible: true,
            cached: true,
        })
    }

    /// Get the wallet address via the CLI.
    pub async fn get_address(&self) -> Result<String, AppError> {
        let output = self
            .cli
            .run(AwalCommand::GetAddress)
            .await
            .map_err(|e| match e {
                CliError::Timeout => AppError::CliTimeout,
                CliError::SessionExpired => AppError::CliSessionExpired,
                CliError::NotFound(msg) => AppError::CliNotFound(msg),
                CliError::CommandFailed { stderr, .. } => AppError::CliError(stderr),
                CliError::ParseError(msg) => AppError::CliError(msg),
            })?;

        // Real CLI returns bare string, try that first, then object format for backward compat
        output
            .data
            .as_str()
            .or_else(|| output.data["address"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::CliError("Missing address in CLI response".into()))
    }
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
    use crate::db::models::AgentStatus;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // -----------------------------------------------------------------
    // CountingMockCli — wraps MockCliExecutor, counts calls
    // -----------------------------------------------------------------
    struct CountingMockCli {
        inner: MockCliExecutor,
        call_count: Arc<AtomicUsize>,
    }

    impl CountingMockCli {
        fn new() -> Self {
            Self {
                inner: MockCliExecutor::with_defaults(),
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl CliExecutable for CountingMockCli {
        async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            self.inner.run(cmd).await
        }
    }

    // -----------------------------------------------------------------
    // Helper: build a WalletService with counting mock + test DB
    // -----------------------------------------------------------------
    fn make_service(ttl_secs: u64) -> (WalletService, Arc<CountingMockCli>) {
        let db = setup_test_db();
        let cli = Arc::new(CountingMockCli::new());
        let svc = WalletService::new(cli.clone(), db, Duration::from_secs(ttl_secs));
        (svc, cli)
    }

    fn make_service_with_db(ttl_secs: u64, db: Arc<Database>) -> (WalletService, Arc<CountingMockCli>) {
        let cli = Arc::new(CountingMockCli::new());
        let svc = WalletService::new(cli.clone(), db, Duration::from_secs(ttl_secs));
        (svc, cli)
    }

    // =================================================================
    // Test 1: Cache hit within TTL — CLI is NOT called
    // =================================================================
    #[tokio::test]
    async fn test_balance_cache_hit_within_ttl() {
        let (svc, cli) = make_service(30);

        // First call — populates cache
        let bal = svc.get_balance().await.unwrap();
        assert_eq!(bal.balances["USDC"].formatted, "1247.83");
        assert_eq!(cli.count(), 1);

        // Second call within TTL — should return cached, CLI NOT called again
        let bal2 = svc.get_balance().await.unwrap();
        assert_eq!(bal2.balances["USDC"].formatted, "1247.83");
        assert_eq!(cli.count(), 1); // still 1
    }

    // =================================================================
    // Test 2: Cache miss triggers CLI call
    // =================================================================
    #[tokio::test]
    async fn test_balance_cache_miss_triggers_cli_call() {
        let (svc, cli) = make_service(30);

        assert_eq!(cli.count(), 0);

        let bal = svc.get_balance().await.unwrap();
        assert_eq!(bal.balances["USDC"].formatted, "1247.83");
        assert_eq!(cli.count(), 1);
    }

    // =================================================================
    // Test 3: TTL expiry causes re-fetch
    // =================================================================
    #[tokio::test]
    async fn test_balance_cache_ttl_expiry_refetches() {
        // Use a very short TTL so it expires immediately
        let db = setup_test_db();
        let cli = Arc::new(CountingMockCli::new());
        let cache = BalanceCache::new(Duration::from_millis(1));
        let svc = WalletService {
            cli: cli.clone(),
            db,
            cache,
        };

        // First call
        let _bal = svc.get_balance().await.unwrap();
        assert_eq!(cli.count(), 1);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Second call — TTL expired, should call CLI again
        let _bal2 = svc.get_balance().await.unwrap();
        assert_eq!(cli.count(), 2);
    }

    // =================================================================
    // Test 4: Concurrent access — CLI called only once
    // =================================================================
    #[tokio::test]
    async fn test_balance_cache_concurrent_access_single_fetch() {
        let (svc, cli) = make_service(30);
        let svc = Arc::new(svc);

        let mut handles = vec![];
        for _ in 0..10 {
            let s = svc.clone();
            handles.push(tokio::spawn(async move { s.get_balance().await }));
        }

        for h in handles {
            let result = h.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().balances["USDC"].formatted, "1247.83");
        }

        // With the double-check pattern, the CLI should be called at most once
        // (all concurrent callers will see the cache populated by the first writer).
        assert_eq!(cli.count(), 1);
    }

    // =================================================================
    // Test 5: Agent with balance_visible = false
    // =================================================================
    #[tokio::test]
    async fn test_balance_visibility_per_agent_hidden() {
        let db = setup_test_db();
        let mut agent = create_test_agent("HiddenBot", AgentStatus::Active);
        agent.balance_visible = false;
        insert_agent(&db, &agent).unwrap();

        let (svc, cli) = make_service_with_db(30, db);

        let resp = svc.get_balance_for_agent(&agent.id).await.unwrap();
        assert!(!resp.balance_visible);
        assert!(resp.balance.is_none());
        assert!(resp.asset.is_none());
        // CLI should NOT have been called at all
        assert_eq!(cli.count(), 0);
    }

    // =================================================================
    // Test 6: Agent with balance_visible = true
    // =================================================================
    #[tokio::test]
    async fn test_balance_visibility_per_agent_visible() {
        let db = setup_test_db();
        let mut agent = create_test_agent("VisibleBot", AgentStatus::Active);
        agent.balance_visible = true;
        insert_agent(&db, &agent).unwrap();

        let (svc, cli) = make_service_with_db(30, db);

        let resp = svc.get_balance_for_agent(&agent.id).await.unwrap();
        assert!(resp.balance_visible);
        assert_eq!(resp.balance.unwrap(), "1247.83"); // USDC formatted balance
        assert_eq!(resp.asset.unwrap(), "USDC");
        assert!(resp.balances.is_some());
        assert_eq!(cli.count(), 1);
    }

    // =================================================================
    // Additional: get_address
    // =================================================================
    #[tokio::test]
    async fn test_get_address() {
        let (svc, _cli) = make_service(30);
        let addr = svc.get_address().await.unwrap();
        assert_eq!(addr, "0xMockWalletAddress123");
    }

    // =================================================================
    // Additional: invalidate cache
    // =================================================================
    #[tokio::test]
    async fn test_invalidate_cache_forces_refetch() {
        let (svc, cli) = make_service(30);

        let _bal = svc.get_balance().await.unwrap();
        assert_eq!(cli.count(), 1);

        svc.cache.invalidate().await;

        let _bal2 = svc.get_balance().await.unwrap();
        assert_eq!(cli.count(), 2);
    }

    // =================================================================
    // Additional: agent not found
    // =================================================================
    #[tokio::test]
    async fn test_balance_for_nonexistent_agent_returns_not_found() {
        let (svc, _cli) = make_service(30);
        let result = svc.get_balance_for_agent("nonexistent-agent-id").await;
        assert!(result.is_err());
    }

    // =====================================================================
    // NEW TDD TESTS: Real CLI format matching
    // =====================================================================

    #[tokio::test]
    async fn test_balance_fetches_all_assets() {
        let (svc, _cli) = make_service(30);
        let bal = svc.get_balance().await.unwrap();
        assert!(bal.balances.contains_key("USDC"));
        assert!(bal.balances.contains_key("ETH"));
        assert!(bal.balances.contains_key("WETH"));
    }

    #[tokio::test]
    async fn test_get_address_bare_string() {
        let (svc, _cli) = make_service(30);
        let addr = svc.get_address().await.unwrap();
        assert!(addr.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_send_usdc_only_no_asset_in_args() {
        use crate::cli::commands::AwalCommand;
        use rust_decimal::Decimal;
        let cmd = AwalCommand::Send {
            to: "0xRecipient".into(),
            amount: Decimal::new(100, 2),
            chain: None,
        };
        let args = cmd.to_args();
        assert!(!args.contains(&"--asset".to_string()));
        assert_eq!(args[0], "send");
    }
}
