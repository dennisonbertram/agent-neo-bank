## Findings (frontend-specific)

### HIGH — Hardcoded wallet address used for “deposit/fund” flows (risk of misdirected funds)
**Where**
- `src/pages/Fund.tsx`: `const walletAddress = "0x72AE..."` used for copy + display.
- `src/pages/Dashboard.tsx`: if `invoke("get_address")` fails, falls back to the same hardcoded address.

**Why it matters**
- If `get_address` fails (backend command unavailable, transient error, first-run race, etc.), the UI will still present a valid-looking address and enable copying/funding. Users can deposit to the wrong address, causing irreversible loss/misdirection of funds.
- Also embeds an address in the shipped client bundle (scrapable), which may be sensitive operationally even if not a “secret”.

**Recommended fix**
- Remove the hardcoded address fallback for any deposit-critical UI.
- Block “Deposit”/“Copy” actions until a real address is fetched (show an explicit error state + retry).
- If you *must* have a fallback, use a clearly invalid sentinel and keep copy/continue disabled (similar to `FundStep`’s `addressReady` gating).

---

### MEDIUM — Clipboard writes are not guarded (can throw; UI may mislead user)
**Where**
- `src/components/shared/MonoAddress.tsx`: `await navigator.clipboard.writeText(address);` (no try/catch)
- `src/pages/Fund.tsx`: `await navigator.clipboard.writeText(walletAddress);` (no try/catch)
- `src/components/onboarding/FundStep.tsx`: same pattern (no try/catch)

**Why it matters**
- `navigator.clipboard` may be unavailable or permission-restricted in some environments (including desktop webviews depending on Tauri/webview settings).
- Unhandled promise rejection can cause user-visible errors and/or the UI showing “Copied!” even if the write failed (or never reaches the state update).

**Recommended fix**
- Feature-detect and wrap in `try/catch`; only set “copied” state on success.
- Provide a fallback (e.g., Tauri clipboard API if available) and/or show a non-intrusive error message (“Copy failed”).

---

### MEDIUM — Floating-point formatting for balances/amounts can misrepresent financial values
**Where**
- `src/components/shared/CurrencyDisplay.tsx`: `parseFloat(amount)` + `toLocaleString(...)`
- `src/pages/Dashboard.tsx`: `formatBalance()` uses `parseFloat(...)`

**Why it matters**
- `parseFloat` + JS `number` introduces rounding/precision issues (especially for crypto-like decimals, large values, or values with >2 decimals).
- In a fintech UI, incorrect display can cause users to approve/deny actions based on wrong amounts (frontend display correctness issue).

**Recommended fix**
- Keep values as decimal strings and format using a decimal/bignumber library (or an existing shared formatter) with explicit precision rules per asset (e.g., USDC 2 or 6 depending on representation).
- Avoid converting to `number` for display unless you can guarantee safe ranges/precision.

---

### LOW — Spending policy numeric validation allows non-finite values (Infinity / exponential)
**Where**
- `src/pages/AgentDetail.tsx`: `validateField` uses `parseFloat(value)` and checks `isNaN` / `< 0`, but does **not** reject `Infinity` or very large exponent inputs like `1e309`.

**Why it matters**
- Can lead to confusing/incorrect UI states (limits display/progress bars with `Infinity`, ratios, etc.).
- Backend will validate, but frontend can still display misleading values or behave oddly before rejection.

**Recommended fix**
- Add `Number.isFinite(num)` check and enforce sane maximums (and optionally decimal places).
- Consider normalizing input strings (reject exponent notation if undesired).

---

### LOW — Pagination offset can drift beyond total / multiple rapid clicks
**Where**
- `src/pages/Transactions.tsx`: `handleNext` always `prev + PAGE_SIZE`; button is disabled based on current `offset + PAGE_SIZE >= total` but not disabled while loading, and `offset` isn’t clamped when `total` changes.

**Why it matters**
- Users can spam “Next” during loading and end up requesting empty/out-of-range pages, showing confusing “No matching transactions” states.

**Recommended fix**
- Disable pagination controls while `isLoading`.
- Clamp next offset: `Math.min(prev + PAGE_SIZE, Math.max(0, total - (total % PAGE_SIZE || PAGE_SIZE)))`, and/or after fetch if `offset >= total` then reset to last valid page.

---

## XSS check
- No `dangerouslySetInnerHTML` observed.
- Payload/description fields are rendered as React text nodes (escaped), so direct XSS risk is low in the provided files.

CRITICAL: 0 HIGH: 1 APPROVED: NO
