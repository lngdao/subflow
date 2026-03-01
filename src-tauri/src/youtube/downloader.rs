use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::error::{Result, SubflowError};

pub async fn check_ytdlp() -> Result<()> {
    let result = Command::new("yt-dlp").arg("--version").output().await;
    match result {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(SubflowError::YtDlpNotFound),
    }
}

pub async fn download_subtitle(
    url: &str,
    output_dir: &Path,
    sub_lang: &str,
) -> Result<PathBuf> {
    check_ytdlp().await?;

    std::fs::create_dir_all(output_dir)?;

    let output = Command::new("yt-dlp")
        .args([
            "--write-subs",
            "--write-auto-subs",
            "--sub-format",
            "srt",
            "--sub-langs",
            sub_lang,
            "--skip-download",
            "--output",
            &output_dir.join("%(id)s").to_string_lossy(),
            url,
        ])
        .output()
        .await
        .map_err(|e| SubflowError::YouTube(format!("Failed to run yt-dlp: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SubflowError::YouTube(format!(
            "yt-dlp failed: {}",
            stderr
        )));
    }

    // Find the downloaded subtitle file
    let entries = std::fs::read_dir(output_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "srt" || ext == "vtt" {
                return Ok(path);
            }
        }
    }

    Err(SubflowError::YouTube(
        "No subtitle file found after download".to_string(),
    ))
}
