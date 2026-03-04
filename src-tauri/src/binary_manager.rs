use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{Result, SubflowError};

/// Directory where SubFlow stores downloaded binaries
fn bin_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("subflow")
        .join("bin")
}

/// Check if a binary exists and is executable
fn binary_exists(name: &str) -> bool {
    let path = bin_dir().join(name);
    path.exists()
}

/// Check if a binary is available in system PATH
async fn binary_in_path(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get download URL for yt-dlp based on current platform
fn ytdlp_download_url() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        // Universal binary for macOS
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    }
    #[cfg(target_os = "windows")]
    {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    }
    #[cfg(target_os = "linux")]
    {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux"
    }
}

/// Get download URL for ffmpeg based on current platform
fn ffmpeg_download_url() -> Option<&'static str> {
    #[cfg(target_os = "macos")]
    {
        // ffmpeg static build for macOS (universal binary)
        Some("https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-darwin-arm64")
    }
    #[cfg(target_os = "windows")]
    {
        Some("https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-win32-x64.exe")
    }
    #[cfg(target_os = "linux")]
    {
        Some("https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-linux-x64")
    }
}

/// Download a file from URL to the specified path
async fn download_binary(url: &str, dest: &PathBuf) -> Result<()> {
    tracing::info!("Downloading {} to {:?}", url, dest);

    let response = reqwest::get(url)
        .await
        .map_err(|e| SubflowError::YouTube(format!("Failed to download: {}", e)))?;

    if !response.status().is_success() {
        // Check for redirect (GitHub releases redirect)
        return Err(SubflowError::YouTube(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| SubflowError::YouTube(format!("Failed to read download: {}", e)))?;

    std::fs::write(dest, &bytes)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dest, perms)?;
    }

    tracing::info!("Downloaded successfully: {:?}", dest);
    Ok(())
}

/// Ensure yt-dlp is available. Downloads if not found.
pub async fn ensure_ytdlp() -> Result<PathBuf> {
    let bin = bin_dir();
    let ytdlp_path = bin.join("yt-dlp");

    // Already downloaded locally
    if ytdlp_path.exists() {
        return Ok(ytdlp_path);
    }

    // Available in system PATH
    if binary_in_path("yt-dlp").await {
        return Ok(PathBuf::from("yt-dlp"));
    }

    // Download it
    std::fs::create_dir_all(&bin)?;
    let url = ytdlp_download_url();
    download_binary(url, &ytdlp_path).await?;

    Ok(ytdlp_path)
}

/// Ensure ffmpeg is available. Downloads if not found.
pub async fn ensure_ffmpeg() -> Result<Option<PathBuf>> {
    let bin = bin_dir();
    let ffmpeg_path = bin.join("ffmpeg");

    // Already downloaded locally
    if ffmpeg_path.exists() {
        return Ok(Some(ffmpeg_path));
    }

    // Available in system PATH
    if binary_in_path("ffmpeg").await {
        return Ok(Some(PathBuf::from("ffmpeg")));
    }

    // Download it
    if let Some(url) = ffmpeg_download_url() {
        std::fs::create_dir_all(&bin)?;
        match download_binary(url, &ffmpeg_path).await {
            Ok(()) => Ok(Some(ffmpeg_path)),
            Err(e) => {
                tracing::warn!("Failed to download ffmpeg (not critical): {}", e);
                Ok(None) // ffmpeg is optional for subtitle-only workflows
            }
        }
    } else {
        Ok(None)
    }
}

/// Status of binary dependencies
#[derive(serde::Serialize)]
pub struct BinaryStatus {
    pub ytdlp_available: bool,
    pub ffmpeg_available: bool,
    pub ytdlp_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub nllb_600m_available: bool,
    pub nllb_600m_path: Option<String>,
    pub nllb_1_3b_available: bool,
    pub nllb_1_3b_path: Option<String>,
}

/// Check current status of binaries
pub async fn check_status() -> BinaryStatus {
    let ytdlp_local = bin_dir().join("yt-dlp");
    let ffmpeg_local = bin_dir().join("ffmpeg");

    let ytdlp_available = ytdlp_local.exists() || binary_in_path("yt-dlp").await;
    let ffmpeg_available = ffmpeg_local.exists() || binary_in_path("ffmpeg").await;

    use crate::model_manager::{is_model_ready, nllb_model_dir, NllbModelVariant};

    let nllb_600m_available = is_model_ready(NllbModelVariant::Distilled600M);
    let nllb_600m_dir = nllb_model_dir(NllbModelVariant::Distilled600M);
    let nllb_1_3b_available = is_model_ready(NllbModelVariant::Distilled1_3B);
    let nllb_1_3b_dir = nllb_model_dir(NllbModelVariant::Distilled1_3B);

    BinaryStatus {
        ytdlp_available,
        ffmpeg_available,
        ytdlp_path: if ytdlp_local.exists() {
            Some(ytdlp_local.to_string_lossy().to_string())
        } else if ytdlp_available {
            Some("yt-dlp (system)".to_string())
        } else {
            None
        },
        ffmpeg_path: if ffmpeg_local.exists() {
            Some(ffmpeg_local.to_string_lossy().to_string())
        } else if ffmpeg_available {
            Some("ffmpeg (system)".to_string())
        } else {
            None
        },
        nllb_600m_available,
        nllb_600m_path: if nllb_600m_available {
            Some(nllb_600m_dir.to_string_lossy().to_string())
        } else {
            None
        },
        nllb_1_3b_available,
        nllb_1_3b_path: if nllb_1_3b_available {
            Some(nllb_1_3b_dir.to_string_lossy().to_string())
        } else {
            None
        },
    }
}
