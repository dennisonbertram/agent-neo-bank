use std::sync::Arc;

use crate::db::models::NotificationPreferences;
use crate::db::queries::{get_notification_preferences, upsert_notification_preferences};
use crate::db::schema::Database;
use crate::error::AppError;

/// Types of notifications the system can emit.
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    TransactionConfirmed {
        tx_id: String,
        amount: String,
        recipient: String,
    },
    TransactionDenied {
        tx_id: String,
        reason: String,
    },
    TransactionFailed {
        tx_id: String,
        error: String,
    },
    ApprovalRequired {
        approval_id: String,
        agent_name: String,
        amount: String,
    },
    AgentRegistered {
        agent_id: String,
        name: String,
    },
    LimitChangeRequested {
        agent_id: String,
        agent_name: String,
    },
    Error {
        message: String,
    },
}

pub struct NotificationService {
    db: Arc<Database>,
}

impl NotificationService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Check if a notification should be sent based on user preferences.
    /// Returns `Some((title, body))` if the notification should be sent, `None` otherwise.
    pub fn should_notify(
        &self,
        notification: &NotificationType,
    ) -> Result<Option<(String, String)>, AppError> {
        let prefs = self.get_preferences()?;

        if !prefs.enabled {
            return Ok(None);
        }

        match notification {
            NotificationType::TransactionConfirmed {
                amount, recipient, ..
            } => {
                let short_recipient = &recipient[..8.min(recipient.len())];
                if prefs.on_all_tx {
                    Ok(Some((
                        "Transaction Confirmed".to_string(),
                        format!("Sent {} to {}", amount, short_recipient),
                    )))
                } else if prefs.on_large_tx {
                    let threshold: rust_decimal::Decimal =
                        prefs.large_tx_threshold.parse().unwrap_or_default();
                    let amt: rust_decimal::Decimal = amount.parse().unwrap_or_default();
                    if amt >= threshold {
                        Ok(Some((
                            "Large Transaction Confirmed".to_string(),
                            format!("Sent {} to {}", amount, short_recipient),
                        )))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NotificationType::TransactionDenied { reason, .. } => {
                if prefs.on_all_tx || prefs.on_errors {
                    Ok(Some(("Transaction Denied".to_string(), reason.clone())))
                } else {
                    Ok(None)
                }
            }
            NotificationType::TransactionFailed { error, .. } => {
                if prefs.on_errors {
                    Ok(Some(("Transaction Failed".to_string(), error.clone())))
                } else {
                    Ok(None)
                }
            }
            NotificationType::ApprovalRequired {
                agent_name,
                amount,
                ..
            } => {
                // Always notify on approval required (it's critical)
                Ok(Some((
                    "Approval Required".to_string(),
                    format!("{} wants to send {}", agent_name, amount),
                )))
            }
            NotificationType::AgentRegistered { name, .. } => {
                if prefs.on_agent_registration {
                    Ok(Some((
                        "New Agent Registered".to_string(),
                        format!("{} has registered", name),
                    )))
                } else {
                    Ok(None)
                }
            }
            NotificationType::LimitChangeRequested { agent_name, .. } => {
                if prefs.on_limit_requests {
                    Ok(Some((
                        "Limit Change Requested".to_string(),
                        format!("{} requested a limit increase", agent_name),
                    )))
                } else {
                    Ok(None)
                }
            }
            NotificationType::Error { message } => {
                if prefs.on_errors {
                    Ok(Some(("Error".to_string(), message.clone())))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Get current notification preferences from DB.
    /// Returns defaults if not found.
    pub fn get_preferences(&self) -> Result<NotificationPreferences, AppError> {
        match get_notification_preferences(&self.db)? {
            Some(prefs) => Ok(prefs),
            None => Ok(Self::default_preferences()),
        }
    }

    /// Update notification preferences.
    pub fn update_preferences(&self, prefs: &NotificationPreferences) -> Result<(), AppError> {
        upsert_notification_preferences(&self.db, prefs)
    }

    /// Get default preferences (everything enabled).
    pub fn default_preferences() -> NotificationPreferences {
        NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: true,
            on_large_tx: false,
            large_tx_threshold: "100".to_string(),
            on_errors: true,
            on_limit_requests: true,
            on_agent_registration: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_db;

    fn make_service() -> NotificationService {
        let db = setup_test_db();
        NotificationService::new(db)
    }

    fn make_service_with_prefs(prefs: NotificationPreferences) -> NotificationService {
        let db = setup_test_db();
        upsert_notification_preferences(&db, &prefs).unwrap();
        NotificationService::new(db)
    }

    #[test]
    fn test_notification_dispatch_when_enabled() {
        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: true,
            on_large_tx: false,
            large_tx_threshold: "100".to_string(),
            on_errors: true,
            on_limit_requests: true,
            on_agent_registration: true,
        };
        let svc = make_service_with_prefs(prefs);

        let notif = NotificationType::TransactionConfirmed {
            tx_id: "tx-1".to_string(),
            amount: "50".to_string(),
            recipient: "0xabcdef1234567890".to_string(),
        };

        let result = svc.should_notify(&notif).unwrap();
        assert!(result.is_some(), "Should notify when on_all_tx is true");
        let (title, body) = result.unwrap();
        assert_eq!(title, "Transaction Confirmed");
        assert!(body.contains("50"));
        assert!(body.contains("0xabcdef"));
    }

    #[test]
    fn test_notification_dispatch_when_disabled() {
        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: false,
            on_all_tx: true,
            on_large_tx: false,
            large_tx_threshold: "100".to_string(),
            on_errors: true,
            on_limit_requests: true,
            on_agent_registration: true,
        };
        let svc = make_service_with_prefs(prefs);

        let notif = NotificationType::TransactionConfirmed {
            tx_id: "tx-1".to_string(),
            amount: "50".to_string(),
            recipient: "0xabcdef1234567890".to_string(),
        };

        let result = svc.should_notify(&notif).unwrap();
        assert!(result.is_none(), "Should not notify when enabled is false");
    }

    #[test]
    fn test_notification_preference_filtering_large_tx() {
        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: false,
            on_large_tx: true,
            large_tx_threshold: "100".to_string(),
            on_errors: true,
            on_limit_requests: true,
            on_agent_registration: true,
        };
        let svc = make_service_with_prefs(prefs);

        // Small tx should not notify
        let small_tx = NotificationType::TransactionConfirmed {
            tx_id: "tx-small".to_string(),
            amount: "50".to_string(),
            recipient: "0xabcdef1234567890".to_string(),
        };
        let result = svc.should_notify(&small_tx).unwrap();
        assert!(result.is_none(), "Small tx should not trigger notification");

        // Large tx should notify
        let large_tx = NotificationType::TransactionConfirmed {
            tx_id: "tx-large".to_string(),
            amount: "200".to_string(),
            recipient: "0xabcdef1234567890".to_string(),
        };
        let result = svc.should_notify(&large_tx).unwrap();
        assert!(result.is_some(), "Large tx should trigger notification");
        let (title, _) = result.unwrap();
        assert_eq!(title, "Large Transaction Confirmed");

        // Tx exactly at threshold should notify
        let exact_tx = NotificationType::TransactionConfirmed {
            tx_id: "tx-exact".to_string(),
            amount: "100".to_string(),
            recipient: "0xabcdef1234567890".to_string(),
        };
        let result = svc.should_notify(&exact_tx).unwrap();
        assert!(
            result.is_some(),
            "Tx at threshold should trigger notification"
        );
    }

    #[test]
    fn test_notification_approval_always_notifies() {
        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: false,
            on_large_tx: false,
            large_tx_threshold: "100".to_string(),
            on_errors: false,
            on_limit_requests: false,
            on_agent_registration: false,
        };
        let svc = make_service_with_prefs(prefs);

        let notif = NotificationType::ApprovalRequired {
            approval_id: "appr-1".to_string(),
            agent_name: "TestBot".to_string(),
            amount: "500".to_string(),
        };

        let result = svc.should_notify(&notif).unwrap();
        assert!(
            result.is_some(),
            "ApprovalRequired should always notify when enabled"
        );
        let (title, body) = result.unwrap();
        assert_eq!(title, "Approval Required");
        assert!(body.contains("TestBot"));
        assert!(body.contains("500"));
    }

    #[test]
    fn test_notification_agent_registration_filtering() {
        // on_agent_registration = false
        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: true,
            on_large_tx: false,
            large_tx_threshold: "100".to_string(),
            on_errors: true,
            on_limit_requests: true,
            on_agent_registration: false,
        };
        let svc = make_service_with_prefs(prefs);

        let notif = NotificationType::AgentRegistered {
            agent_id: "agent-1".to_string(),
            name: "NewBot".to_string(),
        };

        let result = svc.should_notify(&notif).unwrap();
        assert!(
            result.is_none(),
            "AgentRegistered should not notify when on_agent_registration is false"
        );
    }

    #[test]
    fn test_notification_error_filtering() {
        // on_errors = false
        let prefs_off = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: false,
            on_large_tx: false,
            large_tx_threshold: "100".to_string(),
            on_errors: false,
            on_limit_requests: true,
            on_agent_registration: true,
        };
        let svc = make_service_with_prefs(prefs_off);

        let notif = NotificationType::Error {
            message: "Something went wrong".to_string(),
        };
        let result = svc.should_notify(&notif).unwrap();
        assert!(
            result.is_none(),
            "Error should not notify when on_errors is false"
        );

        // on_errors = true
        let prefs_on = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: false,
            on_large_tx: false,
            large_tx_threshold: "100".to_string(),
            on_errors: true,
            on_limit_requests: true,
            on_agent_registration: true,
        };
        let svc2 = make_service_with_prefs(prefs_on);

        let result = svc2.should_notify(&notif).unwrap();
        assert!(
            result.is_some(),
            "Error should notify when on_errors is true"
        );
        let (title, body) = result.unwrap();
        assert_eq!(title, "Error");
        assert_eq!(body, "Something went wrong");
    }

    #[test]
    fn test_notification_preferences_persistence() {
        let svc = make_service();

        let prefs = NotificationPreferences {
            id: "default".to_string(),
            enabled: true,
            on_all_tx: false,
            on_large_tx: true,
            large_tx_threshold: "250".to_string(),
            on_errors: false,
            on_limit_requests: true,
            on_agent_registration: false,
        };

        svc.update_preferences(&prefs).unwrap();
        let fetched = svc.get_preferences().unwrap();

        assert_eq!(fetched.id, "default");
        assert!(fetched.enabled);
        assert!(!fetched.on_all_tx);
        assert!(fetched.on_large_tx);
        assert_eq!(fetched.large_tx_threshold, "250");
        assert!(!fetched.on_errors);
        assert!(fetched.on_limit_requests);
        assert!(!fetched.on_agent_registration);
    }

    #[test]
    fn test_notification_default_preferences() {
        let defaults = NotificationService::default_preferences();

        assert_eq!(defaults.id, "default");
        assert!(defaults.enabled);
        assert!(defaults.on_all_tx);
        assert!(!defaults.on_large_tx);
        assert_eq!(defaults.large_tx_threshold, "100");
        assert!(defaults.on_errors);
        assert!(defaults.on_limit_requests);
        assert!(defaults.on_agent_registration);
    }
}
