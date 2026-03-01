use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::error::{Result, SubflowError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub id: String,
    pub title: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub channel: Option<String>,
    pub upload_date: Option<String>,
}

pub async fn get_metadata(url: &str) -> Result<VideoMetadata> {
    crate::youtube::downloader::check_ytdlp().await?;

    let output = Command::new("yt-dlp")
        .args(["--dump-json", "--no-download", url])
        .output()
        .await
        .map_err(|e| SubflowError::YouTube(format!("Failed to run yt-dlp: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SubflowError::YouTube(format!(
            "yt-dlp metadata failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| SubflowError::YouTube(e.to_string()))?;

    Ok(VideoMetadata {
        id: json["id"].as_str().unwrap_or("unknown").to_string(),
        title: json["title"].as_str().unwrap_or("Unknown").to_string(),
        duration: json["duration"].as_f64(),
        thumbnail: json["thumbnail"].as_str().map(String::from),
        channel: json["channel"].as_str().map(String::from),
        upload_date: json["upload_date"].as_str().map(String::from),
    })
}
