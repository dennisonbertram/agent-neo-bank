## XSS
- No `dangerouslySetInnerHTML` usage found in the provided frontend files.
- All potentially untrusted fields (agent names, purposes, approval payload fields, descriptions) are rendered as normal React text nodes, so React escaping prevents DOM XSS in these components.

## HIGH

### 1) Incorrect currency/amount rendering (rounding + asset-decimals ignored)
**Where**
- `src/components/shared/CurrencyDisplay.tsx`

**What / impact**
- `CurrencyDisplay` parses with `parseFloat()` and forces **exactly 2 decimal places**:
  - Any amount requiring more precision (e.g., ETH, WETH, or USDC sub-cent values) will be rounded/truncated in display (e.g., `"0.000001"` becomes `$0.00`).
- The `asset?: string` prop exists but is unused, suggesting intended asset-aware formatting is missing.
- In a fintech UI, this can cause **materially incorrect displayed financial data** (balances, caps, spent), even if backend values are correct.

**Recommendation**
- Make formatting asset-aware and avoid `number` for money:
  - Use backend-provided formatted strings where possible, or
  - Use decimal/bignumber formatting (e.g., `decimal.js`, `big.js`) and per-asset decimals.
- At minimum: honor `asset` and configure fraction digits accordingly.

## MEDIUM

### 2) Unguarded clipboard write can throw/unhandled rejection (stability + UX)
**Where**
- `src/components/onboarding/FundStep.tsx`

**What / impact**
- `handleCopy()` calls `await navigator.clipboard.writeText(address);` **without try/catch**.
- In Tauri or restricted contexts, clipboard calls can fail and throw, causing unhandled promise rejections and potentially breaking the onboarding flow UI.

**Recommendation**
- Wrap clipboard access in try/catch (like `MonoAddress` does), and provide a non-crashing fallback/notification.

---

### 3) Transaction search filters only the currently loaded page (can mislead)
**Where**
- `src/pages/Transactions.tsx`

**What / impact**
- `searchQuery` is applied **client-side only** to `transactions` already fetched for the current `offset` page.
- Users can see “No matching transactions” even though matches exist on other pages (or in the full dataset), which is an **incorrect/misleading financial activity display** pattern.

**Recommendation**
- Either:
  - Implement server-side search (preferred), or
  - Make UI explicit: “No matches on this page”, or
  - When searching, fetch from offset 0 and/or fetch more/all results for local filtering (with clear limits).

---

### 4) Amount display inconsistency (missing formatting/asset context)
**Where**
- `src/pages/AgentDetail.tsx` (Activity feed + spending limits)
  - Activity shows: `<span> ${tx.amount}</span>` (raw string with `$`), no locale formatting, no asset
  - Limits show: `${row.spent} / ${row.limit}` with raw numbers

**What / impact**
- Inconsistent formatting vs other screens can cause users to misread amounts (thousands separators, decimals, asset units), especially if assets expand beyond USDC.

**Recommendation**
- Use a single shared formatter for all monetary values (preferably asset-aware) and display the asset/unit consistently.

## LOW

### 5) Hardcoded identity-like UI value in sidebar
**Where**
- `src/components/layout/Sidebar.tsx` (`dennison`, `Connected`)

**What / impact**
- Hardcoded username-like string shipped in the client bundle can be mistaken for the actual logged-in identity (and is minor sensitive-data hygiene risk if it’s a real person).

**Recommendation**
- Replace with dynamic user info (or neutral placeholder) sourced from backend state.

---

CRITICAL: 0 HIGH: 1 APPROVED: NO
