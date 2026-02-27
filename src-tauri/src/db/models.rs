use serde::{Deserialize, Serialize};

// -------------------------------------------------------------------------
// Enums
// -------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Pending,
    Active,
    Suspended,
    Revoked,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Pending => write!(f, "pending"),
            AgentStatus::Active => write!(f, "active"),
            AgentStatus::Suspended => write!(f, "suspended"),
            AgentStatus::Revoked => write!(f, "revoked"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxStatus {
    Pending,
    Approved,
    Executing,
    Confirmed,
    Failed,
    Denied,
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxStatus::Pending => write!(f, "pending"),
            TxStatus::Approved => write!(f, "approved"),
            TxStatus::Executing => write!(f, "executing"),
            TxStatus::Confirmed => write!(f, "confirmed"),
            TxStatus::Failed => write!(f, "failed"),
            TxStatus::Denied => write!(f, "denied"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Send,
    Receive,
    Earn,
}

impl std::fmt::Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxType::Send => write!(f, "send"),
            TxType::Receive => write!(f, "receive"),
            TxType::Earn => write!(f, "earn"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalRequestType {
    Transaction,
    LimitIncrease,
    Registration,
}

impl std::fmt::Display for ApprovalRequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalRequestType::Transaction => write!(f, "transaction"),
            ApprovalRequestType::LimitIncrease => write!(f, "limit_increase"),
            ApprovalRequestType::Registration => write!(f, "registration"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::Pending => write!(f, "pending"),
            ApprovalStatus::Approved => write!(f, "approved"),
            ApprovalStatus::Denied => write!(f, "denied"),
            ApprovalStatus::Expired => write!(f, "expired"),
        }
    }
}

// -------------------------------------------------------------------------
// Model Structs
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub purpose: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub api_token_hash: Option<String>,
    pub token_prefix: Option<String>,
    pub balance_visible: bool,
    pub invitation_code: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_active_at: Option<i64>,
    pub metadata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingPolicy {
    pub agent_id: String,
    pub per_tx_max: String,
    pub daily_cap: String,
    pub weekly_cap: String,
    pub monthly_cap: String,
    pub auto_approve_max: String,
    pub allowlist: Vec<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPolicy {
    pub id: String,
    pub daily_cap: String,
    pub weekly_cap: String,
    pub monthly_cap: String,
    pub min_reserve_balance: String,
    pub kill_switch_active: bool,
    pub kill_switch_reason: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSpendingLedger {
    pub period: String,
    pub total: String,
    pub tx_count: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub agent_id: Option<String>,
    pub tx_type: TxType,
    pub amount: String,
    pub asset: String,
    pub recipient: Option<String>,
    pub sender: Option<String>,
    pub chain_tx_hash: Option<String>,
    pub status: TxStatus,
    pub category: String,
    pub memo: String,
    pub description: String,
    pub service_name: String,
    pub service_url: String,
    pub reason: String,
    pub webhook_url: Option<String>,
    pub error_message: Option<String>,
    pub period_daily: String,
    pub period_weekly: String,
    pub period_monthly: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub agent_id: String,
    pub request_type: ApprovalRequestType,
    pub payload: String,
    pub status: ApprovalStatus,
    pub tx_id: Option<String>,
    pub expires_at: i64,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationCode {
    pub code: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub used_by: Option<String>,
    pub used_at: Option<i64>,
    pub max_uses: i32,
    pub use_count: i32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDelivery {
    pub agent_id: String,
    pub encrypted_token: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub delivered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub id: String,
    pub enabled: bool,
    pub on_all_tx: bool,
    pub on_large_tx: bool,
    pub large_tx_threshold: String,
    pub on_errors: bool,
    pub on_limit_requests: bool,
    pub on_agent_registration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingLedger {
    pub agent_id: String,
    pub period: String,
    pub total: String,
    pub tx_count: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigEntry {
    pub key: String,
    pub value: String,
}
