use crate::config::AppConfig;
use crate::error::SubflowError;
use crate::queue::orchestrator;
use crate::binary_manager;

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
pub async fn get_api_key_preview(provider: String) -> Result<Option<String>, SubflowError> {
    let service = format!("subflow_{}", provider);
    match orchestrator::keyring_get_pub(&service) {
        Ok(key) if key.len() > 4 => Ok(Some(format!("••••{}", &key[key.len()-4..]))),
        Ok(_) => Ok(Some("••••".to_string())),
        Err(_) => Ok(None),
    }
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

#[tauri::command]
pub async fn ensure_directory(path: String) -> Result<(), SubflowError> {
    std::fs::create_dir_all(&path)?;
    Ok(())
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), SubflowError> {
    std::fs::create_dir_all(&path)?;
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| SubflowError::Config(format!("Failed to open folder: {}", e)))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| SubflowError::Config(format!("Failed to open folder: {}", e)))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| SubflowError::Config(format!("Failed to open folder: {}", e)))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn setup_binaries() -> Result<binary_manager::BinaryStatus, SubflowError> {
    // Auto-download yt-dlp and ffmpeg if not found
    let _ = binary_manager::ensure_ytdlp().await;
    let _ = binary_manager::ensure_ffmpeg().await;
    Ok(binary_manager::check_status().await)
}

#[tauri::command]
pub async fn get_binary_status() -> Result<binary_manager::BinaryStatus, SubflowError> {
    Ok(binary_manager::check_status().await)
}

#[tauri::command]
pub async fn download_nllb_model(app_handle: tauri::AppHandle, variant: String) -> Result<(), SubflowError> {
    let v = crate::model_manager::NllbModelVariant::from_str(&variant);
    crate::model_manager::download_nllb_model(app_handle, v).await
}

#[tauri::command]
pub async fn delete_nllb_model(variant: String) -> Result<(), SubflowError> {
    let v = crate::model_manager::NllbModelVariant::from_str(&variant);
    crate::model_manager::delete_nllb_model(v)
}
