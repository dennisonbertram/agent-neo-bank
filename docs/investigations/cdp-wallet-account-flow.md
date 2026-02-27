# CDP Agentic Wallet - Account & Auth Flow Analysis

> Research Date: 2026-02-27
> Sources: CDP docs (welcome, quickstart, skills/authenticate, skills/fund), existing local research docs

## Key Findings

### 1. Can someone get an Agent Wallet WITHOUT a Coinbase account?

**Yes.** The Agentic Wallet does not require a pre-existing Coinbase account. The quickstart documentation states that running `npx awal auth login user@example.com` will "create an agentic wallet mapped to the given email." The only prerequisite listed is an email address and Node.js v24+. No Coinbase account, no CDP API keys, no KYC -- just an email.

The wallet is provisioned automatically on first authentication. The email OTP flow serves as both identity verification and wallet creation trigger.

### 2. If someone already HAS a Coinbase account, how does wallet creation work?

**The documentation does not address this case explicitly.** The wallet is "mapped to the given email" -- so if a user authenticates with the same email they use for Coinbase, the wallet is presumably associated with that identity in Coinbase's infrastructure. However, the docs never describe explicit linking, merging, or any different behavior for existing Coinbase users vs. new users.

The wallet is a separate entity from a Coinbase exchange account. It lives on Base, holds its own USDC balance, and has its own address. It is not the same as the user's Coinbase custody wallet.

### 3. What is the exact auth flow?

```
1. Agent runs: npx awal auth login user@example.com
   - System sends 6-digit OTP to the email
   - Returns a flowId (unique to this auth attempt)
   - If this is the first login for this email, a wallet is created

2. User/agent retrieves OTP from email

3. Agent runs: npx awal auth verify <flowId> <otp>
   - Validates the OTP
   - Establishes authenticated session
   - Session persists for subsequent commands

4. Agent runs: npx awal status
   - Confirms auth state, shows wallet address
```

**What happens behind the scenes (inferred, not documented):**
- Coinbase infrastructure generates a key pair for the wallet
- Private key is stored in Coinbase's infrastructure (user/agent never sees it)
- Wallet address is derived and mapped to the email
- All subsequent operations are signed by Coinbase's infrastructure on behalf of the authenticated session

**What the docs do NOT clarify:**
- Whether a Coinbase account (the exchange product) is created behind the scenes
- Whether this uses Coinbase's existing identity system or a parallel one
- Session duration / expiration behavior
- Whether re-authenticating with the same email returns the same wallet (very likely yes, given the "mapped to email" language)

### 4. Is there a way to link the Agent Wallet to an existing Coinbase account?

**Not explicitly documented as a feature.** However, the funding flow provides indirect linkage:

- When funding via Coinbase Onramp, one payment method is "Coinbase" -- described as "Transfer from an existing Coinbase account"
- This means a user can move funds FROM their Coinbase account TO the Agent Wallet through the Onramp UI
- This is a payment method, not an account link -- it works the same way Coinbase Pay works on any third-party site

There is no documented API or CLI command for linking accounts, viewing linked accounts, or synchronizing between a Coinbase exchange account and the Agent Wallet.

### 5. How do funds flow between a user's Coinbase account and their Agent Wallet?

**Funding the Agent Wallet (inbound):**

| Method | Requires Coinbase Account | Speed | How |
|--------|--------------------------|-------|-----|
| Coinbase transfer | Yes | Instant (assumed) | Via Onramp UI, select "Coinbase" payment method |
| Apple Pay | No | Instant | Via Onramp UI |
| Debit card | No | Instant | Via Onramp UI |
| ACH bank transfer | No | 1-3 business days | Via Onramp UI |
| Direct USDC transfer | No | ~seconds on Base | Send USDC to wallet's Base address from any source |

**To access the Onramp UI:** Run `npx awal show` to open the companion window, then click "Fund."

**Moving funds OUT of the Agent Wallet:**
- `npx awal send <amount> <recipient>` sends USDC to any Base address or ENS name
- To move funds back to a Coinbase account, the user would need to send USDC to their Coinbase Base deposit address
- There is no built-in "withdraw to Coinbase" command

**No bidirectional sync exists.** The Agent Wallet and Coinbase exchange account are separate balance pools. Moving funds between them requires explicit transfers in each direction.

## Implications for Onboarding Design

1. **Low friction entry**: Any email can create a wallet instantly. No Coinbase account needed. This is good for onboarding users who are not already in the Coinbase ecosystem.

2. **Funding is the friction point**: While wallet creation is instant, getting USDC into the wallet requires either a payment method (card, Apple Pay, bank) or an existing crypto holding. Users without crypto or payment methods ready will hit a wall here.

3. **No account unification**: We cannot show users a unified view of their Coinbase balance + Agent Wallet balance. These are separate. If a user has $500 in Coinbase and $0 in the Agent Wallet, they need to explicitly fund the wallet.

4. **The "Coinbase" funding option is the bridge**: For existing Coinbase users, the smoothest path is: authenticate with Agent Wallet -> open Onramp -> select "Coinbase" payment -> transfer desired amount. This is still manual, not automatic.

5. **Direct USDC transfer is the power-user path**: Users who already have USDC on Base can send directly to their wallet address. This bypasses Onramp entirely.

6. **Re-authentication likely returns the same wallet**: The "mapped to email" language strongly suggests wallet persistence. We should verify this experimentally but can design assuming email = stable wallet identity.

## Open Questions (Need Experimental Verification)

- Does authenticating with a Coinbase-registered email behave any differently?
- What is the session duration before re-authentication is required?
- Can a single email have multiple Agent Wallets, or is it strictly 1:1?
- What happens if the user changes their email on their Coinbase account?
- Is there any way to programmatically trigger the Onramp flow (vs. requiring the companion window)?
- What are the Onramp fee structures for each payment method?
