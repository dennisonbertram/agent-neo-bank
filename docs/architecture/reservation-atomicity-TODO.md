# Reservation Atomicity — Technical Debt

## Current State

The spending reservation and transaction/approval insertion are separate database operations:

1. `check_policy_and_reserve_atomic` — reserves spending cap budget
2. `insert_transaction` — creates the transaction record
3. `insert_approval_request` — creates the approval record (if needed)

These run as individual SQL statements, not within a single database transaction.

## Risk

If the process crashes or errors between step 1 and step 2:

- Spending caps are consumed (budget reserved) but no transaction record exists
- The reserved budget is effectively "lost" — it reduces the agent's remaining cap
  without any corresponding transaction
- Manual intervention would be needed to restore the cap

## Required Fix

Wrap reserve + transaction insert + approval insert in a single SQLite transaction:

```
BEGIN;
  check_policy_and_reserve_atomic(...)
  insert_transaction(...)
  insert_approval_request(...)  -- if RequiresApproval
COMMIT;
```

On any failure, the entire operation rolls back atomically.

## Additional Considerations

- **Rollback idempotency**: Rollback operations should be tied to `tx_id` so they
  can be safely retried without double-reverting
- **Approval denial/expiry**: When an approval is denied or expires, the associated
  reservation must be rolled back. This should also happen within a transaction:
  ```
  BEGIN;
    update_approval_status(denied/expired)
    rollback_reservation(tx_id)
  COMMIT;
  ```
- **CLI failure rollback**: Currently handled inline in `mcp_router.rs` handlers.
  Should be consolidated into a single `fail_transaction_and_rollback(tx_id)` helper
  that atomically marks the tx as Failed and reverses the reservation

## Priority

Medium — the window between reserve and insert is small (microseconds in normal
operation), but the consequence of hitting it is silent budget loss. Should be
addressed before production deployment.
