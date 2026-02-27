# TOCTOU Race Condition Deep Dive: Spending Policy Engine

## Executive Summary

There is a **Time-of-Check-to-Time-of-Use (TOCTOU) race condition** in the transaction processing pipeline. The spending policy evaluation (the "check") reads the spending ledger **outside** any database transaction, while the ledger update (the "use") happens later inside a `BEGIN EXCLUSIVE` block. Between these two points, concurrent transactions can all pass the policy check against stale ledger data, allowing overspend beyond configured caps.

---

## 1. Exact Code Flow for a Send Request

### File: `src-tauri/src/core/tx_processor.rs`

`process_send()` (line 94-253) executes this sequence:

```
Line  99: let now = Utc::now();
Line 100: let tx_id = uuid::Uuid::new_v4().to_string();
Line 102-104: Compute period keys (daily, weekly, monthly)

--- STEP 1: SPENDING POLICY CHECK (NO DB TRANSACTION) ---
Line 107-109: self.spending_engine.evaluate(agent_id, request.amount, &request.to)
             -> This calls get_spending_for_period() which does a plain SELECT
             -> No BEGIN EXCLUSIVE, no locking
             -> Returns PolicyDecision::AutoApproved / RequiresApproval / Denied

Line 111-129: If Denied -> insert_transaction (Denied status), return immediately

--- STEP 2: GLOBAL POLICY CHECK (NO DB TRANSACTION) ---
Line 132: let balance = *self.current_balance.read().await;
Line 133: self.global_engine.evaluate(request.amount, balance)
          -> This calls get_global_spending_for_period() which does a plain SELECT
          -> No BEGIN EXCLUSIVE, no locking

Line 135-153: If Denied -> insert_transaction (Denied status), return immediately

--- STEP 3: INSERT TRANSACTION + DISPATCH ---
Line 156-252: match spending_decision
  AutoApproved (line 157-207):
    Line 158-169: build_transaction(status=Executing), insert_transaction()
    Line 185-201: tokio::spawn(execute_send(...))  <-- BACKGROUND TASK
    Line 203-206: Return Accepted { status: "executing" }

  RequiresApproval (line 208-247):
    Line 209-220: build_transaction(status=AwaitingApproval), insert_transaction()
    Line 222-241: insert_approval_request()
    Line 243-246: Return Accepted { status: "awaiting_approval" }
```

### Background Task: `execute_send()` (line 256-349)

```
Line 270-276: cli.run(AwalCommand::Send {...}).await  <-- Calls external wallet CLI

Line 280-318: match cli_result
  Ok (line 281-318):
    Line 290-300: update_transaction_and_ledgers_atomic(...)
                  -> THIS is where BEGIN EXCLUSIVE happens
                  -> Updates tx status to confirmed
                  -> Upserts spending_ledger (agent, daily/weekly/monthly)
                  -> Upserts global_spending_ledger (daily/weekly/monthly)
                  -> COMMIT

  Err (line 320-331):
    Line 322-330: update_transaction_status(Failed)
                  -> Ledger is NOT updated (correct behavior)
```

---

## 2. Where the Race Window Exists

### The TOCTOU Gap

```
THREAD A (process_send for Agent A):                THREAD B (process_send for Agent B):

Line 107: evaluate() reads ledger -> 0 spent
          "0 + 15 <= 20 daily cap? YES"             Line 107: evaluate() reads ledger -> 0 spent
          Returns AutoApproved                                "0 + 15 <= 20 daily cap? YES"
                                                              Returns AutoApproved
Line 169: insert_transaction(Executing)
Line 185: tokio::spawn(execute_send)                Line 169: insert_transaction(Executing)
          |                                         Line 185: tokio::spawn(execute_send)
          v                                                   |
      [BACKGROUND]                                            v
      Line 270: cli.run().await                           [BACKGROUND]
      Line 290: BEGIN EXCLUSIVE                           Line 270: cli.run().await
                UPDATE tx -> confirmed                    Line 290: BEGIN EXCLUSIVE (BLOCKS until A commits)
                UPSERT ledger += 15                                 UPDATE tx -> confirmed
                COMMIT                                              UPSERT ledger += 15 (now 30 total!)
                                                                    COMMIT

RESULT: Ledger = 30, but daily cap was 20!
```

### Precise Race Window

The race window spans from:
- **START**: `spending_policy.rs` line 106 (`get_spending_for_period` returns)
- **END**: `queries.rs` line 966 (`BEGIN EXCLUSIVE` acquired in `update_transaction_and_ledgers_atomic`)

This window includes:
1. The rest of `spending_policy::evaluate()` (lines 106-151)
2. All of `global_policy::evaluate()` (lines 132-133 in tx_processor.rs)
3. Transaction insertion (line 169)
4. Spawning the background task (line 185)
5. The entire CLI execution (line 270-276) -- this can take **seconds**
6. Everything until `BEGIN EXCLUSIVE` is acquired (line 966 in queries.rs)

The window is **enormous** -- it spans the entire CLI wallet execution time, which is the dominant latency in the system.

---

## 3. Current Transaction Boundaries

### What IS inside BEGIN EXCLUSIVE

**`update_transaction_and_ledgers_atomic()`** (queries.rs lines 954-1027):
```rust
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
) -> Result<(), AppError>
```

Inside the EXCLUSIVE transaction:
- UPDATE transactions SET status = 'confirmed'
- UPSERT spending_ledger for daily, weekly, monthly (agent-level)
- UPSERT global_spending_ledger for daily, weekly, monthly
- COMMIT (or ROLLBACK on error)

**`upsert_spending_ledger()`** (queries.rs lines 539-574):
- Standalone BEGIN EXCLUSIVE for a single agent+period upsert
- Used in tests but NOT in the main transaction flow (the atomic function does inline upserts)

**`upsert_global_spending_ledger()`** (queries.rs lines 863-897):
- Standalone BEGIN EXCLUSIVE for a single period upsert
- Same as above -- not used in main flow

### What is NOT inside BEGIN EXCLUSIVE (the problem)

**`get_spending_for_period()`** (queries.rs lines 576-603):
```rust
pub fn get_spending_for_period(
    db: &Database,
    agent_id: &str,
    period: &str,
) -> Result<Option<SpendingLedger>, AppError>
```
- Plain SELECT with no transaction
- Called by `SpendingPolicyEngine::evaluate()` -> `get_period_total()` (spending_policy.rs line 154)
- Called 3 times per evaluate: daily, weekly, monthly

**`get_global_spending_for_period()`** (queries.rs lines 899-924):
```rust
pub fn get_global_spending_for_period(
    db: &Database,
    period: &str,
) -> Result<Option<GlobalSpendingLedger>, AppError>
```
- Plain SELECT with no transaction
- Called by `GlobalPolicyEngine::evaluate()` -> `get_period_total()` (global_policy.rs line 183)
- Called up to 3 times per evaluate: daily, weekly, monthly (if cap > 0)

**`insert_transaction()`** -- also outside any exclusive transaction

---

## 4. All Functions That Need to Change

### Architecture Change Required

The fix must move the policy check + ledger update into a single atomic operation. There are two main approaches:

#### Approach A: Check-and-Reserve (Recommended)

Create a new function that does policy check + ledger reservation inside BEGIN EXCLUSIVE:

```rust
// New function needed in queries.rs
pub fn check_and_reserve_spending_atomic(
    db: &Database,
    agent_id: &str,
    amount: &str,
    period_daily: &str,
    period_weekly: &str,
    period_monthly: &str,
) -> Result<SpendingCheckResult, AppError>
```

This would:
1. BEGIN EXCLUSIVE
2. Read spending_ledger for daily/weekly/monthly
3. Read global_spending_ledger for daily/weekly/monthly
4. Evaluate against caps
5. If within caps: upsert ledger (reserve the spend), COMMIT, return Allowed
6. If over caps: ROLLBACK, return Denied

#### Approach B: Move evaluate() inside the EXCLUSIVE block

Refactor so that policy evaluation happens inside the same BEGIN EXCLUSIVE as the ledger update.

### Functions That Must Change

| File | Function | Current Line | Change Needed |
|------|----------|-------------|---------------|
| `tx_processor.rs` | `process_send()` | 94-253 | Move policy check into atomic block, or call new check-and-reserve |
| `spending_policy.rs` | `evaluate()` | 63-151 | Either: (a) accept a connection/transaction parameter, or (b) split into "check within txn" variant |
| `spending_policy.rs` | `get_period_total()` | 153-159 | Must read within the same DB transaction |
| `global_policy.rs` | `evaluate()` | 66-157 | Same as spending_policy -- must read within same txn |
| `global_policy.rs` | `get_period_total()` | 183-191 | Must read within the same DB transaction |
| `queries.rs` | `get_spending_for_period()` | 576-603 | Needs a variant that accepts an existing connection (within txn) |
| `queries.rs` | `get_global_spending_for_period()` | 899-924 | Needs a variant that accepts an existing connection (within txn) |
| `queries.rs` | `update_transaction_and_ledgers_atomic()` | 954-1027 | Must incorporate the policy check, or be replaced by check-and-reserve |
| `queries.rs` | `get_spending_policy()` | (reads policy) | Needs variant that accepts existing connection |
| `queries.rs` | `get_global_policy()` | (reads global policy) | Needs variant that accepts existing connection |

### Exact Types Involved

```rust
// From db/models.rs
pub struct SpendingPolicy {
    pub agent_id: String,
    pub per_tx_max: String,        // Decimal as string
    pub daily_cap: String,
    pub weekly_cap: String,
    pub monthly_cap: String,
    pub auto_approve_max: String,
    pub allowlist: Vec<String>,
    pub updated_at: i64,
}

pub struct SpendingLedger {
    pub agent_id: String,
    pub period: String,
    pub total: String,             // Decimal as string
    pub tx_count: i64,
    pub updated_at: i64,
}

pub struct GlobalSpendingLedger {
    pub period: String,
    pub total: String,             // Decimal as string
    pub tx_count: i64,
    pub updated_at: i64,
}

pub struct GlobalPolicy {
    pub id: String,
    pub daily_cap: String,
    pub weekly_cap: String,
    pub monthly_cap: String,
    pub min_reserve_balance: String,
    pub kill_switch_active: bool,
    pub kill_switch_reason: String,
    pub updated_at: i64,
}

// From core/spending_policy.rs
pub enum PolicyDecision {
    AutoApproved,
    RequiresApproval { reason: String },
    Denied { reason: String },
}

// From core/global_policy.rs
pub enum GlobalPolicyDecision {
    Allowed,
    Denied { reason: String },
}

// From core/tx_processor.rs
pub enum TransactionResult {
    Accepted { tx_id: String, status: String },
    Denied { tx_id: String, reason: String },
}

// Database connection type (from db/schema.rs)
// Pool: r2d2::Pool<SqliteConnectionManager>
// Connection: r2d2::PooledConnection<SqliteConnectionManager>
// File-based pool_size = 4, in-memory pool_size = 1
```

### Suggested New Function Signature

```rust
/// Atomically check spending policy + global policy and reserve spending.
/// Returns the combined policy decision. If AutoApproved, the ledger is
/// already updated (reserved). If Denied, no ledger changes.
///
/// This eliminates the TOCTOU gap by performing the check and the write
/// inside a single BEGIN EXCLUSIVE transaction.
pub fn check_policy_and_reserve_atomic(
    db: &Database,
    agent_id: &str,
    amount: Decimal,
    recipient: &str,
    current_balance: Decimal,
    period_daily: &str,
    period_weekly: &str,
    period_monthly: &str,
    updated_at: i64,
) -> Result<AtomicPolicyResult, AppError>

pub enum AtomicPolicyResult {
    AutoApproved,                          // Ledger reserved
    RequiresApproval { reason: String },   // Ledger NOT reserved (reserve on approval)
    Denied { reason: String },             // Ledger NOT reserved
}
```

**Important design consideration for RequiresApproval**: If a transaction requires approval, the reservation should probably happen at approval time, not at check time. Otherwise, the reserved amount would need to be "released" if the approval is denied or expires. This adds complexity but is necessary for correctness.

---

## 5. Existing Concurrent Test Analysis

### File: `src-tauri/tests/concurrent_transactions.rs`

#### Test 1: `test_concurrent_sends_no_overspend` (line 141)

**What it tests**: Two agents each send 15 with a global daily cap of 30.

**Does it catch the TOCTOU bug?** **NO.** The test explicitly acknowledges the TOCTOU gap in its comments (lines 133-140):

> "Because the spending policy check (evaluate) happens BEFORE the BEGIN EXCLUSIVE ledger update, both requests may pass policy checks concurrently."

The assertion is deliberately weak:
```rust
// Line 201-210: "At least one should be accepted"
assert!(accepted_count >= 1, ...);

// Line 222-224: Accepts either 15 OR 30 in the ledger
assert!((total - 15.0).abs() < 0.01 || (total - 30.0).abs() < 0.01, ...);
```

This test **documents the bug** but **does not fail when the bug is present**. It passes whether or not the TOCTOU is exploited.

**To actually catch the bug**, the assertion should be:
```rust
// STRICT: Global cap is 30, individual caps are 20 each.
// Both 15+15=30 fits within global cap, so both SHOULD succeed.
// This test doesn't actually test overspend.
```

This test scenario doesn't even demonstrate overspend because 15+15=30 which exactly equals the global cap of 30.

#### Test 2: `test_concurrent_sends_both_succeed_within_caps` (line 244)

**What it tests**: Both sends are within all caps. Both should succeed.

**Does it catch TOCTOU?** **No.** This is a positive test -- it verifies both succeed when they should. No race condition is relevant.

#### Test 3: `test_concurrent_sends_serialization_correctness` (line 356)

**What it tests**: Single agent, daily_cap=20, sends 5x5.00 concurrently. Total attempted = 25, cap = 20.

**Does it catch the TOCTOU bug?** **NO.** The test explicitly allows overspend:

```rust
// Line 416-421:
assert!(accepted >= 4, ...);  // Allows ALL 5 to be accepted!
```

And then verifies ledger accuracy:
```rust
// Line 431-434: "total should equal accepted * 5"
// If all 5 accepted due to TOCTOU, total = 25 which exceeds cap of 20
// The test PASSES because it only checks ledger accuracy, not cap enforcement
```

**This test proves the ledger doesn't lose updates but explicitly does NOT test cap enforcement.** The comments (lines 347-355) acknowledge this:

> "The BEGIN EXCLUSIVE serialization ensures ledger ACCURACY (no lost updates), but does NOT prevent TOCTOU overspend because the policy check is outside the exclusive transaction."

### Verdict on Existing Tests

All three concurrent tests verify **ledger accuracy** (no lost updates) but **none of them actually assert that spending caps are enforced under concurrency**. They all have weak assertions that explicitly allow the TOCTOU overspend. The tests document the bug rather than catching it.

**A proper TOCTOU regression test should**:
1. Set up an agent with daily_cap=20
2. Send 5 concurrent requests of 5.00 each (total 25)
3. Assert that **at most 4** are accepted (total <= 20)
4. Assert that the 5th is **denied** with a daily cap reason
5. Assert ledger total <= 20

---

## 6. Additional Observations

### The `current_balance` RwLock is Also Vulnerable

In `process_send()` line 132:
```rust
let balance = *self.current_balance.read().await;
```

This reads the balance for the global policy min_reserve_balance check, but the balance is never atomically updated when a transaction is confirmed. Multiple concurrent transactions will all see the same balance and could collectively overdraw below the minimum reserve.

### The Ledger Update Happens AFTER CLI Execution

The ledger is updated in `execute_send()` which is a background task. This means:
1. `process_send()` returns `Accepted { status: "executing" }` immediately
2. The CLI call happens (seconds of latency)
3. Only THEN is the ledger updated

During step 2, any number of concurrent `process_send()` calls will read the stale (pre-update) ledger. The race window is as wide as the CLI execution time, making it trivially exploitable.

### SQLite File-Based Pool Size = 4

The `Database::new()` (schema.rs) uses `max_size(4)` for file-based DBs. This means up to 4 concurrent connections can exist, each performing independent reads. The `BEGIN EXCLUSIVE` only blocks other writers, but the policy reads happen on separate connections before any EXCLUSIVE lock is acquired.

### CAST to REAL Precision Issue

The ledger upsert uses `CAST(... AS REAL)` for arithmetic (queries.rs line 554, 877, 984, 1000). SQLite REAL is 64-bit IEEE 754 float, which can introduce rounding errors for decimal currency values. This is a separate issue but worth noting -- the fix should consider using integer arithmetic (store cents) or a decimal library at the SQL level.
