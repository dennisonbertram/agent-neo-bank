## CRITICAL

### 1) No authentication/route-guarding; onboarding can be bypassed to reach privileged functionality
**Where**
- `src/App.tsx`: all sensitive pages are mounted under `<Shell />` with **no auth gate**:
  - `/` (Dashboard), `/agents`, `/agents/:id`, `/transactions`, `/approvals`, `/fund`, `/settings`
- `src/pages/Onboarding.tsx`: onboarding state is purely client-side (`step`), and completion is just `window.location.href = "/"`.

**Why it matters**
If the backend (Tauri commands) relies on the UI flow for access control (common in desktop apps), an attacker can:
- Open the app and directly navigate to sensitive routes (e.g., `/approvals`) without completing OTP.
- Trigger privileged actions (approve/deny, suspend agents, update spending policy) without an authenticated session check happening in the UI.

**Exploit sketch**
- Launch app → manually browse to `/approvals` → click **Approve/Deny** (calls `invoke("resolve_approval", ...)`)
- Or go to `/agents/:id` and click **Suspend Agent** (calls `invoke("suspend_agent", ...)`)
- Or edit limits and click **Save** (calls `invoke("update_agent_spending_policy", ...)`)

**Fix**
- Add an auth boundary at the router level (e.g., `<ProtectedRoute>` wrapping `<Shell />`) that blocks access until `invoke("auth_status")` indicates authenticated.
- Also enforce auth on the Rust side for *every* command (do not trust frontend gating).

---

## HIGH

### 2) IDOR / authorization risk via URL parameter `:id` used directly in backend invocations
**Where**
- `src/pages/AgentDetail.tsx`:
  - `const { id } = useParams<{ id: string }>();`
  - Used directly in:
    - `invoke("get_agent", { agentId: id })`
    - `invoke("get_agent_spending_policy", { agentId: id })`
    - `invoke("get_agent_transactions", { agentId: id, limit: 20 })`
  - Also used for action: `invoke("suspend_agent", { agentId: id })`

**Why it matters**
If agent IDs are guessable/obtainable (or leaked from `list_agents`), a user can change the URL to access or act on another agent’s data (read policy/transactions, suspend agent).

**Exploit sketch**
- Navigate to `/agents/<victimAgentId>` and the UI will request all the victim agent’s data.
- Click **Suspend Agent** to suspend the victim agent if backend doesn’t re-check ownership.

**Fix**
- Backend must enforce per-user ownership checks for every `agentId`.
- Consider using opaque IDs and/or scoping agent queries to the authenticated user so the client never supplies raw `agentId` for authorization decisions.

---

### 3) Approval resolution is callable without any UI-level authorization checks
**Where**
- `src/pages/Approvals.tsx`:
  - `handleResolve` calls `invoke("resolve_approval", { approval_id: approvalId, decision })`
  - No auth/role checks in UI; no confirmation, no secondary verification step.

**Why it matters**
Approvals are typically “high-impact” (approving transactions/limit increases). If backend authorization is weak, this is a direct path to unauthorized fund movement/limit escalation.

**Fix**
- Enforce authorization in backend (who can resolve which approval).
- Add UI friction: confirmation modal + require fresh auth (re-enter OTP/passkey) for high-risk approvals.

---

## MEDIUM

### 4) Parameter tampering / lack of input validation for sensitive updates (spending policy)
**Where**
- `src/pages/AgentDetail.tsx`:
  - Editable inputs directly write strings into `editPolicy`
  - Saved with: `invoke("update_agent_spending_policy", { policy: editPolicy })`
  - No numeric validation, no min/max bounds, no type enforcement.

**Why it matters**
An attacker can enter negative values, extremely large values, NaN-ish strings, etc. If backend parsing/validation is insufficient, this can cause:
- Policy bypass (set caps to huge values)
- Backend crashes / logic errors

**Fix**
- Validate client-side (number parsing, bounds) *and* backend-side (authoritative validation).
- Consider using structured numeric types in the API rather than free-form strings.

---

### 5) `invoke()` surface is broadly reachable; filters/pagination can be abused for data scraping/DoS if backend is permissive
**Where**
- `src/pages/Transactions.tsx`: `invoke("list_transactions", { limit, offset, status, agent_id })`
  - UI uses constants, but an attacker can call `invoke` directly (devtools / compromised renderer).

**Why it matters**
If backend doesn’t enforce maximum `limit`, non-negative `offset`, and per-user scoping, this enables bulk export/scraping or performance attacks.

**Fix**
- Backend: enforce strict bounds and scoping; rate limit expensive queries.
- Consider returning only transactions for the authenticated user without accepting arbitrary `agent_id`.

---

### 6) Clipboard/phishing risk: copying attacker-controlled addresses verbatim
**Where**
- `src/components/shared/MonoAddress.tsx` and `src/components/onboarding/FundStep.tsx`
  - `navigator.clipboard.writeText(address)` with no normalization/sanitization.

**Why it matters**
If a malicious/compromised backend (or malicious agent-controlled metadata) can influence displayed addresses (e.g., allowlist entries), it can push:
- Lookalike addresses
- Addresses with hidden characters/whitespace (less visible but copied)

**Fix**
- Normalize copied values (trim, remove zero-width chars).
- Display checksum / chain + warning on non-checksummed addresses.
- Consider a “copy & verify” UX for high-value transfers.

---

## LOW

### 7) Hardcoded identity-like strings in UI (possible accidental PII/environment leakage)
**Where**
- `src/components/layout/Sidebar.tsx`: username `"dennison"` is hardcoded.
- `src/pages/Dashboard.tsx` / `src/pages/Fund.tsx`: hardcoded wallet-like addresses shown in UI.

**Why it matters**
Not a direct exploit, but can leak internal/test identities or confuse users; if real data accidentally ships, it’s a privacy issue.

**Fix**
- Load from authenticated profile/wallet state; ensure no real addresses/usernames are committed.

---

## XSS / Open Redirect summary
- **No obvious XSS sinks** found in this subset (no `dangerouslySetInnerHTML`; React text rendering is used).
- **No open redirects** observed (all `<Link to="...">` are static internal paths).

---

CRITICAL: 1  
HIGH: 2  
APPROVED: NO
