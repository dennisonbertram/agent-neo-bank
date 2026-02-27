## CRITICAL

### 1) **Dashboard shows/copies an incorrect wallet address (truncated string gets re-truncated)**
**Where:** `src/pages/Dashboard.tsx`, `src/components/shared/MonoAddress.tsx`  
**What:** Dashboard passes `"0x72AE...C504B4"` (already truncated) into `MonoAddress`, which *then truncates again* and copies that truncated value to clipboard.  
- Display becomes something like `0x72AE...04B4` (wrong).
- Clipboard copy uses the `address` prop **as-is**, so users copy `0x72AE...C504B4` (not a real address).

**Impact:** Users can fund/send to the wrong address (loss of funds / failed deposits) because the UI is presenting and copying an invalid address.

**Evidence:**
- Dashboard: `address="0x72AE...C504B4"`
- MonoAddress: `navigator.clipboard.writeText(address)` + `displayAddress = `${address.slice(0, 6)}...${address.slice(-4)}``

**Fix:** Always pass the **full canonical address** to `MonoAddress`. Only truncate for display inside the component. Never store/display/copy a pre-truncated address string.

---

### 2) **Fund → Deposit tab uses a hardcoded wallet address + “Copy” button is a no-op**
**Where:** `src/pages/Fund.tsx`  
**What:**
- Deposit address is hardcoded: `0x72AE334b...`
- “Copy” button has no `onClick`, so it does not copy anything.

**Impact:**  
- Users may deposit to the wrong address (if their actual wallet differs) because the UI is not tied to current user state.
- Users may believe they copied the address but actually didn’t, increasing risk of manual errors.

**Fix:** Fetch/render the real wallet address (from app state/hook) and implement copy with success/failure feedback + disabled state while loading.

---

### 3) **Onboarding can display a placeholder wallet address (“0x...”) and still encourages funding / copying**
**Where:** `src/pages/Onboarding.tsx`, `src/components/onboarding/FundStep.tsx`  
**What:**
- `walletAddress` defaults to `"0x..."`.
- If `auth_status` fails or returns no `address`, the Fund step still renders and the user can copy `"0x..."` and proceed.

Also: if `auth_login` returns `status: "verified"` (skip OTP), the code jumps to step 3 **without attempting to fetch wallet address at all**, so it will very likely remain `"0x..."`.

**Impact:** Users may attempt to fund an invalid address copied from the UI.

**Fix:**  
- Require a valid address before showing Fund step content / enabling copy / enabling “Continue to Dashboard”.
- On the `"verified"` fast-path in `handleEmailSubmit`, also call `auth_status` (or equivalent) to populate `walletAddress`.

---

## HIGH

### 4) **Dashboard “AgentCard” always shows “Active” status badge (can be incorrect)**
**Where:** `src/pages/Dashboard.tsx` (`AgentCard`)  
**What:** `StatusBadge status="active"` is hardcoded for every agent summary.

**Impact:** Misrepresents agent state (e.g., suspended/revoked) which can cause users to make incorrect operational decisions (thinking an agent is allowed to spend when it isn’t).

**Fix:** Either:
- fetch agent status and render accurately, or
- remove the status badge from this card if status isn’t available in `AgentBudgetSummary`.

---

### 5) **Agent spending limits editor has no validation + no error/loading UX, can mislead**
**Where:** `src/pages/AgentDetail.tsx`  
**What:**
- Inputs accept any string (empty, negative, non-numeric). No client-side validation.
- Save path has no loading state and no error display; failures will be silent/unhandled.
- `toggleEdit()` calls `handleSaveLimits()` without `await` or `try/catch` (potential unhandled promise rejection + unclear UI state on failure).

**Impact:** Users can believe limits were updated (or attempt to set nonsensical limits) without clear feedback. In fintech UI, this is a high-risk UX failure even if backend rejects.

**Fix:** Add numeric validation (>=0, required fields), disable Save during request, show success/error toast/message, and `await` save with proper error handling.

---

## MEDIUM

### 6) Transactions: client-side search conflicts with server pagination/total (“Showing X–Y of Z” becomes misleading)
**Where:** `src/pages/Transactions.tsx`  
**What:** `searchQuery` filters only the currently loaded page client-side, but the UI still shows `Showing {showStart}-{showEnd} of {total}` from server totals.

**Impact:** Users can think there are “no matching transactions” when matches exist on other pages, or misunderstand totals.

**Fix:** Either implement server-side search, or clearly label search as “current page only”, or change totals display when search is active.

---

### 7) Agents page: no error state; fetch failure looks like “No agents yet”
**Where:** `src/pages/Agents.tsx`  
**What:** `catch(() => {})` then sets loading false, causing empty-state UI indistinguishable from real “zero agents”.

**Impact:** Misleads user about data availability vs. connectivity/app error.

**Fix:** Track `error` state and show a retry UI.

---

### 8) Approvals: resolve actions have no per-item loading/disable; easy to double-submit
**Where:** `src/pages/Approvals.tsx`  
**What:** Approve/Deny stays clickable during request; failures are silent.

**Impact:** Confusing UX; potential duplicate resolves and unclear outcome.

**Fix:** Disable buttons while resolving + show inline status/error.

---

## LOW

### 9) Clipboard copy actions don’t handle failures
**Where:** `MonoAddress.tsx`, `FundStep.tsx`  
**What:** `navigator.clipboard.writeText` can throw (permissions/unavailable). No try/catch; can create false confidence.

**Fix:** try/catch + fallback + error tooltip/toast.

---

### 10) AgentDetail “Allowed Recipients” remove (X) button has no behavior
**Where:** `src/pages/AgentDetail.tsx`  
**Impact:** Dead control in a sensitive policy area; user assumes recipient removed when it isn’t.

**Fix:** Remove button until implemented or wire to real state/action.

---

CRITICAL: 3  
HIGH: 2  
APPROVED: NO
