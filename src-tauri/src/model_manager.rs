use std::path::PathBuf;

use futures_util::StreamExt;
use serde::Serialize;
use tauri::Emitter;

use crate::error::{Result, SubflowError};

const NLLB_MODEL_DIR_NAME: &str = "nllb-600M";
const HF_BASE_URL: &str =
    "https://huggingface.co/entai2965/nllb-200-distilled-600M-ctranslate2/resolve/main";

const REQUIRED_FILES: &[&str] = &[
    "model.bin",
    "sentencepiece.bpe.model",
    "shared_vocabulary.json",
    "config.json",
];

#[derive(Debug, Clone, Serialize)]
pub struct ModelDownloadProgress {
    pub model: String,
    pub file: String,
    pub bytes_downloaded: u64,
    pub bytes_total: Option<u64>,
    pub percent: f64,
    pub status: String, // "downloading" | "completed" | "failed"
}

/// Root directory for all SubFlow models
pub fn models_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("subflow")
        .join("models")
}

/// Directory for the NLLB-600M model
pub fn nllb_model_dir() -> PathBuf {
    models_dir().join(NLLB_MODEL_DIR_NAME)
}

/// Check if all required model files exist
pub fn is_model_ready() -> bool {
    let dir = nllb_model_dir();
    REQUIRED_FILES.iter().all(|f| dir.join(f).exists())
}

/// Download the NLLB model from HuggingFace with progress events
pub async fn download_nllb_model(app_handle: tauri::AppHandle) -> Result<()> {
    let model_dir = nllb_model_dir();
    std::fs::create_dir_all(&model_dir)?;

    let client = reqwest::Client::new();

    for file_name in REQUIRED_FILES {
        let dest = model_dir.join(file_name);

        // Skip already downloaded files
        if dest.exists() {
            tracing::info!("Model file already exists, skipping: {}", file_name);
            let _ = app_handle.emit(
                "model-download-progress",
                ModelDownloadProgress {
                    model: "nllb".into(),
                    file: file_name.to_string(),
                    bytes_downloaded: 0,
                    bytes_total: None,
                    percent: 100.0,
                    status: "completed".into(),
                },
            );
            continue;
        }

        let part_path = model_dir.join(format!("{}.part", file_name));
        let url = format!("{}/{}", HF_BASE_URL, file_name);

        tracing::info!("Downloading {} from {}", file_name, url);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| SubflowError::Translation(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SubflowError::Translation(format!(
                "Download failed for {}: HTTP {}",
                file_name,
                response.status()
            )));
        }

        let total_size = response.content_length();
        let mut downloaded: u64 = 0;
        let mut file = std::fs::File::create(&part_path)?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|e| SubflowError::Translation(format!("Download error: {}", e)))?;
            std::io::Write::write_all(&mut file, &chunk)?;
            downloaded += chunk.len() as u64;

            let percent = total_size
                .map(|t| (downloaded as f64 / t as f64) * 100.0)
                .unwrap_or(0.0);

            let _ = app_handle.emit(
                "model-download-progress",
                ModelDownloadProgress {
                    model: "nllb".into(),
                    file: file_name.to_string(),
                    bytes_downloaded: downloaded,
                    bytes_total: total_size,
                    percent,
                    status: "downloading".into(),
                },
            );
        }

        drop(file);
        std::fs::rename(&part_path, &dest)?;

        let _ = app_handle.emit(
            "model-download-progress",
            ModelDownloadProgress {
                model: "nllb".into(),
                file: file_name.to_string(),
                bytes_downloaded: downloaded,
                bytes_total: total_size,
                percent: 100.0,
                status: "completed".into(),
            },
        );

        tracing::info!("Downloaded {}: {} bytes", file_name, downloaded);
    }

    Ok(())
}

/// Delete the NLLB model directory to free disk space
pub fn delete_nllb_model() -> Result<()> {
    let dir = nllb_model_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
        tracing::info!("Deleted NLLB model at {:?}", dir);
    }
    Ok(())
}
