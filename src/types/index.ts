// TypeScript types matching Rust models in src-tauri/src/db/models.rs

export type AgentStatus = "pending" | "active" | "suspended" | "revoked";
export type TxStatus = "pending" | "approved" | "awaiting_approval" | "executing" | "confirmed" | "failed" | "denied";
export type TxType = "send" | "receive" | "earn";
export type ApprovalRequestType = "transaction" | "limit_increase" | "registration";
export type ApprovalStatus = "pending" | "approved" | "denied" | "expired";

export interface Agent {
  id: string;
  name: string;
  description: string;
  purpose: string;
  agent_type: string;
  capabilities: string[];
  status: AgentStatus;
  api_token_hash: string | null;
  token_prefix: string | null;
  balance_visible: boolean;
  invitation_code: string | null;
  created_at: number;
  updated_at: number;
  last_active_at: number | null;
  metadata: string;
}

export interface SpendingPolicy {
  agent_id: string;
  per_tx_max: string;
  daily_cap: string;
  weekly_cap: string;
  monthly_cap: string;
  auto_approve_max: string;
  allowlist: string[];
  updated_at: number;
}

export interface GlobalPolicy {
  id: string;
  daily_cap: string;
  weekly_cap: string;
  monthly_cap: string;
  min_reserve_balance: string;
  kill_switch_active: boolean;
  kill_switch_reason: string;
  updated_at: number;
}

export interface Transaction {
  id: string;
  agent_id: string | null;
  tx_type: TxType;
  amount: string;
  asset: string;
  recipient: string | null;
  sender: string | null;
  chain_tx_hash: string | null;
  status: TxStatus;
  category: string;
  memo: string;
  description: string;
  service_name: string;
  service_url: string;
  reason: string;
  webhook_url: string | null;
  error_message: string | null;
  period_daily: string;
  period_weekly: string;
  period_monthly: string;
  created_at: number;
  updated_at: number;
}

export interface ApprovalRequest {
  id: string;
  agent_id: string;
  request_type: ApprovalRequestType;
  payload: string;
  status: ApprovalStatus;
  tx_id: string | null;
  expires_at: number;
  created_at: number;
  resolved_at: number | null;
  resolved_by: string | null;
}

export interface InvitationCode {
  code: string;
  created_at: number;
  expires_at: number | null;
  used_by: string | null;
  used_at: number | null;
  max_uses: number;
  use_count: number;
  label: string;
}

export interface TokenDelivery {
  agent_id: string;
  encrypted_token: string;
  created_at: number;
  expires_at: number;
  delivered: boolean;
}

export interface NotificationPreferences {
  id: string;
  enabled: boolean;
  on_all_tx: boolean;
  on_large_tx: boolean;
  large_tx_threshold: string;
  on_errors: boolean;
  on_limit_requests: boolean;
  on_agent_registration: boolean;
}

export interface SpendingLedger {
  agent_id: string;
  period: string;
  total: string;
  tx_count: number;
  updated_at: number;
}

export interface AgentBudgetSummary {
  agent_id: string;
  agent_name: string;
  daily_spent: string;
  daily_cap: string;
  weekly_spent: string;
  weekly_cap: string;
  monthly_spent: string;
  monthly_cap: string;
}

export interface GlobalBudgetSummary {
  daily_spent: string;
  daily_cap: string;
  weekly_spent: string;
  weekly_cap: string;
  monthly_spent: string;
  monthly_cap: string;
  kill_switch_active: boolean;
}

export interface AssetBalance {
  raw: string;
  formatted: string;
  decimals: number;
}

export interface BalanceResponse {
  balance: string | null;
  asset: string | null;
  balances: Record<string, AssetBalance> | null;
  balance_visible: boolean;
  cached: boolean;
}

export interface AddressResponse {
  address: string;
}

export interface AuthStatusResponse {
  authenticated: boolean;
  email?: string;
}

// ── Provisioning Types ──

export type ToolId =
  | "claude_code"
  | "claude_desktop"
  | "cursor"
  | "windsurf"
  | "codex"
  | "continue_dev"
  | "cline"
  | "aider"
  | "copilot";

export type DetectionMethod =
  | "config_directory"
  | "config_file"
  | "binary_in_path"
  | "application_bundle"
  | "vs_code_extension"
  | "process_running";

export type ConfigFormat =
  | "json"
  | "json_with_servers_key"
  | "toml"
  | "yaml"
  | "markdown"
  | "markdown_with_frontmatter"
  | "standalone_file";

export type ConfigPurpose = "mcp_server" | "system_instructions" | "skill" | "convention_file";

export type FileChangeType =
  | "create_file"
  | "merge_json_key"
  | "append_toml_section"
  | "append_markdown_section"
  | "merge_yaml_entry";

export type ToolStatus = "unknown" | "detected" | "provisioned" | "needs_update" | "removed" | "excluded";

export type VerificationStatus = "intact" | "modified" | "missing" | "corrupted";

export type RollbackStrategy = "surgical_removal" | "diff_based" | "full_restore" | "already_clean";

export interface ConfigFileInfo {
  path: string;
  resolved_path: string;
  exists: boolean;
  writable: boolean;
  format: ConfigFormat;
  purpose: ConfigPurpose;
  is_symlink: boolean;
}

export interface DetectionResult {
  tool: ToolId;
  detected: boolean;
  methods: DetectionMethod[];
  version: string | null;
  config_paths: ConfigFileInfo[];
}

export interface FileChange {
  path: string;
  change_type: FileChangeType;
  description: string;
  diff: string | null;
}

export interface ProvisionPreview {
  tool: ToolId;
  changes: FileChange[];
}

export interface ModifiedFile {
  path: string;
  change_type: FileChangeType;
  backup_path: string | null;
  sha256_before: string | null;
  sha256_after: string;
  created_new: boolean;
}

export interface ProvisionResult {
  tool: ToolId;
  success: boolean;
  files_modified: ModifiedFile[];
  error: string | null;
  needs_restart: boolean;
}

export interface UnprovisionResult {
  tool: ToolId;
  success: boolean;
  files_restored: string[];
  files_deleted: string[];
  error: string | null;
  strategy_used: RollbackStrategy;
}

export interface ToolProvisioningState {
  status: ToolStatus;
  provisioned_at: string | null;
  last_verified: string | null;
  provisioned_version: string | null;
  tool_version: string | null;
  removal_count: number;
  respect_removal: boolean;
  files_managed: string[];
}

export interface ProvisioningState {
  schema_version: number;
  machine_id: string;
  tally_version: string;
  tools: Record<string, ToolProvisioningState>;
  excluded_tools: ToolId[];
  last_scan: string | null;
}

export interface McpInjectionConfig {
  server_command: string;
  server_args: string[];
  env: Record<string, string>;
  tally_version: string;
  provisioned_at: string;
}

export interface VerificationDetail {
  path: string;
  expected_hash: string | null;
  actual_hash: string | null;
  status: VerificationStatus;
}

export interface VerificationResult {
  tool: ToolId;
  status: VerificationStatus;
  details: VerificationDetail[];
}
