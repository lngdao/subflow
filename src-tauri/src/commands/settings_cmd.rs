use crate::config::AppConfig;
use crate::error::SubflowError;
use crate::queue::orchestrator;

#[tauri::command]
pub async fn get_settings() -> Result<AppConfig, SubflowError> {
    AppConfig::load()
}

#[tauri::command]
pub async fn save_settings(config: AppConfig) -> Result<(), SubflowError> {
    config.save()
}

#[tauri::command]
pub async fn save_api_key(provider: String, api_key: String) -> Result<(), SubflowError> {
    let service = format!("subflow_{}", provider);
    orchestrator::keyring_set(&service, &api_key)
        .map_err(|e| SubflowError::Config(format!("Failed to save API key: {}", e)))
}

#[tauri::command]
pub async fn test_provider_connection(
    provider: String,
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<bool, SubflowError> {
    let translator = crate::translate::provider::create_provider(
        &provider,
        &api_key,
        base_url.as_deref(),
        model.as_deref(),
    )?;
    translator.test_connection().await
}
