-- 001_initial.sql

-- Application configuration (key-value)
CREATE TABLE IF NOT EXISTS app_config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Agent registry
CREATE TABLE IF NOT EXISTS agents (
    id                TEXT PRIMARY KEY,           -- UUID v4
    name              TEXT NOT NULL,
    description       TEXT DEFAULT '',
    purpose           TEXT DEFAULT '',            -- what the agent is built for
    agent_type        TEXT DEFAULT '',            -- e.g., "coding_assistant", "research"
    capabilities      TEXT DEFAULT '[]',          -- JSON array: ["send", "receive"]
    status            TEXT NOT NULL DEFAULT 'pending',  -- pending | active | suspended | revoked
    api_token_hash    TEXT,                       -- argon2 hash of the agent's bearer token
    token_prefix      TEXT,                       -- first 8 chars for display (e.g., "anb_a3f8...")
    balance_visible   INTEGER NOT NULL DEFAULT 1, -- whether agent can see wallet balance
    invitation_code   TEXT,                       -- the invitation code used to register
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL,
    last_active_at    INTEGER,
    metadata          TEXT DEFAULT '{}'           -- JSON blob for extensible data
);

CREATE INDEX idx_agents_status ON agents(status);

-- Spending policies (one per agent)
CREATE TABLE IF NOT EXISTS spending_policies (
    agent_id         TEXT PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    per_tx_max       TEXT NOT NULL DEFAULT '0',     -- Decimal string in USDC
    daily_cap        TEXT NOT NULL DEFAULT '0',
    weekly_cap       TEXT NOT NULL DEFAULT '0',
    monthly_cap      TEXT NOT NULL DEFAULT '0',
    auto_approve_max TEXT NOT NULL DEFAULT '0',     -- Below this = auto-approve
    allowlist        TEXT DEFAULT '[]',             -- JSON array of allowed addresses/domains
    updated_at       INTEGER NOT NULL
);

-- Global policy -- wallet-level controls above all agent policies
CREATE TABLE IF NOT EXISTS global_policy (
    id                   TEXT PRIMARY KEY DEFAULT 'default',
    daily_cap            TEXT NOT NULL DEFAULT '0',       -- Global daily spending cap across all agents
    weekly_cap           TEXT NOT NULL DEFAULT '0',
    monthly_cap          TEXT NOT NULL DEFAULT '0',
    min_reserve_balance  TEXT NOT NULL DEFAULT '0',       -- Refuse txs that would drop below this
    kill_switch_active   INTEGER NOT NULL DEFAULT 0,      -- 1 = all agent operations suspended
    kill_switch_reason   TEXT DEFAULT '',
    updated_at           INTEGER NOT NULL
);

-- Global spending ledger -- aggregate across all agents
CREATE TABLE IF NOT EXISTS global_spending_ledger (
    period     TEXT PRIMARY KEY,               -- 'daily:2026-02-27' | 'weekly:2026-W09' | 'monthly:2026-02'
    total      TEXT NOT NULL DEFAULT '0',
    tx_count   INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL
);

-- Transactions
CREATE TABLE IF NOT EXISTS transactions (
    id              TEXT PRIMARY KEY,             -- UUID v4
    agent_id        TEXT REFERENCES agents(id),   -- NULL for user-initiated txs
    tx_type         TEXT NOT NULL,                -- send | receive | earn
    amount          TEXT NOT NULL,                -- Decimal string in USDC
    asset           TEXT NOT NULL DEFAULT 'USDC',
    recipient       TEXT,                         -- Address or service identifier
    sender          TEXT,                         -- For receive txs
    chain_tx_hash   TEXT,                         -- On-chain tx hash when available
    status          TEXT NOT NULL DEFAULT 'pending', -- pending | approved | executing | confirmed | failed | denied
    category        TEXT DEFAULT 'uncategorized',
    memo            TEXT DEFAULT '',
    description     TEXT DEFAULT '',              -- detailed description from agent
    service_name    TEXT DEFAULT '',              -- what service this payment is for
    service_url     TEXT DEFAULT '',              -- URL of the service
    reason          TEXT DEFAULT '',              -- why the agent needs this payment
    webhook_url     TEXT,                         -- optional callback URL for status updates
    error_message   TEXT,
    period_daily    TEXT,                         -- UTC period key at creation time
    period_weekly   TEXT,                         -- UTC period key at creation time
    period_monthly  TEXT,                         -- UTC period key at creation time
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

CREATE INDEX idx_tx_agent ON transactions(agent_id);
CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_created ON transactions(created_at);
CREATE INDEX idx_tx_type ON transactions(tx_type);

-- Approval requests
CREATE TABLE IF NOT EXISTS approval_requests (
    id           TEXT PRIMARY KEY,              -- UUID v4
    agent_id     TEXT NOT NULL REFERENCES agents(id),
    request_type TEXT NOT NULL,                 -- transaction | limit_increase | registration
    payload      TEXT NOT NULL,                 -- JSON: the full request details
    status       TEXT NOT NULL DEFAULT 'pending', -- pending | approved | denied | expired
    tx_id        TEXT REFERENCES transactions(id), -- Links to tx if type=transaction
    expires_at   INTEGER NOT NULL,              -- auto-expire after this timestamp
    created_at   INTEGER NOT NULL,
    resolved_at  INTEGER,
    resolved_by  TEXT                           -- 'user' or 'auto'
);

CREATE INDEX idx_approval_status ON approval_requests(status);
CREATE INDEX idx_approval_agent ON approval_requests(agent_id);
CREATE INDEX idx_approval_expires ON approval_requests(expires_at);

-- Invitation codes -- user-generated codes for agent registration
CREATE TABLE IF NOT EXISTS invitation_codes (
    code         TEXT PRIMARY KEY,              -- Short alphanumeric code
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER,                       -- Optional expiry
    used_by      TEXT REFERENCES agents(id),    -- NULL until used
    used_at      INTEGER,
    max_uses     INTEGER NOT NULL DEFAULT 1,    -- Usually 1 (one-time use)
    use_count    INTEGER NOT NULL DEFAULT 0,
    label        TEXT DEFAULT ''                -- User-facing label ("for Claude Code")
);

-- Token delivery cache -- short-lived encrypted token storage
CREATE TABLE IF NOT EXISTS token_delivery (
    agent_id     TEXT PRIMARY KEY REFERENCES agents(id),
    encrypted_token TEXT NOT NULL,              -- AES-encrypted bearer token
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER NOT NULL,              -- created_at + 300 (5 minutes)
    delivered    INTEGER NOT NULL DEFAULT 0     -- 1 after first retrieval (then deleted)
);

-- Notification preferences
CREATE TABLE IF NOT EXISTS notification_preferences (
    id         TEXT PRIMARY KEY DEFAULT 'default',
    enabled    INTEGER NOT NULL DEFAULT 1,
    on_all_tx  INTEGER NOT NULL DEFAULT 0,
    on_large_tx INTEGER NOT NULL DEFAULT 1,
    large_tx_threshold TEXT NOT NULL DEFAULT '10.00',
    on_errors  INTEGER NOT NULL DEFAULT 1,
    on_limit_requests INTEGER NOT NULL DEFAULT 1,
    on_agent_registration INTEGER NOT NULL DEFAULT 1
);

-- Spending ledger (rolling aggregates for fast policy checks)
-- All reads and writes for a given agent MUST use BEGIN EXCLUSIVE transactions.
CREATE TABLE IF NOT EXISTS spending_ledger (
    agent_id   TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    period     TEXT NOT NULL,                  -- 'daily:2026-02-27' | 'weekly:2026-W09' | 'monthly:2026-02'
    total      TEXT NOT NULL DEFAULT '0',      -- Running total for this period
    tx_count   INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (agent_id, period)
);
