use tauri::State;

use crate::db::models::Agent;
use crate::db::queries;
use crate::error::AppError;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn list_agents(
    state: State<'_, AppState>,
) -> Result<Vec<Agent>, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || queries::list_all_agents(&db))
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Agent, AgentStatus};
    use crate::db::queries::insert_agent;
    use crate::db::schema::Database;

    fn create_test_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn create_test_agent(id: &str, name: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test agent".to_string(),
            purpose: "Testing".to_string(),
            agent_type: "test".to_string(),
            capabilities: vec!["send".to_string()],
            status: AgentStatus::Active,
            api_token_hash: None,
            token_prefix: None,
            balance_visible: true,
            invitation_code: None,
            created_at: 1740700800,
            updated_at: 1740700800,
            last_active_at: None,
            metadata: "{}".to_string(),
        }
    }

    #[test]
    fn test_list_agents_command() {
        let db = create_test_db();

        // Insert test agents
        let agent1 = create_test_agent("agent-1", "Claude");
        let agent2 = create_test_agent("agent-2", "GPT");

        insert_agent(&db, &agent1).unwrap();
        insert_agent(&db, &agent2).unwrap();

        // Query all agents
        let agents = queries::list_all_agents(&db).unwrap();
        assert_eq!(agents.len(), 2);

        let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"Claude"));
        assert!(names.contains(&"GPT"));
    }
}
