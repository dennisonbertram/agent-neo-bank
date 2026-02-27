# User Onboarding & Wallet Linking Flow

> **Version:** 1.0
> **Date:** 2026-02-27
> **Status:** Draft
> **Parent:** [Architecture Plan](./architecture-plan.md) -- Phase 1a prerequisite

---

## Table of Contents

1. [Overview](#1-overview)
2. [State Machine](#2-state-machine)
3. [Screen-by-Screen Wireframes](#3-screen-by-screen-wireframes)
4. [Tauri IPC Commands](#4-tauri-ipc-commands)
5. [SQLite Schema Additions](#5-sqlite-schema-additions)
6. [Rust Backend: New Modules](#6-rust-backend-new-modules)
7. [React Frontend: New Pages & Components](#7-react-frontend-new-pages--components)
8. [Error Handling](#8-error-handling)
9. [Test Cases](#9-test-cases)
10. [Integration with Existing Architecture](#10-integration-with-existing-architecture)

---

## 1. Overview

### The Problem

The existing architecture defines everything that happens **after** a user is set up: agent registration, spending controls, transaction processing. But it never specifies how a user gets from "I just downloaded this app" to "I have a linked wallet and can start setting up agents."

This document fills that gap. It specifies the complete first-launch-to-dashboard experience.

### Key Principles

1. **No crypto jargon on first screen.** The welcome screen talks about giving AI agents spending power, not about wallets, keys, or blockchains.
2. **One wallet per user (v1).** The app supports exactly one Coinbase Agent Wallet, tied to one email. Multi-wallet is out of scope.
3. **The CLI is invisible.** The user never sees `awal` commands. The app wraps everything.
4. **Wallet creation is automatic.** When a user authenticates with their email, the Coinbase Agent Wallet system creates a wallet if one does not exist. We do not need a separate "create wallet" step.
5. **Session awareness.** The app continuously monitors CLI session health and handles expiration gracefully.

### What the User Experiences

```
Download app --> Open app --> Welcome screen --> Enter email --> Check email for code
--> Enter 6-digit code --> See dashboard with wallet balance --> Start using the app
```

Total steps: 3 user actions (enter email, check email, enter code). Everything else is automated.

---

## 2. State Machine

### 2.1 Primary State Machine (ASCII)

```
                    +------------------+
                    |      FRESH       |
                    | (no user_profile |
                    |  record exists)  |
                    +--------+---------+
                             |
                     app launches,
                     no user_profile found
                             |
                             v
                    +------------------+
                    |   ONBOARDING     |
                    | (welcome screen  |
                    |  displayed)      |
                    +--------+---------+
                             |
                     user clicks
                     "Get Started"
                             |
                             v
                    +------------------+
                    | WALLET_LINKING   |
                    | (email input     |
                    |  screen)         |
                    +--------+---------+
                             |
                     user enters email,
                     app calls `awal auth login <email>`,
                     receives flowId
                             |
                             v
                    +------------------+
                    | OTP_VERIFICATION |
                    | (enter 6-digit   |
                    |  code screen)    |
                    +--------+---------+
                             |
                     user enters OTP,
                     app calls `awal auth verify <flowId> <otp>`,
                     on success: fetch balance + address
                             |
                             v
                    +------------------+
                    |     ACTIVE       |
                    | (dashboard       |
                    |  displayed)      |
                    +--------+---------+
                             |
                     session expires
                     (detected by health check)
                             |
                             v
                    +------------------+
                    | SESSION_EXPIRED  |
                    | (re-auth modal   |
                    |  overlay)        |
                    +--------+---------+
                             |
                     user re-authenticates
                     (same email + OTP flow)
                             |
                             v
                    +------------------+
                    |     ACTIVE       |
                    +------------------+
```

### 2.2 State Transitions Table

| From | To | Trigger | Side Effects |
|---|---|---|---|
| FRESH | ONBOARDING | App launch, no `user_profile` row | Create `user_profile` row with state=ONBOARDING |
| ONBOARDING | WALLET_LINKING | User clicks "Get Started" | Update state |
| WALLET_LINKING | OTP_VERIFICATION | Email submitted, `awal auth login` succeeds | Store email + flowId in user_profile |
| WALLET_LINKING | WALLET_LINKING | `awal auth login` fails (invalid email, network error) | Show error, stay on screen |
| OTP_VERIFICATION | ACTIVE | OTP verified via `awal auth verify` | Fetch balance + address, store wallet_address, update state |
| OTP_VERIFICATION | OTP_VERIFICATION | Wrong OTP entered | Show error, allow retry |
| OTP_VERIFICATION | WALLET_LINKING | OTP expired (user clicks "Resend") | Clear flowId, restart login |
| ACTIVE | SESSION_EXPIRED | `awal auth status` returns unauthenticated | Show re-auth overlay |
| SESSION_EXPIRED | ACTIVE | Re-auth OTP flow completes | Dismiss overlay, refresh balance |
| Any | FRESH | User clicks "Reset" in settings (factory reset) | Delete user_profile, clear all data |

### 2.3 State Storage

State is stored in the `app_config` table (existing) with the key `onboarding_state`. The full user profile lives in a new `user_profile` table (see Section 5).

```
app_config:
  key: "onboarding_state"
  value: "FRESH" | "ONBOARDING" | "WALLET_LINKING" | "OTP_VERIFICATION" | "ACTIVE" | "SESSION_EXPIRED"
```

---

## 3. Screen-by-Screen Wireframes

### 3.1 Welcome Screen (State: ONBOARDING)

```
+------------------------------------------------------------------+
|                                                                    |
|                                                                    |
|                         [App Logo/Icon]                            |
|                                                                    |
|                    Agent Neo Bank                                   |
|                                                                    |
|              Give your AI agents spending power.                   |
|                                                                    |
|         Set up a wallet, define budgets, and let your              |
|         AI agents pay for services autonomously --                 |
|         with guardrails you control.                               |
|                                                                    |
|                                                                    |
|                  +---------------------------+                     |
|                  |       Get Started          |                    |
|                  +---------------------------+                     |
|                       (primary Button)                             |
|                                                                    |
|                                                                    |
|              Powered by Coinbase Agent Wallet                      |
|                                                                    |
+------------------------------------------------------------------+
```

**shadcn components:**
- `Card` (centered, max-w-md)
- `Button` (primary, full-width within card)
- Text uses `text-muted-foreground` for subtitle

**Behavior:**
- "Get Started" transitions to WALLET_LINKING
- No other actions available
- App logo uses the app icon from `src-tauri/icons/`

### 3.2 Email Input Screen (State: WALLET_LINKING)

```
+------------------------------------------------------------------+
|                                                                    |
|   < Back                                                           |
|                                                                    |
|                    Connect your wallet                             |
|                                                                    |
|         Enter your email to set up your Agent Wallet.              |
|         We'll send you a verification code.                        |
|                                                                    |
|         +--------------------------------------------+             |
|         |  Email address                              |            |
|         +--------------------------------------------+             |
|                                                                    |
|         +--------------------------------------------+             |
|         |  Display name (optional)                    |            |
|         +--------------------------------------------+             |
|                                                                    |
|                  +---------------------------+                     |
|                  |    Send Verification Code  |                    |
|                  +---------------------------+                     |
|                                                                    |
|         By continuing, you agree to the terms of the               |
|         Coinbase Agent Wallet.                                     |
|                                                                    |
|                                                                    |
|   +-----------------------------------------------------------+   |
|   | (i) What happens next?                                     |   |
|   |                                                            |   |
|   | 1. A wallet is created (or retrieved) for your email       |   |
|   | 2. You'll receive a 6-digit code at this address           |   |
|   | 3. Once verified, your wallet is linked to this app        |   |
|   +-----------------------------------------------------------+   |
|                                                                    |
+------------------------------------------------------------------+
```

**shadcn components:**
- `Input` (email, with label)
- `Input` (display name, optional)
- `Button` (primary, "Send Verification Code")
- `Button` (ghost, "< Back")
- `Alert` or `Collapsible` for the info box

**Behavior:**
- Validates email format client-side before submission
- On submit: calls `awal auth login <email>` via Tauri command
- Shows loading spinner on button while CLI executes
- On success: stores email + flowId, transitions to OTP_VERIFICATION
- On error: shows inline error message (see Section 8)
- "Back" returns to ONBOARDING (welcome screen)
- Display name is stored locally only (never sent to Coinbase)

### 3.3 OTP Verification Screen (State: OTP_VERIFICATION)

```
+------------------------------------------------------------------+
|                                                                    |
|   < Back                                                           |
|                                                                    |
|                    Check your email                                |
|                                                                    |
|         We sent a 6-digit verification code to                     |
|         user@example.com                                           |
|                                                                    |
|         +---+ +---+ +---+ +---+ +---+ +---+                       |
|         | _ | | _ | | _ | | _ | | _ | | _ |                       |
|         +---+ +---+ +---+ +---+ +---+ +---+                       |
|                                                                    |
|         Didn't receive it?  Resend code                            |
|                             ^^^^^^^^^^^                            |
|                             (link button)                          |
|                                                                    |
|                  +---------------------------+                     |
|                  |        Verify              |                    |
|                  +---------------------------+                     |
|                                                                    |
|         Code expires in 4:32                                       |
|                                                                    |
+------------------------------------------------------------------+
```

**shadcn components:**
- `InputOTP` (6-digit, auto-focus, auto-advance)
- `Button` (primary, "Verify")
- `Button` (link variant, "Resend code")
- `Button` (ghost, "< Back")
- Countdown timer text

**Behavior:**
- Auto-focuses first OTP input on mount
- Each digit auto-advances to next input
- On 6th digit entered: auto-submits (calls `awal auth verify <flowId> <otp>`)
- Also supports manual "Verify" button click
- Shows loading state during verification
- On success: fetches balance + address, stores wallet_address, transitions to ACTIVE
- On failure: shows error, clears inputs, refocuses first input
- "Resend code": calls `awal auth login <email>` again with stored email, resets flowId + timer
- "Back": returns to WALLET_LINKING (email input), clears flowId
- 5-minute countdown timer (estimated OTP validity). At zero, shows "Code may have expired. Click Resend."
- Resend has a 60-second cooldown between attempts

### 3.4 Dashboard (State: ACTIVE, first visit)

```
+------------------------------------------------------------------+
|  [sidebar]  |                                                      |
|             |   Welcome, Dennis!                                   |
|  Dashboard  |                                                      |
|  Agents     |   +---------------------------------------------+   |
|  History    |   |  Wallet Balance                               |  |
|  Approvals  |   |                                               |  |
|  Settings   |   |  $0.00 USDC                                   |  |
|             |   |                                               |  |
|             |   |  0x1a2b...3c4d  [copy icon]                   |  |
|             |   |                                               |  |
|             |   |  Network: Base Sepolia (testnet)              |  |
|             |   +---------------------------------------------+   |
|             |                                                      |
|             |   +---------------------------------------------+   |
|             |   |  Fund your wallet                             |  |
|             |   |                                               |  |
|             |   |  Your wallet is empty. Add funds to start     |  |
|             |   |  giving your AI agents spending power.        |  |
|             |   |                                               |  |
|             |   |  [  Add Funds  ]   [  Copy Address  ]         |  |
|             |   +---------------------------------------------+   |
|             |                                                      |
|             |   +---------------------------------------------+   |
|             |   |  Your AI Agents                               |  |
|             |   |                                               |  |
|             |   |  No agents yet. Set up your first agent to    |  |
|             |   |  get started.                                 |  |
|             |   |                                               |  |
|             |   |  [  Set Up Your First Agent  ]                |  |
|             |   +---------------------------------------------+   |
|             |                                                      |
+------------------------------------------------------------------+
```

**shadcn components:**
- `Shell` layout (existing from architecture)
- `Card` for balance display
- `Card` with `EmptyState` for fund CTA
- `Card` with `EmptyState` for agent CTA
- `Badge` for network indicator
- `Button` (primary, "Add Funds")
- `Button` (outline, "Copy Address")
- `Button` (primary, "Set Up Your First Agent")

**Behavior:**
- Balance refreshes on mount and every 30 seconds (via existing BalanceCache)
- "Add Funds" navigates to `/fund` (existing Fund page)
- "Copy Address" copies wallet address to clipboard via `tauri-plugin-clipboard-manager`
- "Set Up Your First Agent" navigates to `/agents/new` (agent creation flow)
- If balance > 0 on first visit, fund CTA is still shown but messaging changes to "Add more funds"
- Sidebar navigation is fully visible and functional

### 3.5 Session Expired Overlay (State: SESSION_EXPIRED)

```
+------------------------------------------------------------------+
|                                                                    |
|  [entire dashboard is visible but dimmed/blurred]                  |
|                                                                    |
|       +-----------------------------------------------+           |
|       |                                               |           |
|       |        Session Expired                        |           |
|       |                                               |           |
|       |   Your wallet session has expired.            |           |
|       |   Re-verify your email to continue.           |           |
|       |                                               |           |
|       |   user@example.com                            |           |
|       |                                               |           |
|       |   [  Send Verification Code  ]                |           |
|       |                                               |           |
|       |   Use a different email                       |           |
|       |   ^^^^^^^^^^^^^^^^^^^^^^^^                    |           |
|       |   (link - goes to full re-onboarding)         |           |
|       |                                               |           |
|       +-----------------------------------------------+           |
|                                                                    |
+------------------------------------------------------------------+
```

**shadcn components:**
- `Dialog` (modal, non-dismissable via overlay click)
- `Button` (primary, "Send Verification Code")
- `Button` (link variant, "Use a different email")
- Blurred/dimmed backdrop

**Behavior:**
- Modal cannot be dismissed by clicking outside or pressing Escape
- Pre-fills the stored email address
- "Send Verification Code" triggers the same OTP flow as onboarding
- After successful verification: modal dismisses, dashboard refreshes
- "Use a different email" transitions to WALLET_LINKING (full re-onboarding)
- While in SESSION_EXPIRED state, all Tauri commands that require CLI access return a `SessionExpired` error to the frontend, which the error boundary catches and shows this overlay

---

## 4. Tauri IPC Commands

### 4.1 New Commands

| Command | Module | Input | Output | Description |
|---|---|---|---|---|
| `get_onboarding_state` | `commands::onboarding` | (none) | `OnboardingState` enum | Returns current state from app_config |
| `get_user_profile` | `commands::onboarding` | (none) | `Option<UserProfile>` | Returns user profile if exists |
| `set_display_name` | `commands::onboarding` | `name: String` | `()` | Updates display name in user_profile |
| `initiate_wallet_link` | `commands::onboarding` | `email: String` | `WalletLinkResult { flow_id: String }` | Calls `awal auth login <email>`, stores email + flowId |
| `verify_wallet_link` | `commands::onboarding` | `flow_id: String, otp: String` | `WalletVerifyResult { address: String, balance: BalanceInfo }` | Calls `awal auth verify`, fetches balance + address |
| `check_session_health` | `commands::onboarding` | (none) | `SessionStatus { valid: bool, email: Option<String> }` | Calls `awal auth status` |
| `resend_otp` | `commands::onboarding` | (none) | `WalletLinkResult { flow_id: String }` | Re-sends OTP using stored email |
| `reset_onboarding` | `commands::onboarding` | (none) | `()` | Factory reset: deletes user_profile, clears session |

### 4.2 Modified Commands

| Command | Change |
|---|---|
| `auth_login` | Now delegates to `initiate_wallet_link` internally (same CLI call, but updates onboarding state) |
| `auth_verify` | Now delegates to `verify_wallet_link` internally |
| `auth_status` | Now also updates onboarding state if session is expired |

### 4.3 Command Implementations

```rust
// commands/onboarding.rs

#[tauri::command]
async fn get_onboarding_state(
    state: tauri::State<'_, Arc<CoreServices>>,
) -> Result<OnboardingState, AppError> {
    state.user_service.get_onboarding_state().await
}

#[tauri::command]
async fn initiate_wallet_link(
    state: tauri::State<'_, Arc<CoreServices>>,
    email: String,
    display_name: Option<String>,
) -> Result<WalletLinkResult, AppError> {
    // Validate email format
    if !is_valid_email(&email) {
        return Err(AppError::InvalidEmail);
    }

    // Create or update user profile
    state.user_service.upsert_profile(&email, display_name.as_deref()).await?;

    // Call CLI: awal auth login <email>
    let output = state.cli.run(AwalCommand::AuthLogin { email: email.clone() }).await?;
    let flow_id = parse_flow_id(&output)?;

    // Store flow_id and update state
    state.user_service.set_pending_flow(&flow_id).await?;
    state.user_service.set_onboarding_state(OnboardingState::OtpVerification).await?;

    Ok(WalletLinkResult { flow_id })
}

#[tauri::command]
async fn verify_wallet_link(
    state: tauri::State<'_, Arc<CoreServices>>,
    flow_id: String,
    otp: String,
) -> Result<WalletVerifyResult, AppError> {
    // Call CLI: awal auth verify <flowId> <otp>
    let output = state.cli.run(AwalCommand::AuthVerify {
        flow_id: flow_id.clone(),
        otp,
    }).await?;

    if !output.success {
        return Err(AppError::OtpVerificationFailed(output.stderr));
    }

    // Fetch wallet address
    let addr_output = state.cli.run(AwalCommand::GetAddress).await?;
    let address = parse_address(&addr_output)?;

    // Fetch balance
    let balance = state.wallet_service.get_balance().await?;

    // Update user profile with wallet address
    state.user_service.set_wallet_address(&address).await?;
    state.user_service.set_onboarding_state(OnboardingState::Active).await?;

    // Clear pending flow
    state.user_service.clear_pending_flow().await?;

    Ok(WalletVerifyResult { address, balance })
}

#[tauri::command]
async fn check_session_health(
    state: tauri::State<'_, Arc<CoreServices>>,
) -> Result<SessionStatus, AppError> {
    state.session_manager.check_health().await
}
```

---

## 5. SQLite Schema Additions

### 5.1 New Table: `user_profile`

```sql
-- Part of 001_initial.sql (or a new migration if schema already exists)

-- User profile (one row, single-user app)
CREATE TABLE IF NOT EXISTS user_profile (
    id              TEXT PRIMARY KEY DEFAULT 'default',  -- Always 'default' (single user)
    display_name    TEXT DEFAULT '',
    email           TEXT DEFAULT '',                     -- Coinbase Agent Wallet email
    wallet_address  TEXT DEFAULT '',                     -- Base address (0x...)
    pending_flow_id TEXT DEFAULT '',                     -- Active OTP flow ID (cleared after verify)
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);
```

### 5.2 New Entry in `app_config`

```sql
-- Onboarding state tracking
INSERT OR IGNORE INTO app_config (key, value) VALUES ('onboarding_state', 'FRESH');

-- Session health check interval (milliseconds)
INSERT OR IGNORE INTO app_config (key, value) VALUES ('session_check_interval_ms', '300000');

-- Last successful session check timestamp
INSERT OR IGNORE INTO app_config (key, value) VALUES ('last_session_check', '0');

-- OTP resend cooldown (seconds)
INSERT OR IGNORE INTO app_config (key, value) VALUES ('otp_resend_cooldown_seconds', '60');
```

### 5.3 Relationship to Existing Schema

The `user_profile` table is independent of the `agents` table. The user is the human operator, not an agent. The `email` in `user_profile` is the same email used to authenticate with the Coinbase Agent Wallet CLI. The `wallet_address` is the on-chain address of the wallet.

No changes to existing tables are needed. The `app_config` table already exists and supports arbitrary key-value pairs.

---

## 6. Rust Backend: New Modules

### 6.1 `core/user_service.rs`

Manages the human user's profile and onboarding state.

```rust
// core/user_service.rs

pub struct UserService {
    db: Arc<Database>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OnboardingState {
    Fresh,
    Onboarding,
    WalletLinking,
    OtpVerification,
    Active,
    SessionExpired,
}

impl OnboardingState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Fresh => "FRESH",
            Self::Onboarding => "ONBOARDING",
            Self::WalletLinking => "WALLET_LINKING",
            Self::OtpVerification => "OTP_VERIFICATION",
            Self::Active => "ACTIVE",
            Self::SessionExpired => "SESSION_EXPIRED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ONBOARDING" => Self::Onboarding,
            "WALLET_LINKING" => Self::WalletLinking,
            "OTP_VERIFICATION" => Self::OtpVerification,
            "ACTIVE" => Self::Active,
            "SESSION_EXPIRED" => Self::SessionExpired,
            _ => Self::Fresh,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub wallet_address: String,
    pub pending_flow_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl UserService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_onboarding_state(&self) -> Result<OnboardingState, AppError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            let value: String = conn.query_row(
                "SELECT value FROM app_config WHERE key = 'onboarding_state'",
                [],
                |row| row.get(0),
            ).unwrap_or_else(|_| "FRESH".to_string());
            Ok(OnboardingState::from_str(&value))
        }).await?
    }

    pub async fn set_onboarding_state(&self, state: OnboardingState) -> Result<(), AppError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value) VALUES ('onboarding_state', ?1)",
                params![state.as_str()],
            )?;
            Ok(())
        }).await?
    }

    pub async fn get_profile(&self) -> Result<Option<UserProfile>, AppError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            let result = conn.query_row(
                "SELECT * FROM user_profile WHERE id = 'default'",
                [],
                |row| Ok(UserProfile {
                    id: row.get("id")?,
                    display_name: row.get("display_name")?,
                    email: row.get("email")?,
                    wallet_address: row.get("wallet_address")?,
                    pending_flow_id: row.get("pending_flow_id")?,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                }),
            );
            match result {
                Ok(profile) => Ok(Some(profile)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(AppError::from(e)),
            }
        }).await?
    }

    pub async fn upsert_profile(
        &self,
        email: &str,
        display_name: Option<&str>,
    ) -> Result<(), AppError> {
        let db = self.db.clone();
        let email = email.to_string();
        let name = display_name.unwrap_or("").to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            let now = Utc::now().timestamp();
            conn.execute(
                "INSERT INTO user_profile (id, display_name, email, created_at, updated_at)
                 VALUES ('default', ?1, ?2, ?3, ?3)
                 ON CONFLICT(id) DO UPDATE SET
                   email = ?2,
                   display_name = CASE WHEN ?1 = '' THEN display_name ELSE ?1 END,
                   updated_at = ?3",
                params![name, email, now],
            )?;
            Ok(())
        }).await?
    }

    pub async fn set_wallet_address(&self, address: &str) -> Result<(), AppError> {
        let db = self.db.clone();
        let address = address.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            let now = Utc::now().timestamp();
            conn.execute(
                "UPDATE user_profile SET wallet_address = ?1, updated_at = ?2 WHERE id = 'default'",
                params![address, now],
            )?;
            Ok(())
        }).await?
    }

    pub async fn set_pending_flow(&self, flow_id: &str) -> Result<(), AppError> {
        let db = self.db.clone();
        let flow_id = flow_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            let now = Utc::now().timestamp();
            conn.execute(
                "UPDATE user_profile SET pending_flow_id = ?1, updated_at = ?2 WHERE id = 'default'",
                params![flow_id, now],
            )?;
            Ok(())
        }).await?
    }

    pub async fn clear_pending_flow(&self) -> Result<(), AppError> {
        self.set_pending_flow("").await
    }

    /// Factory reset: delete user profile and reset onboarding state.
    pub async fn reset(&self) -> Result<(), AppError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            conn.execute("DELETE FROM user_profile WHERE id = 'default'", [])?;
            conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value) VALUES ('onboarding_state', 'FRESH')",
                [],
            )?;
            Ok(())
        }).await?
    }
}
```

### 6.2 `core/session_manager.rs`

Monitors CLI session health and triggers re-authentication when needed.

```rust
// core/session_manager.rs

pub struct SessionManager {
    cli: Arc<dyn CliExecutable>,
    user_service: Arc<UserService>,
    event_bus: Arc<EventBus>,
    check_interval: Duration,  // Default: 5 minutes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub valid: bool,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
}

impl SessionManager {
    pub fn new(
        cli: Arc<dyn CliExecutable>,
        user_service: Arc<UserService>,
        event_bus: Arc<EventBus>,
        check_interval: Duration,
    ) -> Self {
        Self { cli, user_service, event_bus, check_interval }
    }

    /// Check session health by calling `awal auth status`.
    pub async fn check_health(&self) -> Result<SessionStatus, AppError> {
        let output = self.cli.run(AwalCommand::AuthStatus).await?;

        let profile = self.user_service.get_profile().await?;

        if output.success {
            // Session is valid
            Ok(SessionStatus {
                valid: true,
                email: profile.as_ref().map(|p| p.email.clone()),
                wallet_address: profile.as_ref().map(|p| p.wallet_address.clone()),
            })
        } else {
            // Session expired -- update state
            let current_state = self.user_service.get_onboarding_state().await?;
            if current_state == OnboardingState::Active {
                self.user_service.set_onboarding_state(OnboardingState::SessionExpired).await?;
                self.event_bus.emit(Event::SessionExpired);
            }
            Ok(SessionStatus {
                valid: false,
                email: profile.as_ref().map(|p| p.email.clone()),
                wallet_address: profile.as_ref().map(|p| p.wallet_address.clone()),
            })
        }
    }

    /// Background task that periodically checks session health.
    /// Spawned in the Tauri setup hook.
    pub async fn run_health_check_loop(&self) {
        let mut interval = tokio::time::interval(self.check_interval);
        loop {
            interval.tick().await;

            // Only check if we're in ACTIVE state
            let state = self.user_service.get_onboarding_state().await;
            if let Ok(OnboardingState::Active) = state {
                if let Err(e) = self.check_health().await {
                    tracing::warn!("Session health check failed: {}", e);
                }
            }
        }
    }
}
```

### 6.3 Updates to `core/services.rs` (CoreServices)

Add `UserService` and `SessionManager` to the `CoreServices` struct:

```rust
// Addition to core/services.rs

pub struct CoreServices {
    // ... existing fields ...
    pub user_service: Arc<UserService>,
    pub session_manager: Arc<SessionManager>,
}

impl CoreServices {
    pub async fn new(db: Arc<Database>, cli: Arc<dyn CliExecutable>, config: Config) -> Result<Self, AppError> {
        let user_service = Arc::new(UserService::new(db.clone()));
        let event_bus = Arc::new(EventBus::new());

        let session_manager = Arc::new(SessionManager::new(
            cli.clone(),
            user_service.clone(),
            event_bus.clone(),
            Duration::from_millis(config.session_check_interval_ms),
        ));

        // ... existing initialization ...

        Ok(Self {
            // ... existing fields ...
            user_service,
            session_manager,
        })
    }
}
```

### 6.4 Updates to `main.rs` (Startup)

The startup flow changes to account for onboarding state:

```rust
// Updated main.rs setup hook

// 1. Initialize database
let db = Arc::new(Database::new(&data_dir)?);
db.run_migrations().await?;

// 2. Check onboarding state BEFORE checking CLI
let onboarding_state = db.get_app_config("onboarding_state")
    .unwrap_or("FRESH".to_string());

// 3. Build CLI executor
let cli: Arc<dyn CliExecutable> = if config.mock_mode {
    Arc::new(MockCliExecutor::new())
} else {
    let real_cli = match RealCliExecutor::new(&config.awal_binary_path) {
        Ok(cli) => cli,
        Err(_) => {
            // CLI not found -- this is OK during onboarding.
            // The user may not have awal installed yet.
            // We'll handle this in the onboarding flow.
            // For now, use a "pending" CLI that errors on all calls.
            Arc::new(PendingCliExecutor::new()) // Returns CliNotInstalled error on all calls
        }
    };

    // Only check auth status if user is past onboarding
    if onboarding_state == "ACTIVE" {
        match real_cli.run(AwalCommand::AuthStatus).await {
            Ok(output) if output.success => Arc::new(real_cli),
            Ok(_) => {
                // Session expired -- update state, continue with the CLI
                db.set_app_config("onboarding_state", "SESSION_EXPIRED");
                Arc::new(real_cli)
            }
            Err(_) => Arc::new(PendingCliExecutor::new()),
        }
    } else {
        Arc::new(real_cli)
    }
};

// 4. Build CoreServices (now includes UserService + SessionManager)
let core = Arc::new(CoreServices::new(db, cli, config).await?);

// 5. Spawn background session health check
let core_for_health = core.clone();
tauri::async_runtime::spawn(async move {
    core_for_health.session_manager.run_health_check_loop().await;
});

// 6. Register all commands including new onboarding commands
app.manage(core.clone());
```

### 6.5 New Error Variants

Add to `error.rs`:

```rust
pub enum AppError {
    // ... existing variants ...

    // Onboarding errors
    InvalidEmail,
    OtpVerificationFailed(String),
    OtpExpired,
    FlowIdNotFound,
    SessionExpired,
    CliNotInstalled,
    WalletLinkFailed(String),
    OtpResendCooldown { remaining_seconds: u64 },
}
```

---

## 7. React Frontend: New Pages & Components

### 7.1 New Pages

| Page | Path | Description |
|---|---|---|
| `Onboarding.tsx` | `/onboarding` | Multi-step onboarding flow (already listed in architecture, now fully specified) |

The `Onboarding.tsx` page is a single page that renders different step components based on the onboarding state. It does NOT use separate routes for each step.

### 7.2 New Components

```
src/components/onboarding/
  WelcomeStep.tsx       -- Welcome screen with "Get Started" CTA
  EmailStep.tsx         -- Email input + display name
  OtpStep.tsx           -- 6-digit OTP input with timer
  FundStep.tsx          -- Post-link funding CTA (optional, shown on first dashboard visit)
  SessionExpiredModal.tsx -- Re-auth overlay when session expires
```

### 7.3 New Hooks

```
src/hooks/
  useOnboarding.ts      -- Manages onboarding state machine
  useSessionHealth.ts   -- Polls session health, triggers re-auth overlay
```

### 7.4 New Store

```
src/stores/
  onboardingStore.ts    -- Zustand store for onboarding state + user profile
```

### 7.5 Component Specifications

#### `useOnboarding.ts`

```typescript
// hooks/useOnboarding.ts

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

type OnboardingState =
  | 'FRESH'
  | 'ONBOARDING'
  | 'WALLET_LINKING'
  | 'OTP_VERIFICATION'
  | 'ACTIVE'
  | 'SESSION_EXPIRED';

interface UserProfile {
  id: string;
  display_name: string;
  email: string;
  wallet_address: string;
  pending_flow_id: string;
}

interface OnboardingStore {
  state: OnboardingState;
  profile: UserProfile | null;
  flowId: string | null;
  error: string | null;
  loading: boolean;

  // Actions
  initialize: () => Promise<void>;
  startOnboarding: () => void;
  submitEmail: (email: string, displayName?: string) => Promise<void>;
  submitOtp: (otp: string) => Promise<void>;
  resendOtp: () => Promise<void>;
  goBack: () => void;
  reset: () => Promise<void>;
}

export const useOnboardingStore = create<OnboardingStore>((set, get) => ({
  state: 'FRESH',
  profile: null,
  flowId: null,
  error: null,
  loading: false,

  initialize: async () => {
    const state = await invoke<OnboardingState>('get_onboarding_state');
    const profile = await invoke<UserProfile | null>('get_user_profile');
    set({ state, profile });
  },

  startOnboarding: () => {
    set({ state: 'WALLET_LINKING', error: null });
  },

  submitEmail: async (email, displayName) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<{ flow_id: string }>('initiate_wallet_link', {
        email,
        displayName,
      });
      set({
        state: 'OTP_VERIFICATION',
        flowId: result.flow_id,
        loading: false,
      });
    } catch (e: any) {
      set({ error: e.message || 'Failed to send verification code', loading: false });
    }
  },

  submitOtp: async (otp) => {
    const { flowId } = get();
    if (!flowId) {
      set({ error: 'No active verification flow. Please go back and try again.' });
      return;
    }
    set({ loading: true, error: null });
    try {
      const result = await invoke<{ address: string; balance: any }>('verify_wallet_link', {
        flowId,
        otp,
      });
      const profile = await invoke<UserProfile>('get_user_profile');
      set({
        state: 'ACTIVE',
        profile,
        flowId: null,
        loading: false,
      });
    } catch (e: any) {
      set({ error: e.message || 'Invalid verification code', loading: false });
    }
  },

  resendOtp: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<{ flow_id: string }>('resend_otp');
      set({ flowId: result.flow_id, loading: false });
    } catch (e: any) {
      set({ error: e.message || 'Failed to resend code', loading: false });
    }
  },

  goBack: () => {
    const { state } = get();
    if (state === 'WALLET_LINKING') set({ state: 'ONBOARDING', error: null });
    if (state === 'OTP_VERIFICATION') set({ state: 'WALLET_LINKING', error: null, flowId: null });
  },

  reset: async () => {
    await invoke('reset_onboarding');
    set({ state: 'FRESH', profile: null, flowId: null, error: null });
  },
}));
```

#### `useSessionHealth.ts`

```typescript
// hooks/useSessionHealth.ts

import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useOnboardingStore } from '../stores/onboardingStore';

const CHECK_INTERVAL_MS = 300_000; // 5 minutes

export function useSessionHealth() {
  const intervalRef = useRef<NodeJS.Timeout>();
  const setState = useOnboardingStore((s) => s.state);

  useEffect(() => {
    // Listen for session expired events from the backend
    const unlisten = listen('session-expired', () => {
      useOnboardingStore.setState({ state: 'SESSION_EXPIRED' });
    });

    // Periodic polling as a fallback
    intervalRef.current = setInterval(async () => {
      const currentState = useOnboardingStore.getState().state;
      if (currentState !== 'ACTIVE') return;

      try {
        const status = await invoke<{ valid: boolean }>('check_session_health');
        if (!status.valid) {
          useOnboardingStore.setState({ state: 'SESSION_EXPIRED' });
        }
      } catch {
        // Swallow errors -- health check is best-effort
      }
    }, CHECK_INTERVAL_MS);

    return () => {
      unlisten.then((fn) => fn());
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);
}
```

#### `App.tsx` Router Integration

```typescript
// Updated App.tsx routing logic

function App() {
  const { state, initialize } = useOnboardingStore();

  useEffect(() => {
    initialize();
  }, []);

  useSessionHealth();

  // If not yet active, show onboarding
  if (state !== 'ACTIVE' && state !== 'SESSION_EXPIRED') {
    return <Onboarding />;
  }

  // If active but session expired, show dashboard with overlay
  return (
    <>
      <Shell>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/agents" element={<AgentList />} />
          <Route path="/agents/:id" element={<AgentDetail />} />
          <Route path="/transactions" element={<Transactions />} />
          <Route path="/approvals" element={<Approvals />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/fund" element={<Fund />} />
        </Routes>
      </Shell>
      {state === 'SESSION_EXPIRED' && <SessionExpiredModal />}
    </>
  );
}
```

---

## 8. Error Handling

### 8.1 Error Scenarios and User-Facing Messages

| Scenario | Detection | User Message | Recovery Action |
|---|---|---|---|
| **Invalid email format** | Client-side regex | "Please enter a valid email address." | Fix email and resubmit |
| **CLI not installed** | `RealCliExecutor::new()` fails | "The Coinbase Agent Wallet CLI (awal) is not installed. Install it with: npm install -g awal" | Show installation instructions with copy button |
| **CLI not in PATH** | `awal auth login` returns "command not found" | Same as above | Same as above |
| **Node.js not installed** | `npx` command fails | "Node.js v24+ is required. Please install it from nodejs.org" | Link to Node.js download |
| **Network error during login** | CLI returns network/timeout error | "Could not connect to the authentication service. Check your internet connection and try again." | Retry button |
| **Invalid OTP** | `awal auth verify` returns auth error | "Invalid verification code. Please check and try again." | Clear OTP inputs, refocus |
| **Expired OTP** | `awal auth verify` returns expiry error | "This code has expired. Click 'Resend' to get a new one." | Show Resend button prominently |
| **Rate limited (too many OTP attempts)** | CLI returns rate limit error | "Too many attempts. Please wait a few minutes before trying again." | Disable inputs for cooldown period |
| **Session expired** | `awal auth status` returns unauthenticated | (Session expired overlay -- see Section 3.5) | Re-auth flow |
| **CLI process crash** | `tokio::process::Command` returns non-zero exit | "Something went wrong. Please try again." | Retry button, log full error |
| **Unknown error** | Catch-all | "An unexpected error occurred. If this persists, restart the app." | Retry button, link to support |

### 8.2 Error Display Patterns

All errors follow the same visual pattern using shadcn components:

```
+--------------------------------------------+
| (!) Error message text here                 |
|                                             |
|     [ Try Again ]                           |
+--------------------------------------------+
```

- Use `Alert` variant `destructive` for errors
- Use `Alert` variant `default` for informational messages
- Errors appear inline below the relevant form field or action
- Network errors include a "Try Again" button
- OTP errors clear the input and refocus

### 8.3 CLI Not Installed: Special Handling

If the awal CLI is not detected at startup AND the user is in onboarding, show a special installation step:

```
+------------------------------------------------------------------+
|                                                                    |
|                    One more thing...                               |
|                                                                    |
|         Agent Neo Bank uses the Coinbase Agent Wallet CLI.         |
|         You'll need to install it before continuing.               |
|                                                                    |
|         Run this in your terminal:                                 |
|                                                                    |
|         +--------------------------------------------+             |
|         |  npm install -g awal                        |  [copy]    |
|         +--------------------------------------------+             |
|                                                                    |
|         Requires Node.js v24+                                      |
|                                                                    |
|                  +---------------------------+                     |
|                  |    I've installed it       |                    |
|                  +---------------------------+                     |
|                                                                    |
|         (clicks this to retry CLI detection)                       |
|                                                                    |
+------------------------------------------------------------------+
```

This screen appears between Welcome and Email Input if the CLI is not found. The "I've installed it" button retries CLI detection.

---

## 9. Test Cases

### 9.1 Rust Backend Tests

#### UserService Tests

```
test_get_onboarding_state_returns_fresh_for_new_db
test_set_onboarding_state_persists_to_app_config
test_upsert_profile_creates_new_profile
test_upsert_profile_updates_existing_email
test_upsert_profile_preserves_display_name_when_empty
test_set_wallet_address_updates_profile
test_set_pending_flow_stores_flow_id
test_clear_pending_flow_resets_to_empty
test_reset_deletes_profile_and_resets_state
test_get_profile_returns_none_when_no_profile
```

#### SessionManager Tests

```
test_check_health_returns_valid_when_cli_authenticated
test_check_health_returns_invalid_when_cli_not_authenticated
test_check_health_updates_state_to_session_expired
test_check_health_does_not_update_state_if_not_active
test_check_health_includes_email_and_address_from_profile
test_health_check_loop_only_runs_when_active (mock interval)
```

#### Onboarding Command Tests

```
test_initiate_wallet_link_validates_email_format
test_initiate_wallet_link_calls_awal_auth_login
test_initiate_wallet_link_stores_flow_id
test_initiate_wallet_link_updates_state_to_otp_verification
test_initiate_wallet_link_rejects_invalid_email
test_verify_wallet_link_calls_awal_auth_verify
test_verify_wallet_link_fetches_address_on_success
test_verify_wallet_link_updates_state_to_active
test_verify_wallet_link_returns_error_on_wrong_otp
test_verify_wallet_link_clears_pending_flow_on_success
test_resend_otp_calls_awal_auth_login_with_stored_email
test_resend_otp_returns_new_flow_id
test_reset_onboarding_clears_all_user_data
```

#### CLI Wrapper Tests (additions)

```
test_auth_login_parses_flow_id_from_output
test_auth_verify_parses_success_response
test_auth_status_parses_authenticated_response
test_auth_status_parses_unauthenticated_response
test_mock_executor_returns_realistic_auth_responses
```

### 9.2 React Frontend Tests

#### Onboarding Flow Tests

```
test_renders_welcome_step_when_state_is_fresh
test_welcome_step_transitions_to_email_on_get_started
test_email_step_validates_email_format
test_email_step_shows_error_on_invalid_email
test_email_step_calls_initiate_wallet_link
test_email_step_shows_loading_during_submission
test_email_step_shows_error_on_failure
test_email_step_back_button_returns_to_welcome
test_otp_step_renders_six_digit_inputs
test_otp_step_auto_advances_between_inputs
test_otp_step_auto_submits_on_sixth_digit
test_otp_step_shows_error_on_wrong_code
test_otp_step_clears_inputs_on_error
test_otp_step_resend_triggers_new_otp
test_otp_step_resend_has_cooldown
test_otp_step_back_returns_to_email
test_otp_step_shows_countdown_timer
test_session_expired_modal_appears_when_expired
test_session_expired_modal_cannot_be_dismissed
test_session_expired_modal_pre_fills_email
test_session_expired_modal_re_auth_flow_works
test_dashboard_shows_after_successful_onboarding
test_dashboard_shows_fund_cta_when_balance_zero
test_onboarding_state_persists_across_app_restart
```

#### useSessionHealth Tests

```
test_starts_polling_when_active
test_stops_polling_when_not_active
test_sets_session_expired_on_health_check_failure
test_listens_for_session_expired_event
test_cleans_up_interval_on_unmount
```

### 9.3 Integration Tests

```
test_full_onboarding_flow_with_mock_cli (fresh -> active)
test_session_expiry_and_recovery (active -> expired -> active)
test_app_restart_resumes_from_otp_verification
test_app_restart_resumes_from_active
test_factory_reset_returns_to_fresh
test_cli_not_installed_shows_installation_step
```

---

## 10. Integration with Existing Architecture

### 10.1 What Changes

| Component | Change | Details |
|---|---|---|
| `CoreServices` struct | **Add fields** | `user_service: Arc<UserService>`, `session_manager: Arc<SessionManager>` |
| `main.rs` setup | **Modify** | Check onboarding state before CLI health check; spawn health check loop |
| `error.rs` | **Add variants** | `InvalidEmail`, `OtpVerificationFailed`, `SessionExpired`, etc. |
| `db/schema.rs` | **Add table** | `user_profile` table; new `app_config` entries |
| `commands/mod.rs` | **Add module** | Register `commands::onboarding` module and its commands |
| `App.tsx` | **Modify** | Root-level routing now checks onboarding state before rendering shell |
| `authStore.ts` | **Modify** | Integrate with `onboardingStore` for session state |

### 10.2 What's New

| Component | Location | Description |
|---|---|---|
| `UserService` | `core/user_service.rs` | Manages user profile and onboarding state |
| `SessionManager` | `core/session_manager.rs` | Background health checks, session expiry detection |
| Onboarding commands | `commands/onboarding.rs` | 8 new Tauri IPC commands |
| `WelcomeStep` | `components/onboarding/WelcomeStep.tsx` | Welcome screen |
| `EmailStep` | `components/onboarding/EmailStep.tsx` | Email input |
| `OtpStep` | `components/onboarding/OtpStep.tsx` | OTP verification |
| `SessionExpiredModal` | `components/onboarding/SessionExpiredModal.tsx` | Re-auth overlay |
| `useOnboarding` hook | `hooks/useOnboarding.ts` | Onboarding state management |
| `useSessionHealth` hook | `hooks/useSessionHealth.ts` | Session health polling |
| `onboardingStore` | `stores/onboardingStore.ts` | Zustand store |
| `user_profile` table | `db/migrations/` | SQLite table for user data |

### 10.3 What Doesn't Change

- Agent registration and management (unchanged)
- Spending policies and global policy (unchanged)
- Transaction processing (unchanged)
- REST API, MCP server, Unix socket (unchanged)
- Notification system (unchanged)
- All existing database tables (unchanged)

### 10.4 Startup Flow (Updated)

```
App launches
    |
    v
Initialize database + run migrations
    |
    v
Read onboarding_state from app_config
    |
    +-- FRESH or ONBOARDING or WALLET_LINKING or OTP_VERIFICATION
    |       |
    |       v
    |   Show Onboarding page (frontend handles sub-steps)
    |   CLI may or may not be available yet -- that's OK
    |
    +-- ACTIVE
    |       |
    |       v
    |   Check CLI health (awal auth status)
    |       |
    |       +-- authenticated --> Show Dashboard, spawn health check loop
    |       |
    |       +-- not authenticated --> Set SESSION_EXPIRED, show Dashboard + overlay
    |
    +-- SESSION_EXPIRED
            |
            v
        Show Dashboard + re-auth overlay
        CLI is available but session is dead
```

### 10.5 Phase Placement

This document defines work that belongs in **Phase 1a: Plumbing**. The onboarding flow is the very first thing a user encounters. Without it, none of the other phases matter because the user cannot get to the dashboard.

Updated Phase 1a task list (additions in bold):

| Task | Module | Details |
|---|---|---|
| **user_profile table + migration** | `db/schema.rs` | New table, new app_config entries |
| **UserService** | `core/user_service.rs` | Profile CRUD, onboarding state management |
| **SessionManager** | `core/session_manager.rs` | Health checks, session expiry detection |
| **Onboarding commands** | `commands/onboarding.rs` | 8 new IPC commands |
| **Onboarding UI** | `components/onboarding/*` | WelcomeStep, EmailStep, OtpStep, SessionExpiredModal |
| **useOnboarding hook** | `hooks/useOnboarding.ts` | State machine in frontend |
| **useSessionHealth hook** | `hooks/useSessionHealth.ts` | Background polling |
| **App.tsx routing update** | `src/App.tsx` | Route based on onboarding state |
| **Startup flow update** | `main.rs` | Check onboarding state before CLI health check |

These tasks should be implemented **before** the existing "Auth flow (email OTP)" task, as the onboarding flow subsumes it. The `auth_login` and `auth_verify` commands from the original architecture are now thin wrappers around the `initiate_wallet_link` and `verify_wallet_link` commands.

---

## Appendix A: CLI Output Parsing

### `awal auth login` Output

Expected stdout (based on CDP documentation):

```json
{
  "flowId": "abc123-def456-ghi789",
  "message": "Verification code sent to user@example.com"
}
```

Parser extracts `flowId` from JSON. Falls back to regex parsing if output is not valid JSON (e.g., `flowId: abc123...`).

### `awal auth verify` Output

Expected stdout:

```json
{
  "success": true,
  "message": "Authentication successful"
}
```

On failure:

```json
{
  "success": false,
  "error": "Invalid OTP"
}
```

### `awal auth status` Output

Expected stdout:

```json
{
  "authenticated": true,
  "email": "user@example.com"
}
```

Or:

```json
{
  "authenticated": false
}
```

> **Note:** These output formats are inferred from typical CLI behavior and may need adjustment once we test against the real `awal` CLI. The parser module (`cli/parser.rs`) should be designed to handle variations gracefully.

---

## Appendix B: Open Questions

1. **What is the exact session TTL for awal?** Documentation does not specify. The 5-minute health check interval is a conservative default. If sessions last hours, we could reduce check frequency.

2. **Can we programmatically detect awal installation?** Currently we try to spawn the process and check for errors. A more robust approach might be to check `which npx` and `npx awal --version`.

3. **What happens if the user authenticates with a different email?** For v1, we overwrite the stored email and wallet address. The previous wallet still exists on-chain but is no longer tracked by this app instance. A future version could support multiple wallets.

4. **Should we support the `awal auth verify` command with the email parameter?** The CDP docs show `awal auth verify <flowId> <otp>` but some versions may require the email too. We should test and handle both.

5. **Coinbase OAuth integration (future):** Once OAuth partner approval is obtained, the email step could be replaced with "Sign in with Coinbase" which auto-fills the email. See `docs/investigations/coinbase-oauth-research.md` for details. This is a Phase 4+ enhancement.
