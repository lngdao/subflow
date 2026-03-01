use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::error::{Result, SubflowError};

/// Extract clean YouTube video URL (strip playlist params, etc.)
pub fn clean_youtube_url(url: &str) -> String {
    // Try to extract video ID
    let video_id = extract_video_id(url);
    match video_id {
        Some(id) => format!("https://www.youtube.com/watch?v={}", id),
        None => url.to_string(),
    }
}

fn extract_video_id(url: &str) -> Option<&str> {
    // Handle youtu.be/ID
    if let Some(rest) = url.strip_prefix("https://youtu.be/")
        .or_else(|| url.strip_prefix("http://youtu.be/"))
    {
        let id = rest.split(&['?', '&', '/'][..]).next()?;
        if id.len() == 11 {
            return Some(id);
        }
    }
    // Handle youtube.com/watch?v=ID or youtube.com/shorts/ID
    if url.contains("youtube.com/shorts/") {
        let after = url.split("youtube.com/shorts/").nth(1)?;
        let id = after.split(&['?', '&', '/'][..]).next()?;
        if id.len() == 11 {
            return Some(id);
        }
    }
    if url.contains("v=") {
        let after = url.split("v=").nth(1)?;
        let id = after.split(&['&', '#'][..]).next()?;
        if id.len() == 11 {
            return Some(id);
        }
    }
    None
}

pub async fn check_ytdlp() -> Result<()> {
    let yt_dlp = get_ytdlp_path();
    let result = Command::new(&yt_dlp).arg("--version").output().await;
    match result {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(SubflowError::YtDlpNotFound),
    }
}

/// Get path to yt-dlp binary. Checks app data dir first, then falls back to system PATH.
pub fn get_ytdlp_path() -> String {
    if let Some(config_dir) = dirs::config_dir() {
        let local_bin = config_dir.join("subflow").join("bin").join("yt-dlp");
        if local_bin.exists() {
            return local_bin.to_string_lossy().to_string();
        }
    }
    "yt-dlp".to_string()
}

/// Get path to ffmpeg binary. Checks app data dir first, then falls back to system PATH.
fn get_ffmpeg_path() -> String {
    if let Some(config_dir) = dirs::config_dir() {
        let local_bin = config_dir.join("subflow").join("bin").join("ffmpeg");
        if local_bin.exists() {
            return local_bin.to_string_lossy().to_string();
        }
    }
    "ffmpeg".to_string()
}

pub async fn download_subtitle(
    url: &str,
    output_dir: &Path,
    sub_lang: &str,
) -> Result<PathBuf> {
    let clean_url = clean_youtube_url(url);

    // Try direct YouTube API fetch first (faster, no yt-dlp issues)
    if let Some(video_id) = extract_video_id(&clean_url) {
        match super::subtitle_fetcher::fetch_subtitle_direct(video_id, output_dir, sub_lang).await
        {
            Ok(path) => return Ok(path),
            Err(e) => {
                tracing::warn!("Direct subtitle fetch failed, falling back to yt-dlp: {}", e);
            }
        }
    }

    // Fallback: use yt-dlp
    download_subtitle_ytdlp(&clean_url, output_dir, sub_lang).await
}

async fn download_subtitle_ytdlp(
    clean_url: &str,
    output_dir: &Path,
    sub_lang: &str,
) -> Result<PathBuf> {
    check_ytdlp().await?;

    std::fs::create_dir_all(output_dir)?;

    // "auto" means download whatever subtitle is available
    let effective_lang = if sub_lang == "auto" { "en.*" } else { sub_lang };

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
        // Avoid impersonation requirement + reduce 429 errors
        "--extractor-args".to_string(),
        "youtube:player_client=mweb,web".to_string(),
        "--sleep-requests".to_string(),
        "1".to_string(),
        "--output".to_string(),
        output_dir.join("%(id)s").to_string_lossy().to_string(),
    ];

    // Point to local ffmpeg if available
    if ffmpeg_path != "ffmpeg" {
        args.push("--ffmpeg-location".to_string());
        args.push(ffmpeg_path);
    }

    args.push(clean_url.to_string());

    // Retry up to 3 times for transient errors (429, network issues)
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
            return Err(SubflowError::YouTube(
                "No subtitle file found after download".to_string(),
            ));
        }

        last_err = String::from_utf8_lossy(&output.stderr).to_string();

        // Only retry on 429 or network errors
        if !last_err.contains("429") && !last_err.contains("Connection") && !last_err.contains("timed out") {
            break;
        }
    }

    Err(SubflowError::YouTube(format!("yt-dlp failed: {}", last_err)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_youtube_url() {
        assert_eq!(
            clean_youtube_url("https://www.youtube.com/watch?v=nugcMvyRVoE&list=RDnugcMvyRVoE&start_radio=1"),
            "https://www.youtube.com/watch?v=nugcMvyRVoE"
        );
        assert_eq!(
            clean_youtube_url("https://www.youtube.com/watch?v=pS40dZhch3o"),
            "https://www.youtube.com/watch?v=pS40dZhch3o"
        );
        assert_eq!(
            clean_youtube_url("https://youtu.be/pS40dZhch3o"),
            "https://www.youtube.com/watch?v=pS40dZhch3o"
        );
    }

    #[test]
    fn test_extract_video_id() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=nugcMvyRVoE&list=RDnugcMvyRVoE"),
            Some("nugcMvyRVoE")
        );
        assert_eq!(
            extract_video_id("https://youtu.be/pS40dZhch3o"),
            Some("pS40dZhch3o")
        );
        assert_eq!(
            extract_video_id("https://www.youtube.com/shorts/abcdefghijk"),
            Some("abcdefghijk")
        );
    }
}
