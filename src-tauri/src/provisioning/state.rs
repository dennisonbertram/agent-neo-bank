use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::provisioning::error::ProvisioningError;
use crate::provisioning::types::*;

/// Manages provisioning state persistence.
pub struct StateManager {
    state_path: PathBuf,
}

impl StateManager {
    pub fn new(tally_dir: &Path) -> Self {
        Self {
            state_path: tally_dir.join("provisioning-state.json"),
        }
    }

    /// Load state from disk, or create default state if file doesn't exist.
    pub fn load(&self) -> Result<ProvisioningState, ProvisioningError> {
        if !self.state_path.exists() {
            return Ok(self.create_default_state()?);
        }

        let contents = std::fs::read_to_string(&self.state_path)
            .map_err(|e| ProvisioningError::StateError(format!("Failed to read state: {}", e)))?;
        let state: ProvisioningState = serde_json::from_str(&contents)
            .map_err(|e| ProvisioningError::StateError(format!("Failed to parse state: {}", e)))?;

        Ok(state)
    }

    /// Save state to disk.
    pub fn save(&self, state: &ProvisioningState) -> Result<(), ProvisioningError> {
        if let Some(parent) = self.state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&self.state_path, json)
            .map_err(|e| ProvisioningError::StateError(format!("Failed to write state: {}", e)))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &self.state_path,
                std::fs::Permissions::from_mode(0o600),
            )?;
        }

        Ok(())
    }

    /// Create default state with machine ID.
    fn create_default_state(&self) -> Result<ProvisioningState, ProvisioningError> {
        let machine_id = self.load_or_create_machine_id()?;

        Ok(ProvisioningState {
            schema_version: 1,
            machine_id,
            tally_version: env!("CARGO_PKG_VERSION").to_string(),
            tools: HashMap::new(),
            excluded_tools: HashSet::new(),
            last_scan: None,
        })
    }

    /// Load or create `~/.tally/machine-id`.
    fn load_or_create_machine_id(&self) -> Result<String, ProvisioningError> {
        let tally_dir = self.state_path.parent().ok_or(ProvisioningError::Internal(
            "State path has no parent".into(),
        ))?;
        let machine_id_path = tally_dir.join("machine-id");

        if machine_id_path.exists() {
            let id = std::fs::read_to_string(&machine_id_path)
                .map_err(|e| ProvisioningError::StateError(format!("Failed to read machine-id: {}", e)))?;
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }

        // Generate new machine ID
        let id = uuid::Uuid::new_v4().to_string();
        std::fs::create_dir_all(tally_dir)?;
        std::fs::write(&machine_id_path, &id)
            .map_err(|e| ProvisioningError::StateError(format!("Failed to write machine-id: {}", e)))?;

        Ok(id)
    }

    /// Mark a tool as provisioned.
    pub fn mark_provisioned(
        &self,
        state: &mut ProvisioningState,
        tool: ToolId,
        version: &str,
        files: Vec<PathBuf>,
    ) {
        let tool_state = state
            .tools
            .entry(tool)
            .or_insert_with(|| ToolProvisioningState {
                status: ToolStatus::Unknown,
                provisioned_at: None,
                last_verified: None,
                provisioned_version: None,
                tool_version: None,
                removal_count: 0,
                respect_removal: false,
                files_managed: vec![],
            });

        tool_state.status = ToolStatus::Provisioned;
        tool_state.provisioned_at = Some(chrono::Utc::now().to_rfc3339());
        tool_state.provisioned_version = Some(version.to_string());
        tool_state.files_managed = files;
        tool_state.last_verified = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mark a tool as detected but not provisioned.
    pub fn mark_detected(
        &self,
        state: &mut ProvisioningState,
        tool: ToolId,
        tool_version: Option<String>,
    ) {
        let tool_state = state
            .tools
            .entry(tool)
            .or_insert_with(|| ToolProvisioningState {
                status: ToolStatus::Unknown,
                provisioned_at: None,
                last_verified: None,
                provisioned_version: None,
                tool_version: None,
                removal_count: 0,
                respect_removal: false,
                files_managed: vec![],
            });

        // Don't downgrade from Provisioned to Detected
        if tool_state.status != ToolStatus::Provisioned {
            tool_state.status = ToolStatus::Detected;
        }
        tool_state.tool_version = tool_version;
    }

    /// Mark a tool's config as removed (detected missing during verification).
    pub fn mark_removed(&self, state: &mut ProvisioningState, tool: ToolId) {
        if let Some(tool_state) = state.tools.get_mut(&tool) {
            tool_state.status = ToolStatus::Removed;
            tool_state.removal_count += 1;

            // After 2 removals, respect the user's intent
            if tool_state.removal_count >= 2 {
                tool_state.respect_removal = true;
            }
        }
    }

    /// Mark a tool as unprovisioned (explicitly by user).
    pub fn mark_unprovisioned(&self, state: &mut ProvisioningState, tool: ToolId) {
        if let Some(tool_state) = state.tools.get_mut(&tool) {
            tool_state.status = ToolStatus::Detected;
            tool_state.provisioned_at = None;
            tool_state.provisioned_version = None;
            tool_state.files_managed.clear();
        }
    }

    /// Exclude a tool from provisioning.
    pub fn exclude_tool(&self, state: &mut ProvisioningState, tool: ToolId) {
        state.excluded_tools.insert(tool);
        if let Some(tool_state) = state.tools.get_mut(&tool) {
            tool_state.status = ToolStatus::Excluded;
        }
    }

    /// Include a previously excluded tool.
    pub fn include_tool(&self, state: &mut ProvisioningState, tool: ToolId) {
        state.excluded_tools.remove(&tool);
        if let Some(tool_state) = state.tools.get_mut(&tool) {
            if tool_state.status == ToolStatus::Excluded {
                tool_state.status = ToolStatus::Detected;
            }
        }
    }

    /// Update last scan timestamp.
    pub fn update_last_scan(&self, state: &mut ProvisioningState) {
        state.last_scan = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Update last verified timestamp for a tool.
    pub fn update_last_verified(&self, state: &mut ProvisioningState, tool: ToolId) {
        if let Some(tool_state) = state.tools.get_mut(&tool) {
            tool_state.last_verified = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    /// Check if a tool should be re-provisioned (respect removal protocol).
    pub fn should_reprovision(&self, state: &ProvisioningState, tool: ToolId) -> bool {
        if state.excluded_tools.contains(&tool) {
            return false;
        }
        if let Some(tool_state) = state.tools.get(&tool) {
            if tool_state.respect_removal {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: create a StateManager rooted in a temp directory.
    fn setup() -> (TempDir, StateManager) {
        let tmp = TempDir::new().expect("tempdir");
        let mgr = StateManager::new(tmp.path());
        (tmp, mgr)
    }

    /// Helper: create a default ProvisioningState with a known machine_id.
    fn default_state() -> ProvisioningState {
        ProvisioningState {
            schema_version: 1,
            machine_id: "test-machine".into(),
            tally_version: "0.1.0".into(),
            tools: HashMap::new(),
            excluded_tools: HashSet::new(),
            last_scan: None,
        }
    }

    /// Helper: insert a tool with the given status into a state.
    fn insert_tool(state: &mut ProvisioningState, tool: ToolId, status: ToolStatus) {
        state.tools.insert(tool, ToolProvisioningState {
            status,
            provisioned_at: None,
            last_verified: None,
            provisioned_version: None,
            tool_version: None,
            removal_count: 0,
            respect_removal: false,
            files_managed: vec![],
        });
    }

    // ── StateManager::new ──

    #[test]
    fn new_sets_correct_state_path() {
        let tmp = TempDir::new().unwrap();
        let mgr = StateManager::new(tmp.path());
        // Access via load — if file doesn't exist, load succeeds (creates default).
        // The state_path should be tally_dir/provisioning-state.json.
        let expected = tmp.path().join("provisioning-state.json");
        // Save then check the file exists at the expected path.
        let state = default_state();
        mgr.save(&state).unwrap();
        assert!(expected.exists());
    }

    #[test]
    fn new_does_not_create_file() {
        let tmp = TempDir::new().unwrap();
        let _mgr = StateManager::new(tmp.path());
        let path = tmp.path().join("provisioning-state.json");
        assert!(!path.exists(), "constructor should not create the file");
    }

    // ── load ──

    #[test]
    fn load_missing_file_returns_default_state() {
        let (_tmp, mgr) = setup();
        let state = mgr.load().unwrap();
        assert_eq!(state.schema_version, 1);
        assert!(state.tools.is_empty());
        assert!(state.excluded_tools.is_empty());
        assert!(state.last_scan.is_none());
        assert!(!state.machine_id.is_empty(), "machine_id should be generated");
    }

    #[test]
    fn load_valid_json_round_trips() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        state.last_scan = Some("2026-01-01T00:00:00Z".into());
        insert_tool(&mut state, ToolId::Cursor, ToolStatus::Detected);
        mgr.save(&state).unwrap();

        let loaded = mgr.load().unwrap();
        assert_eq!(loaded.machine_id, "test-machine");
        assert_eq!(loaded.last_scan.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert_eq!(loaded.tools.get(&ToolId::Cursor).unwrap().status, ToolStatus::Detected);
    }

    #[test]
    fn load_malformed_json_returns_error() {
        let (tmp, mgr) = setup();
        let path = tmp.path().join("provisioning-state.json");
        std::fs::write(&path, "NOT VALID JSON {{{").unwrap();

        let result = mgr.load();
        assert!(result.is_err(), "malformed JSON should return an error");
    }

    // ── save ──

    #[test]
    fn save_writes_valid_json() {
        let (tmp, mgr) = setup();
        let state = default_state();
        mgr.save(&state).unwrap();

        let raw = std::fs::read_to_string(tmp.path().join("provisioning-state.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["machine_id"], "test-machine");
    }

    #[test]
    fn save_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("a").join("b").join("c");
        let mgr = StateManager::new(&nested);
        let state = default_state();
        mgr.save(&state).unwrap();
        assert!(nested.join("provisioning-state.json").exists());
    }

    #[test]
    fn save_then_load_round_trips() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        state.excluded_tools.insert(ToolId::Aider);
        insert_tool(&mut state, ToolId::ClaudeCode, ToolStatus::Provisioned);
        mgr.save(&state).unwrap();

        let loaded = mgr.load().unwrap();
        assert!(loaded.excluded_tools.contains(&ToolId::Aider));
        assert_eq!(
            loaded.tools.get(&ToolId::ClaudeCode).unwrap().status,
            ToolStatus::Provisioned
        );
    }

    // ── mark_provisioned ──

    #[test]
    fn mark_provisioned_sets_status_and_version() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        let files = vec![PathBuf::from("/a/b"), PathBuf::from("/c/d")];
        mgr.mark_provisioned(&mut state, ToolId::Cursor, "1.2.3", files.clone());

        let ts = state.tools.get(&ToolId::Cursor).unwrap();
        assert_eq!(ts.status, ToolStatus::Provisioned);
        assert_eq!(ts.provisioned_version.as_deref(), Some("1.2.3"));
        assert_eq!(ts.files_managed, files);
        assert!(ts.provisioned_at.is_some());
    }

    #[test]
    fn mark_provisioned_sets_last_verified() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        mgr.mark_provisioned(&mut state, ToolId::Windsurf, "0.1.0", vec![]);
        let ts = state.tools.get(&ToolId::Windsurf).unwrap();
        assert!(ts.last_verified.is_some());
    }

    // ── mark_detected ──

    #[test]
    fn mark_detected_new_tool_becomes_detected() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        mgr.mark_detected(&mut state, ToolId::Aider, Some("3.0".into()));

        let ts = state.tools.get(&ToolId::Aider).unwrap();
        assert_eq!(ts.status, ToolStatus::Detected);
        assert_eq!(ts.tool_version.as_deref(), Some("3.0"));
    }

    #[test]
    fn mark_detected_does_not_downgrade_provisioned() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        mgr.mark_provisioned(&mut state, ToolId::Cline, "1.0", vec![]);
        assert_eq!(state.tools.get(&ToolId::Cline).unwrap().status, ToolStatus::Provisioned);

        mgr.mark_detected(&mut state, ToolId::Cline, Some("2.0".into()));
        let ts = state.tools.get(&ToolId::Cline).unwrap();
        assert_eq!(ts.status, ToolStatus::Provisioned, "should NOT downgrade to Detected");
        assert_eq!(ts.tool_version.as_deref(), Some("2.0"), "version should still update");
    }

    // ── mark_removed ──

    #[test]
    fn mark_removed_sets_status_and_increments_count() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::Codex, ToolStatus::Provisioned);

        mgr.mark_removed(&mut state, ToolId::Codex);
        let ts = state.tools.get(&ToolId::Codex).unwrap();
        assert_eq!(ts.status, ToolStatus::Removed);
        assert_eq!(ts.removal_count, 1);
        assert!(!ts.respect_removal);
    }

    #[test]
    fn mark_removed_twice_sets_respect_removal() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::Codex, ToolStatus::Provisioned);

        mgr.mark_removed(&mut state, ToolId::Codex);
        mgr.mark_removed(&mut state, ToolId::Codex);

        let ts = state.tools.get(&ToolId::Codex).unwrap();
        assert_eq!(ts.removal_count, 2);
        assert!(ts.respect_removal, "respect_removal should be true after 2 removals");
    }

    #[test]
    fn mark_removed_unknown_tool_is_noop() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        // No tool inserted — should not panic.
        mgr.mark_removed(&mut state, ToolId::Copilot);
        assert!(!state.tools.contains_key(&ToolId::Copilot));
    }

    // ── mark_unprovisioned ──

    #[test]
    fn mark_unprovisioned_reverts_to_detected() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        mgr.mark_provisioned(&mut state, ToolId::ClaudeDesktop, "1.0", vec![PathBuf::from("/x")]);

        mgr.mark_unprovisioned(&mut state, ToolId::ClaudeDesktop);
        let ts = state.tools.get(&ToolId::ClaudeDesktop).unwrap();
        assert_eq!(ts.status, ToolStatus::Detected);
        assert!(ts.provisioned_at.is_none());
        assert!(ts.provisioned_version.is_none());
        assert!(ts.files_managed.is_empty());
    }

    #[test]
    fn mark_unprovisioned_unknown_tool_is_noop() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        mgr.mark_unprovisioned(&mut state, ToolId::Copilot);
        assert!(!state.tools.contains_key(&ToolId::Copilot));
    }

    // ── exclude_tool ──

    #[test]
    fn exclude_tool_adds_to_set_and_sets_status() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::Cursor, ToolStatus::Detected);

        mgr.exclude_tool(&mut state, ToolId::Cursor);
        assert!(state.excluded_tools.contains(&ToolId::Cursor));
        assert_eq!(state.tools.get(&ToolId::Cursor).unwrap().status, ToolStatus::Excluded);
    }

    #[test]
    fn exclude_tool_without_prior_tool_state_adds_to_set() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        mgr.exclude_tool(&mut state, ToolId::Aider);
        assert!(state.excluded_tools.contains(&ToolId::Aider));
        // No tool entry exists, so no status to check — but set should contain it.
    }

    // ── include_tool ──

    #[test]
    fn include_tool_removes_from_set_and_reverts_excluded_to_detected() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::Windsurf, ToolStatus::Detected);
        mgr.exclude_tool(&mut state, ToolId::Windsurf);
        assert_eq!(state.tools.get(&ToolId::Windsurf).unwrap().status, ToolStatus::Excluded);

        mgr.include_tool(&mut state, ToolId::Windsurf);
        assert!(!state.excluded_tools.contains(&ToolId::Windsurf));
        assert_eq!(state.tools.get(&ToolId::Windsurf).unwrap().status, ToolStatus::Detected);
    }

    #[test]
    fn include_tool_non_excluded_status_unchanged() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::Cursor, ToolStatus::Provisioned);

        mgr.include_tool(&mut state, ToolId::Cursor);
        assert_eq!(
            state.tools.get(&ToolId::Cursor).unwrap().status,
            ToolStatus::Provisioned,
            "non-Excluded status should not change"
        );
    }

    // ── update_last_scan ──

    #[test]
    fn update_last_scan_sets_timestamp() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        assert!(state.last_scan.is_none());

        mgr.update_last_scan(&mut state);
        assert!(state.last_scan.is_some());
    }

    #[test]
    fn update_last_scan_overwrites_previous() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        state.last_scan = Some("old-timestamp".into());

        mgr.update_last_scan(&mut state);
        assert_ne!(state.last_scan.as_deref(), Some("old-timestamp"));
    }

    // ── update_last_verified ──

    #[test]
    fn update_last_verified_sets_timestamp_for_known_tool() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::ClaudeCode, ToolStatus::Provisioned);

        mgr.update_last_verified(&mut state, ToolId::ClaudeCode);
        assert!(state.tools.get(&ToolId::ClaudeCode).unwrap().last_verified.is_some());
    }

    #[test]
    fn update_last_verified_unknown_tool_is_noop() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        // Should not panic for unknown tool.
        mgr.update_last_verified(&mut state, ToolId::Copilot);
        assert!(!state.tools.contains_key(&ToolId::Copilot));
    }

    // ── should_reprovision ──

    #[test]
    fn should_reprovision_excluded_returns_false() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        state.excluded_tools.insert(ToolId::Aider);

        assert!(!mgr.should_reprovision(&state, ToolId::Aider));
    }

    #[test]
    fn should_reprovision_respect_removal_returns_false() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::Cursor, ToolStatus::Removed);
        state.tools.get_mut(&ToolId::Cursor).unwrap().respect_removal = true;

        assert!(!mgr.should_reprovision(&state, ToolId::Cursor));
    }

    #[test]
    fn should_reprovision_normal_tool_returns_true() {
        let (_tmp, mgr) = setup();
        let mut state = default_state();
        insert_tool(&mut state, ToolId::Cursor, ToolStatus::Detected);

        assert!(mgr.should_reprovision(&state, ToolId::Cursor));
    }

    #[test]
    fn should_reprovision_unknown_tool_returns_true() {
        let (_tmp, mgr) = setup();
        let state = default_state();
        // Tool not in state at all — should still be eligible.
        assert!(mgr.should_reprovision(&state, ToolId::Copilot));
    }
}
