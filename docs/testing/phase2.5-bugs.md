# Phase 2.5 Bugs and Known Issues

## TOCTOU Gap in Spending Policy Evaluation

**Severity**: Medium

**Status**: Documented for Phase 3 fix. Tests written to demonstrate the gap.

### Bug Description

Spending policy evaluation reads the spending ledger OUTSIDE the `BEGIN EXCLUSIVE` transaction. When concurrent requests arrive for the same agent, all requests can pass policy checks before any ledger writes happen, potentially exceeding spending caps.

### Impact

Under concurrent load, global and individual spending caps can be exceeded. Multiple simultaneous transactions can each independently pass the policy check against stale ledger data, then all commit their writes.

### Root Cause

The policy check and ledger update are not atomic. The sequence is:

1. Read current spending totals (outside transaction)
2. Evaluate policy against totals (outside transaction)
3. `BEGIN EXCLUSIVE` transaction
4. Write new ledger entry
5. `COMMIT`

Between steps 1-2 and step 3, another request can read the same stale totals and also pass the policy check.

### Fix Needed

Move policy evaluation inside the `BEGIN EXCLUSIVE` transaction, or use a mutex/semaphore to serialize policy evaluation + ledger update as a single atomic operation.

### Likelihood

Requires concurrent requests targeting the same agent, which is unlikely in normal single-user operation but possible under automated or batch workloads.
