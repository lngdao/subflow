use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::error::{Result, SubflowError};
use crate::source::provider::SourceProvider;
use crate::youtube::downloader::{check_ytdlp, get_ytdlp_path};
use crate::youtube::metadata::VideoMetadata;

pub struct GenericProvider;

impl GenericProvider {
    pub fn new() -> Self {
        Self
    }
}

/// Get path to ffmpeg binary.
fn get_ffmpeg_path() -> String {
    if let Some(config_dir) = dirs::config_dir() {
        let local_bin = config_dir.join("subflow").join("bin").join("ffmpeg");
        if local_bin.exists() {
            return local_bin.to_string_lossy().to_string();
        }
    }
    "ffmpeg".to_string()
}

#[async_trait]
impl SourceProvider for GenericProvider {
    async fn download_subtitle(&self, url: &str, output_dir: &Path, lang: &str) -> Result<PathBuf> {
        check_ytdlp().await?;

        std::fs::create_dir_all(output_dir)?;

        let effective_lang = if lang == "auto" { "en.*" } else { lang };

        let yt_dlp = get_ytdlp_path();
        let ffmpeg_path = get_ffmpeg_path();
        let mut args = vec![
            "--no-playlist".to_string(),
            "--write-subs".to_string(),
            "--write-auto-subs".to_string(),
            "--sub-format".to_string(),
            "srt".to_string(),
            "--sub-langs".to_string(),
            effective_lang.to_string(),
            "--skip-download".to_string(),
            "--sleep-requests".to_string(),
            "1".to_string(),
            "--output".to_string(),
            output_dir.join("%(id)s").to_string_lossy().to_string(),
        ];

        if ffmpeg_path != "ffmpeg" {
            args.push("--ffmpeg-location".to_string());
            args.push(ffmpeg_path);
        }

        args.push(url.to_string());

        let max_retries = 3;
        let mut last_err = String::new();

        for attempt in 0..max_retries {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt as u32))).await;
            }

            let output = Command::new(&yt_dlp)
                .args(&args)
                .output()
                .await
                .map_err(|e| SubflowError::YouTube(format!("Failed to run yt-dlp: {}", e)))?;

            if output.status.success() {
                let entries = std::fs::read_dir(output_dir)?;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "srt" || ext == "vtt" {
                            return Ok(path);
                        }
                    }
                }
                return Err(SubflowError::YouTube(
                    "No subtitle file found after download".to_string(),
                ));
            }

            last_err = String::from_utf8_lossy(&output.stderr).to_string();

            if !last_err.contains("429")
                && !last_err.contains("Connection")
                && !last_err.contains("timed out")
            {
                break;
            }
        }

        Err(SubflowError::YouTube(format!("yt-dlp failed: {}", last_err)))
    }

    async fn get_metadata(&self, url: &str) -> Result<Option<VideoMetadata>> {
        check_ytdlp().await?;

        let yt_dlp = get_ytdlp_path();
        let output = Command::new(&yt_dlp)
            .args([
                "--dump-json",
                "--no-download",
                "--no-playlist",
                url,
            ])
            .output()
            .await
            .map_err(|e| SubflowError::YouTube(format!("Failed to run yt-dlp: {}", e)))?;

        if !output.status.success() {
            // Non-fatal: metadata is optional
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = match serde_json::from_str(&stdout) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        Ok(Some(VideoMetadata {
            id: json["id"].as_str().unwrap_or("unknown").to_string(),
            title: json["title"].as_str().unwrap_or("Unknown").to_string(),
            duration: json["duration"].as_f64(),
            thumbnail: json["thumbnail"].as_str().map(String::from),
            channel: json["channel"]
                .as_str()
                .or_else(|| json["uploader"].as_str())
                .map(String::from),
            upload_date: json["upload_date"].as_str().map(String::from),
        }))
    }

    fn can_handle(&self, _url: &str) -> bool {
        true // Generic provider handles everything as fallback
    }

    fn name(&self) -> &str {
        "Generic (yt-dlp)"
    }
}
