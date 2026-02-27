use crate::config::AppConfig;

// TODO: implement in Phase 1a
// CoreServices will hold all sub-services as described in the architecture:
// - db: Arc<Database>
// - cli: Arc<dyn CliExecutable>
// - agent_registry: AgentRegistry
// - spending_policy: SpendingPolicyEngine
// - global_policy: GlobalPolicyEngine
// - tx_processor: TransactionProcessor
// - auth_service: AuthService
// - wallet_service: WalletService
// - approval_manager: ApprovalManager
// - notification_manager: NotificationManager
// - event_bus: EventBus
// - balance_cache: BalanceCache
// - invitation_manager: InvitationManager
// - config: AppConfig

pub struct CoreServices {
    pub config: AppConfig,
}
