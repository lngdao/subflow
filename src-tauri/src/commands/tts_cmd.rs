use std::path::PathBuf;

use crate::error::SubflowError;
use crate::tts::provider as tts_provider;
use crate::tts::types::VoiceInfo;

#[tauri::command]
pub async fn generate_tts(
    text: String,
    voice: String,
    speed: Option<f32>,
    output_path: String,
    provider_name: Option<String>,
    api_key: Option<String>,
) -> Result<String, SubflowError> {
    let config = crate::config::AppConfig::load()?;
    let provider_name = provider_name.unwrap_or(config.tts.provider);
    let speed = speed.unwrap_or(config.tts.speed);

    let tts = tts_provider::create_provider(&provider_name, api_key.as_deref())?;

    let path = PathBuf::from(&output_path);
    tts.synthesize(&text, &voice, speed, &path).await?;

    Ok(output_path)
}

#[tauri::command]
pub async fn list_tts_voices(
    provider_name: Option<String>,
    api_key: Option<String>,
) -> Result<Vec<VoiceInfo>, SubflowError> {
    let config = crate::config::AppConfig::load()?;
    let provider_name = provider_name.unwrap_or(config.tts.provider);

    let tts = tts_provider::create_provider(&provider_name, api_key.as_deref())?;
    tts.list_voices().await
}
