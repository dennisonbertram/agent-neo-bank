## CRITICAL

### 1) AgentDetail: stale async responses can desync displayed agent vs. route `id`, enabling wrong-agent actions
**Where:** `src/pages/AgentDetail.tsx` (`loadData` via `Promise.all(...)`, called from `useEffect`)

**Issue:** `loadData()` fetches agent/policy/tx/budget for the current route param `id`. If the user navigates quickly between `/agents/:id` routes (same component instance), previous in-flight requests can resolve after the newer request and overwrite state (`setAgent`, `setPolicy`, `setEditPolicy`, `setTransactions`, `setBudget`) with **data for the prior agent**.

**Why this is dangerous (frontend-specific):**
- The UI can show **Agent A** (stale state) while the route param `id` is **Agent B**.
- Action handlers use the **current param `id`** (e.g., `handleSuspend` invokes `suspend_agent` with `{ agentId: id }`), so the user may click “Suspend Agent” while seeing Agent A, but actually suspend Agent B. This is a serious client-side state integrity bug.

**Fix:**
- Guard state updates so only the latest request can commit:
  - Use a monotonically increasing request token in a `useRef`, compare before setting state; or
  - Use an `isStale` flag in the effect cleanup.
- Additionally, consider resetting local state immediately on `id` change (e.g., `setAgent(null); setIsLoading(true)`), and/or disable action buttons if `agent.id !== id`.

---

## MEDIUM

### 2) Transactions / Approvals: out-of-order fetches can show wrong list for current filters (UI integrity)
**Where:**
- `src/pages/Transactions.tsx` (`fetchTransactions` depends on `offset/statusFilter/agentFilter`)
- `src/pages/Approvals.tsx` (`loadApprovals` depends on `filter`)

**Issue:** Rapid filter/pagination toggles can produce multiple concurrent `invoke` calls. Without a “latest request wins” guard, an older response may arrive last and overwrite state, showing results not matching the currently selected filter/page.

**Impact:** Incorrect financial/approval context displayed (misleading UI), especially around status filters/pagination.

**Fix:** Same pattern as above—request token / stale flag check before committing `setTransactions/setTotal` or `setApprovals`.

---

### 3) CurrencyDisplay: potentially misleading decimals for ETH/WETH
**Where:** `src/components/shared/CurrencyDisplay.tsx`

**Issue:** `assetDecimals` sets `ETH`/`WETH` to `6`. If amounts are intended to be displayed with more precision (or if the input strings represent full precision values), the UI may truncate/round in a way that misrepresents value.

**Impact:** Wrong/misleading financial display.

**Fix:** Align display precision with product requirements:
- If values are already human-formatted strings, avoid re-parsing/formatting.
- If values are numeric strings, use correct display precision per asset (commonly ETH has more than 6 decimals for meaningful display), or clearly label rounding.

---

## LOW

### 4) Unhandled clipboard rejection in FundStep can cause noisy/unhandled promise rejection
**Where:** `src/components/onboarding/FundStep.tsx` (`handleCopy`)

**Issue:** `await navigator.clipboard.writeText(address)` is not wrapped in `try/catch`. In restricted contexts, this can throw and create an unhandled rejection.

**Fix:** Wrap clipboard call in `try/catch` (like `MonoAddress` already does).

---

### 5) `setTimeout` state updates after unmount (minor)
**Where:** `MonoAddress`, `Fund`, `FundStep` (copy-to-clipboard “Copied!” timers)

**Issue:** `setTimeout(() => setCopied(false), 2000)` isn’t cleared on unmount; can cause React warnings and minor state weirdness.

**Fix:** Store timeout id in a ref and clear it in `useEffect` cleanup.

---

CRITICAL: 1 HIGH: 0 APPROVED: NO
