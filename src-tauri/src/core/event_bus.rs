use serde::Serialize;
use tokio::sync::broadcast;

// -------------------------------------------------------------------------
// Event types
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    TransactionConfirmed {
        tx_id: String,
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
        agent_id: String,
        amount: String,
    },
    ApprovalResolved {
        approval_id: String,
        status: String,
    },
    AgentRegistered {
        agent_id: String,
        name: String,
    },
    AgentStatusChanged {
        agent_id: String,
        status: String,
    },
    LimitChangeRequested {
        agent_id: String,
        approval_id: String,
    },
    KillSwitchToggled {
        active: bool,
    },
}

// -------------------------------------------------------------------------
// EventBus
// -------------------------------------------------------------------------

pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    /// Publish an event. Returns the number of active receivers that will
    /// receive the event, or 0 if there are no subscribers.
    pub fn publish(&self, event: AppEvent) -> usize {
        self.sender.send(event).unwrap_or(0)
    }

    /// Return a clone-able sender handle for use in other subsystems.
    pub fn sender(&self) -> broadcast::Sender<AppEvent> {
        self.sender.clone()
    }
}

// -------------------------------------------------------------------------
// Tauri bridge
// -------------------------------------------------------------------------

/// Spawns a background task that forwards AppEvents to Tauri's event system.
/// Call this once during app setup.
#[cfg(not(test))]
pub fn bridge_to_tauri(
    app_handle: tauri::AppHandle,
    mut rx: broadcast::Receiver<AppEvent>,
) {
    use tauri::Emitter;

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let event_name = match &event {
                        AppEvent::TransactionConfirmed { .. } => "transaction:confirmed",
                        AppEvent::TransactionDenied { .. } => "transaction:denied",
                        AppEvent::TransactionFailed { .. } => "transaction:failed",
                        AppEvent::ApprovalRequired { .. } => "approval:required",
                        AppEvent::ApprovalResolved { .. } => "approval:resolved",
                        AppEvent::AgentRegistered { .. } => "agent:registered",
                        AppEvent::AgentStatusChanged { .. } => "agent:status_changed",
                        AppEvent::LimitChangeRequested { .. } => "limit:change_requested",
                        AppEvent::KillSwitchToggled { .. } => "killswitch:toggled",
                    };
                    let _ = app_handle.emit(event_name, &event);
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Event bus lagged by {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Event bus closed");
                    break;
                }
            }
        }
    });
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_subscribe_and_publish() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(AppEvent::TransactionConfirmed {
            tx_id: "tx-001".to_string(),
        });

        let event = rx.recv().await.expect("should receive event");
        match event {
            AppEvent::TransactionConfirmed { tx_id } => {
                assert_eq!(tx_id, "tx-001");
            }
            other => panic!("unexpected event variant: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        let mut rx3 = bus.subscribe();

        bus.publish(AppEvent::AgentRegistered {
            agent_id: "agent-1".to_string(),
            name: "TestAgent".to_string(),
        });

        for rx in [&mut rx1, &mut rx2, &mut rx3] {
            let event = rx.recv().await.expect("subscriber should receive event");
            match event {
                AppEvent::AgentRegistered { agent_id, name } => {
                    assert_eq!(agent_id, "agent-1");
                    assert_eq!(name, "TestAgent");
                }
                other => panic!("unexpected event variant: {:?}", other),
            }
        }
    }

    #[tokio::test]
    async fn test_event_bus_publish_returns_subscriber_count() {
        let bus = EventBus::new(16);
        let _rx1 = bus.subscribe();
        let _rx2 = bus.subscribe();

        let count = bus.publish(AppEvent::KillSwitchToggled { active: true });
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_event_bus_no_subscribers_returns_zero() {
        let bus = EventBus::new(16);
        // No subscribers — the initial receiver from broadcast::channel is dropped
        // in EventBus::new, so there are zero active receivers.

        let count = bus.publish(AppEvent::KillSwitchToggled { active: false });
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_event_bus_serialization() {
        let event = AppEvent::TransactionDenied {
            tx_id: "tx-999".to_string(),
            reason: "over limit".to_string(),
        };

        let json = serde_json::to_value(&event).expect("should serialize");
        assert_eq!(json["type"], "TransactionDenied");
        assert_eq!(json["data"]["tx_id"], "tx-999");
        assert_eq!(json["data"]["reason"], "over limit");

        // Also verify KillSwitchToggled serialization (bool data)
        let event2 = AppEvent::KillSwitchToggled { active: true };
        let json2 = serde_json::to_value(&event2).expect("should serialize");
        assert_eq!(json2["type"], "KillSwitchToggled");
        assert_eq!(json2["data"]["active"], true);
    }
}
