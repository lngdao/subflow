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

/// Directory for the Python venv with yt-dlp + curl_cffi
fn ytdlp_env_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("subflow")
        .join("ytdlp-env")
}

/// Get the yt-dlp binary path from the venv
fn ytdlp_env_bin() -> PathBuf {
    let env = ytdlp_env_dir();
    #[cfg(target_os = "windows")]
    { env.join("Scripts").join("yt-dlp.exe") }
    #[cfg(not(target_os = "windows"))]
    { env.join("bin").join("yt-dlp") }
}

/// Get platform-correct binary filename (appends .exe on Windows)
fn binary_filename(name: &str) -> String {
    #[cfg(target_os = "windows")]
    { format!("{}.exe", name) }
    #[cfg(not(target_os = "windows"))]
    { name.to_string() }
}

/// Common binary directories that may not be in PATH when launched as a .app
#[cfg(target_os = "macos")]
const EXTRA_BIN_DIRS: &[&str] = &[
    "/opt/homebrew/bin",
    "/usr/local/bin",
    "/opt/local/bin",
];

#[cfg(target_os = "linux")]
const EXTRA_BIN_DIRS: &[&str] = &[
    "/usr/local/bin",
    "/snap/bin",
    "/home/linuxbrew/.linuxbrew/bin",
];

#[cfg(target_os = "windows")]
const EXTRA_BIN_DIRS: &[&str] = &[];

/// Check if a binary is available in system PATH or common install locations.
/// GUI apps on macOS don't inherit shell PATH, so we also check homebrew etc.
async fn binary_in_path(name: &str) -> bool {
    // ffmpeg uses -version (single dash), others use --version
    let version_arg = if name == "ffmpeg" || name == "ffprobe" { "-version" } else { "--version" };

    // Try system PATH first
    if Command::new(name)
        .arg(version_arg)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check common directories (important for macOS .app bundles)
    for dir in EXTRA_BIN_DIRS {
        let full_path = format!("{}/{}", dir, name);
        if std::path::Path::new(&full_path).exists() {
            if Command::new(&full_path)
                .arg("--version")
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return true;
            }
        }
    }

    false
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
        Some("https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-win32-x64")
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

/// Ensure yt-dlp is available. Checks: venv → local binary → system PATH → download standalone.
pub async fn ensure_ytdlp() -> Result<PathBuf> {
    // Prefer venv with curl_cffi (supports impersonation)
    let env_bin = ytdlp_env_bin();
    if env_bin.exists() {
        return Ok(env_bin);
    }

    let bin = bin_dir();
    let ytdlp_path = bin.join(binary_filename("yt-dlp"));

    // Already downloaded locally
    if ytdlp_path.exists() {
        return Ok(ytdlp_path);
    }

    // Available in system PATH
    if binary_in_path("yt-dlp").await {
        return Ok(PathBuf::from("yt-dlp"));
    }

    // Download standalone binary
    std::fs::create_dir_all(&bin)?;
    let url = ytdlp_download_url();
    download_binary(url, &ytdlp_path).await?;

    Ok(ytdlp_path)
}

/// Ensure ffmpeg is available. Downloads if not found.
pub async fn ensure_ffmpeg() -> Result<Option<PathBuf>> {
    let bin = bin_dir();
    let ffmpeg_path = bin.join(binary_filename("ffmpeg"));

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
    pub curl_cffi_available: bool,
    /// True if we have a managed venv (user can delete it)
    pub ytdlp_env_exists: bool,
}

/// Set up a Python venv with yt-dlp + curl_cffi for browser impersonation.
/// Required for downloading YouTube auto-translated subtitles without 429 errors.
pub async fn setup_ytdlp_env() -> Result<()> {
    let env_dir = ytdlp_env_dir();

    // Find python3
    let python = find_python().await
        .ok_or_else(|| SubflowError::Config("Python 3 not found. Install Python 3 first.".to_string()))?;

    tracing::info!("Setting up yt-dlp env with Python: {}", python);

    // Create venv
    let output = Command::new(&python)
        .args(["-m", "venv", &env_dir.to_string_lossy()])
        .output()
        .await
        .map_err(|e| SubflowError::Config(format!("Failed to create venv: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SubflowError::Config(format!("venv creation failed: {}", stderr)));
    }

    // Get pip path in venv
    #[cfg(target_os = "windows")]
    let pip = env_dir.join("Scripts").join("pip");
    #[cfg(not(target_os = "windows"))]
    let pip = env_dir.join("bin").join("pip");

    // Install yt-dlp + curl_cffi
    let output = Command::new(&pip)
        .args(["install", "--upgrade", "yt-dlp", "curl_cffi"])
        .output()
        .await
        .map_err(|e| SubflowError::Config(format!("pip install failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Clean up failed venv
        let _ = std::fs::remove_dir_all(&env_dir);
        return Err(SubflowError::Config(format!("pip install failed: {}", stderr)));
    }

    tracing::info!("yt-dlp env setup complete at {:?}", env_dir);
    Ok(())
}

/// Remove the yt-dlp venv
pub fn delete_ytdlp_env() -> Result<()> {
    let env_dir = ytdlp_env_dir();
    if env_dir.exists() {
        std::fs::remove_dir_all(&env_dir)?;
    }
    Ok(())
}

/// Check if curl_cffi is available in the current yt-dlp installation
async fn check_curl_cffi() -> bool {
    // Check venv first
    let env_bin = ytdlp_env_bin();
    if env_bin.exists() {
        let output = Command::new(&env_bin)
            .args(["--list-impersonate-targets"])
            .output()
            .await;
        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            return stdout.contains("curl_cffi") && !stdout.contains("(unavailable)");
        }
    }

    // Check system yt-dlp
    if binary_in_path("yt-dlp").await {
        let output = Command::new("yt-dlp")
            .args(["--list-impersonate-targets"])
            .output()
            .await;
        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            return stdout.contains("curl_cffi") && !stdout.contains("(unavailable)");
        }
    }

    false
}

/// Find a working Python 3 executable
async fn find_python() -> Option<String> {
    for name in &["python3", "python"] {
        let output = Command::new(name)
            .args(["--version"])
            .output()
            .await;
        if let Ok(o) = output {
            if o.status.success() {
                let version = String::from_utf8_lossy(&o.stdout);
                if version.contains("Python 3") {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

/// Find a binary's full path — checks local, then PATH, then common directories
async fn resolve_binary_path(name: &str) -> Option<String> {
    let local = bin_dir().join(binary_filename(name));
    if local.exists() {
        return Some(local.to_string_lossy().to_string());
    }

    // Check PATH
    if binary_in_path(name).await {
        return Some(format!("{} (system)", name));
    }

    // Check common directories
    for dir in EXTRA_BIN_DIRS {
        let full_path = format!("{}/{}", dir, name);
        if std::path::Path::new(&full_path).exists() {
            return Some(full_path);
        }
    }

    None
}

/// Check current status of binaries
pub async fn check_status() -> BinaryStatus {
    let ytdlp_env = ytdlp_env_bin();
    let ytdlp_local = bin_dir().join(binary_filename("yt-dlp"));

    let ytdlp_available;
    let ytdlp_path;
    if ytdlp_env.exists() {
        ytdlp_available = true;
        ytdlp_path = Some(ytdlp_env.to_string_lossy().to_string());
    } else if ytdlp_local.exists() {
        ytdlp_available = true;
        ytdlp_path = Some(ytdlp_local.to_string_lossy().to_string());
    } else if let Some(p) = resolve_binary_path("yt-dlp").await {
        ytdlp_available = true;
        ytdlp_path = Some(p);
    } else {
        ytdlp_available = false;
        ytdlp_path = None;
    };

    let ffmpeg_local = bin_dir().join(binary_filename("ffmpeg"));
    let ffmpeg_available;
    let ffmpeg_path;
    if ffmpeg_local.exists() {
        ffmpeg_available = true;
        ffmpeg_path = Some(ffmpeg_local.to_string_lossy().to_string());
    } else if let Some(p) = resolve_binary_path("ffmpeg").await {
        ffmpeg_available = true;
        ffmpeg_path = Some(p);
    } else {
        ffmpeg_available = false;
        ffmpeg_path = None;
    };

    let curl_cffi_available = check_curl_cffi().await;

    use crate::model_manager::{is_model_ready, nllb_model_dir, NllbModelVariant};

    let nllb_600m_available = is_model_ready(NllbModelVariant::Distilled600M);
    let nllb_600m_dir = nllb_model_dir(NllbModelVariant::Distilled600M);
    let nllb_1_3b_available = is_model_ready(NllbModelVariant::Distilled1_3B);
    let nllb_1_3b_dir = nllb_model_dir(NllbModelVariant::Distilled1_3B);

    BinaryStatus {
        ytdlp_available,
        ffmpeg_available,
        ytdlp_path,
        ffmpeg_path,
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
        curl_cffi_available,
        ytdlp_env_exists: ytdlp_env_bin().exists(),
    }
}
