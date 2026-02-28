# Coinbase OAuth / "Sign in with Coinbase" Research

**Date:** 2026-02-27
**Context:** Evaluating Coinbase OAuth for Tally Agentic Wallet (Tauri desktop app wrapping CDP Agentic Wallet CLI)

---

## Executive Summary

"Sign in with Coinbase" exists as a full OAuth 2.0 provider with PKCE support, making it technically viable for a Tauri desktop app. However, there is a critical blocker: **OAuth client creation is currently limited to approved partners only**. Additionally, the Agentic Wallet system and Coinbase OAuth are **completely separate systems** with no documented bridge between them. The most practical near-term benefit would be using Coinbase OAuth to capture the user's verified email and pre-fill it into the Agentic Wallet OTP flow.

---

## 1. Does "Sign in with Coinbase" Exist as an OAuth Provider?

**Yes, fully.** Coinbase offers a complete OAuth 2.0 implementation analogous to "Sign in with Google."

### Key Details

- **Product page:** https://www.coinbase.com/developer-platform/products/sign-in-with-coinbase
- **Marketing claim:** "Connect to Coinbase's 100M+ users without sharing their credentials"
- **Protocol:** Standard OAuth 2.0 Authorization Code flow
- **Endpoints:**
  - Authorization: `GET https://login.coinbase.com/oauth2/auth`
  - Token exchange: `POST https://login.coinbase.com/oauth2/token`
  - Token revocation: `POST https://login.coinbase.com/oauth2/revoke`

### Authorization Request Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `client_id` | Yes | Application ID from registration |
| `response_type` | Yes | Must be `code` |
| `redirect_uri` | No | Where user returns after auth; must be URL-encoded |
| `scope` | No | Comma-separated list of permissions |
| `state` | Recommended | Random string >= 8 chars for CSRF protection |
| `code_challenge` | No | PKCE challenge value |
| `code_challenge_method` | No | `S256` (recommended) or `plain` |
| `layout` | No | Set to `signup` to show registration instead of login |
| `referral` | No | Developer referral ID for bonuses |

### Token Exchange Response

```json
{
  "access_token": "...",
  "refresh_token": "...",
  "expires_in": 3600,
  "scope": "wallet:user:read,wallet:accounts:read",
  "token_type": "bearer"
}
```

### Three Primary Use Cases (Per Coinbase Docs)

1. **Payouts** -- Direct payment transfers to user Coinbase accounts
2. **Pay with Coinbase** -- User payment processing via Coinbase balance
3. **Trading** -- Cryptocurrency trading capabilities within applications

**Sources:**
- https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/oauth2
- https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/reference
- https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/integrations

---

## 2. Available Scopes/Permissions

Coinbase OAuth offers granular scopes following the pattern `service-name:resource:action`.

### Complete Scope List

#### User & Account

| Scope | Description |
|-------|-------------|
| `wallet:user:read` | List detailed user information |
| `wallet:user:update` | Update current user |
| `wallet:user:email` | Read current user's email address |
| `wallet:accounts:read` | List user's accounts and their balances |
| `wallet:accounts:update` | Update account (e.g. change name) |
| `wallet:accounts:create` | Create a new account (e.g. BTC wallet) |
| `wallet:accounts:delete` | Delete existing account |

#### Addresses

| Scope | Description |
|-------|-------------|
| `wallet:addresses:read` | List account's bitcoin or ethereum addresses |
| `wallet:addresses:create` | Create new bitcoin or ethereum addresses |

#### Transactions

| Scope | Description |
|-------|-------------|
| `wallet:transactions:read` | List account's transactions |
| `wallet:transactions:send` | Send bitcoin or ethereum |
| `wallet:transactions:request` | Request crypto from a Coinbase user |
| `wallet:transactions:transfer` | Transfer funds between user's accounts |

#### Buy/Sell/Trade

| Scope | Description |
|-------|-------------|
| `wallet:buys:read` | List account's buys |
| `wallet:buys:create` | Buy bitcoin or ethereum |
| `wallet:sells:read` | List account's sells |
| `wallet:sells:create` | Sell bitcoin or ethereum |
| `wallet:trades:read` | List trades |
| `wallet:trades:create` | Create trades |

#### Deposits/Withdrawals

| Scope | Description |
|-------|-------------|
| `wallet:deposits:read` | List account's deposits |
| `wallet:deposits:create` | Create a new deposit |
| `wallet:withdrawals:read` | List account's withdrawals |
| `wallet:withdrawals:create` | Create a new withdrawal |

#### Payment Methods

| Scope | Description |
|-------|-------------|
| `wallet:payment-methods:read` | List user's payment methods (e.g. bank accounts) |
| `wallet:payment-methods:delete` | Remove existing payment methods |
| `wallet:payment-methods:limits` | Get detailed payment method limits (requires `wallet:payment-methods:read`) |

#### Other

| Scope | Description |
|-------|-------------|
| `wallet:notifications:read` | List user's notifications |
| `offline_access` | Return a refresh token in the response |

### Key Answers

- **Can we read their Coinbase balance?** Yes, via `wallet:accounts:read`.
- **Can we initiate transfers?** Yes, via `wallet:transactions:send` (sends crypto) or `wallet:transactions:transfer` (between user's own accounts).
- **Can we read their email?** Yes, via `wallet:user:email`.
- **Can we read their identity info?** Partially, via `wallet:user:read` (detailed user info).

### Important Scope Restrictions

- Scopes must be declared when registering the OAuth application.
- **Scopes are difficult to change after launch.** Adding scopes later requires all users to re-authorize.
- Some permissions like sending funds require additional security settings.
- Users can selectively grant or deny individual scopes during authorization.

**Source:** https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/scopes

---

## 3. Desktop App (Tauri) Integration -- PKCE Flow

### PKCE Support

Coinbase explicitly supports PKCE (Proof Key for Code Exchange):
- `code_challenge` parameter on the authorization request
- `code_challenge_method` supports `S256` (recommended) or `plain`
- `code_verifier` required on the token exchange when PKCE is used

Coinbase docs state: "We strongly recommend implementing PKCE in your OAuth2 flow, especially for mobile and single-page applications."

### Redirect URI Options for Tauri

Three approaches for handling OAuth redirects in a Tauri desktop app:

#### Option A: Localhost Redirect (Recommended)

Use the `tauri-plugin-oauth` Rust plugin, which spawns a temporary localhost HTTP server to capture the OAuth redirect.

- Register redirect URI as `http://127.0.0.1:{port}/callback`
- Plugin handles port selection and server lifecycle
- Most compatible approach
- **Plugin:** https://github.com/FabianLars/tauri-plugin-oauth

#### Option B: Custom URI Scheme

Register a custom protocol like `agentbank://coinbase-oauth`.

- Coinbase docs mention registering custom schemes as permitted redirect URIs
- Tauri supports custom URI scheme registration
- May have cross-platform edge cases (some OS/browser combos reject non-HTTP schemes)

#### Option C: Out-of-Band (OOB)

Use `urn:ietf:wg:oauth:2.0:oob` redirect URI.

- User manually copies an auth code back into the app
- Poor UX but most universally compatible
- Coinbase mobile SDKs historically supported this

### Recommended Approach for Tauri

```
1. User clicks "Sign in with Coinbase" in the Tauri app
2. App generates PKCE code_verifier + code_challenge
3. App starts local HTTP server on a random port (via tauri-plugin-oauth)
4. App opens system browser to:
   https://login.coinbase.com/oauth2/auth?
     response_type=code&
     client_id=YOUR_CLIENT_ID&
     redirect_uri=http://127.0.0.1:{port}/callback&
     scope=wallet:user:read,wallet:user:email,wallet:accounts:read,offline_access&
     state=RANDOM_STRING&
     code_challenge=CHALLENGE&
     code_challenge_method=S256
5. User authenticates on Coinbase in their browser
6. Coinbase redirects to localhost, captured by tauri-plugin-oauth
7. App exchanges code + code_verifier for tokens at the token endpoint
8. App has access_token + refresh_token
```

**Sources:**
- https://github.com/FabianLars/tauri-plugin-oauth
- https://github.com/tauri-apps/tauri/discussions/8554

---

## 4. Connection Between Coinbase OAuth and Agentic Wallet

### Short Answer: They Are Completely Separate Systems

There is **no documented connection** between Coinbase OAuth (the Coinbase App API) and the Agentic Wallet system.

### Agentic Wallet Architecture

- **Authentication:** Email OTP only (no OAuth option)
- **CLI flow:** `npx awal auth login <email>` --> receives OTP --> `npx awal auth verify <flowId> <otp>`
- **Network:** Base only
- **Wallet type:** Non-custodial, keys held in Coinbase TEE infrastructure
- **Identity:** Email address is the sole identifier
- **No link to Coinbase retail accounts** -- the docs do not indicate any connection to the user's main Coinbase account, balance, or identity

### Coinbase OAuth (Coinbase App API)

- **Authentication:** Full OAuth 2.0
- **Access:** User's Coinbase retail account (balances, transactions, payment methods)
- **Identity:** Full Coinbase user profile (name, email, potentially KYC-verified identity)
- **Completely different API surface** from the Agentic Wallet

### Why They Are Separate

- Agentic Wallet uses CDP infrastructure but is designed for **AI agents**, not end users
- Coinbase OAuth accesses the **retail Coinbase account** via the Coinbase App API
- There is no API endpoint to link an OAuth session to an Agentic Wallet
- There is no way to fund an Agentic Wallet from a Coinbase account via OAuth (you'd need to send USDC to the wallet address manually)

---

## 5. Using Coinbase OAuth to Auto-Fill Email for Agentic Wallet OTP

### This Is Feasible and Likely the Best Near-Term Integration

**Flow:**

```
1. User signs in with Coinbase OAuth (scope: wallet:user:email)
2. App retrieves user's verified email from Coinbase
3. App auto-fills that email into the Agentic Wallet auth flow
4. App calls `npx awal auth login <email>` with the Coinbase-verified email
5. User receives OTP at that email and completes verification
```

### Benefits

- User doesn't need to type their email
- Email is verified by Coinbase (higher trust)
- Establishes a Coinbase identity link in the app (even if systems are separate)
- Could later expand to read Coinbase balances, facilitate funding

### Limitations

- Still requires the OTP step (no way to skip it)
- The Coinbase email may differ from the email they want for the Agentic Wallet
- Adds complexity for a small UX improvement

---

## 6. Requirements to Register as a Coinbase OAuth App

### CRITICAL BLOCKER: Approved Partners Only

The Coinbase documentation states:

> "OAuth client creation is currently limited to approved partners."

Developers must contact Coinbase through their developer interest form to request access.

### Registration Process (Once Approved)

1. Open a Coinbase Account
2. Go to Settings --> API Access --> "+ New OAuth2 Application"
3. **Or** use the CDP Portal: https://portal.cdp.coinbase.com/projects/api-keys/oauth
4. Provide:
   - Application name and description
   - Redirect URIs (can include localhost for development)
   - Requested scopes (must be declared upfront, hard to change later)
   - Optional: Notification/webhook URL
5. Receive `client_id` and `client_secret`

### Compliance Requirements

- Must adhere to Coinbase Developer Agreement
- Cannot imply Coinbase partnership/endorsement without written approval
- Must adequately secure OAuth tokens
- Must keep registration information accurate and current

### Scope Planning Warning

Scopes must be declared at registration time. Adding scopes after launch requires all existing users to re-authorize. Plan carefully.

**Sources:**
- https://docs.cdp.coinbase.com/coinbase-app/oauth2-integration/overview
- https://help.coinbase.com/en/cloud/api/oauth2/create-app
- https://developers.coinbase.com/docs/wallet/terms/2

---

## 7. Recommendations for Tally Agentic Wallet

### Immediate Actions

1. **Apply for OAuth partner access** -- Contact Coinbase via their developer interest form. This is the gating blocker for everything else. Do this first since approval timelines are unknown.

2. **Plan scopes carefully** -- For our use case, request at minimum:
   - `wallet:user:read` -- User profile
   - `wallet:user:email` -- Email (for auto-fill into Agentic Wallet OTP)
   - `wallet:accounts:read` -- See Coinbase balances (for display/funding guidance)
   - `offline_access` -- Refresh tokens
   - Consider: `wallet:transactions:send` (if we want to enable sending from Coinbase to the agent wallet address)

3. **Use tauri-plugin-oauth** -- For the desktop redirect flow with PKCE

### Architecture Suggestion

```
+------------------+       +-------------------+       +------------------+
|   Tauri App UI   | ----> | Coinbase OAuth    | ----> | Coinbase Account |
|                  |       | (PKCE + localhost) |       | (balance, email) |
+------------------+       +-------------------+       +------------------+
        |
        |  (auto-fill email)
        v
+------------------+       +-------------------+       +------------------+
| Agentic Wallet   | ----> | Email OTP Flow    | ----> | Agent Wallet     |
| Auth Module      |       | (CLI/API)         |       | (Base, USDC)     |
+------------------+       +-------------------+       +------------------+
```

Two parallel auth systems, connected only by the shared email address.

### Future Possibilities (If Coinbase Adds Support)

- Direct Agentic Wallet funding via Coinbase OAuth `wallet:transactions:send` (send USDC from Coinbase to the agent wallet address)
- Unified identity if Coinbase links Agentic Wallet to retail accounts
- KYC passthrough from Coinbase account to Agentic Wallet

### Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| OAuth partner approval denied/delayed | High | Build email-first flow, add OAuth as enhancement later |
| Scopes insufficient for needs | Medium | Request broad scopes upfront; plan carefully |
| Coinbase changes OAuth program | Medium | Abstract OAuth layer so provider is swappable |
| No Agentic Wallet API bridge emerges | Low | Email auto-fill still provides value |
| PKCE/localhost redirect issues on some OS | Low | Fall back to custom URI scheme |

---

## Appendix: Key URLs

| Resource | URL |
|----------|-----|
| OAuth2 Overview | https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/oauth2 |
| OAuth2 Reference | https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/reference |
| OAuth2 Scopes | https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/scopes |
| OAuth2 Integration Guide | https://docs.cdp.coinbase.com/coinbase-app/authentication-authorization/oauth2/integrations |
| OAuth2 Integration Overview | https://docs.cdp.coinbase.com/coinbase-app/oauth2-integration/overview |
| Sign in with Coinbase Product | https://www.coinbase.com/developer-platform/products/sign-in-with-coinbase |
| Agentic Wallet Docs | https://docs.cdp.coinbase.com/agentic-wallet/welcome |
| Agentic Wallet Auth Skill | https://docs.cdp.coinbase.com/agentic-wallet/skills/authenticate |
| Tauri OAuth Plugin | https://github.com/FabianLars/tauri-plugin-oauth |
| CDP Portal (OAuth Registration) | https://portal.cdp.coinbase.com/projects/api-keys/oauth |
| Create OAuth App Help | https://help.coinbase.com/en/cloud/api/oauth2/create-app |
| Stytch Coinbase OAuth | https://stytch.com/docs/api/oauth-coinbase-start |
