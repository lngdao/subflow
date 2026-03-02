use std::path::PathBuf;

use crate::error::SubflowError;
use crate::subtitle::parser;
use crate::subtitle::types::SubtitleFile;
use crate::youtube::metadata::VideoMetadata;

#[tauri::command]
pub async fn download_subtitle(
    url: String,
    output_dir: String,
    lang: Option<String>,
) -> Result<String, SubflowError> {
    let lang = lang.unwrap_or_else(|| "en".to_string());
    let path =
        crate::source::provider::download_subtitle(&url, &PathBuf::from(&output_dir), &lang)
            .await?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_video_metadata(url: String) -> Result<Option<VideoMetadata>, SubflowError> {
    crate::source::provider::get_metadata(&url).await
}

#[tauri::command]
pub async fn parse_subtitle_file(path: String) -> Result<SubtitleFile, SubflowError> {
    let content = std::fs::read_to_string(&path)?;
    parser::parse_auto(&content)
}
