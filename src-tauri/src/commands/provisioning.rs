use std::sync::Arc;

use tauri::State;

use crate::provisioning::error::ProvisioningError;
use crate::provisioning::types::*;
use crate::provisioning::ProvisioningService;

#[tauri::command]
pub async fn detect_tools(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<DetectionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    let results = tokio::task::spawn_blocking(move || svc.detect_tools())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?;
    Ok(results)
}

#[tauri::command]
pub async fn get_provisioning_preview(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
    config: McpInjectionConfig,
) -> Result<ProvisionPreview, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.get_preview(tool, &config))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn provision_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
    config: McpInjectionConfig,
) -> Result<ProvisionResult, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.provision_tool(tool, &config))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn provision_all(
    service: State<'_, Arc<ProvisioningService>>,
    config: McpInjectionConfig,
) -> Result<Vec<ProvisionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    let results = tokio::task::spawn_blocking(move || svc.provision_all(&config))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?;
    Ok(results)
}

#[tauri::command]
pub async fn unprovision_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
) -> Result<UnprovisionResult, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.unprovision_tool(tool))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn unprovision_all(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<UnprovisionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    let results = tokio::task::spawn_blocking(move || svc.unprovision_all())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?;
    Ok(results)
}

#[tauri::command]
pub async fn verify_provisioning(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<VerificationResult>, ProvisioningError> {
    let svc = service.inner().clone();
    let results = tokio::task::spawn_blocking(move || svc.verify_provisioning())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?;
    Ok(results)
}

#[tauri::command]
pub async fn get_provisioning_state(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<ProvisioningState, ProvisioningError> {
    service.get_state()
}

#[tauri::command]
pub async fn exclude_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
) -> Result<(), ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.exclude_tool(tool))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn include_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
) -> Result<(), ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.include_tool(tool))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn refresh_detection(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<DetectionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    let results = tokio::task::spawn_blocking(move || svc.refresh_detection())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?;
    Ok(results)
}
