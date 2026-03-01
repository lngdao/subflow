use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{Result, SubflowError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub translation: TranslationConfig,
    pub tts: TtsConfig,
    pub output: OutputConfig,
    pub queue: QueueConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub source_lang: String,
    pub target_langs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub provider: String,
    pub voice: Option<String>,
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
    pub folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub parallel_jobs: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            translation: TranslationConfig {
                provider: "claude".to_string(),
                base_url: None,
                model: Some("claude-haiku-4-5-20251001".to_string()),
                source_lang: "en".to_string(),
                target_langs: vec!["vi".to_string()],
            },
            tts: TtsConfig {
                provider: "edge".to_string(),
                voice: Some("en-US-AriaNeural".to_string()),
                speed: 1.0,
            },
            output: OutputConfig {
                format: "srt".to_string(),
                folder: default_output_folder(),
            },
            queue: QueueConfig { parallel_jobs: 2 },
        }
    }
}

fn default_output_folder() -> String {
    dirs::download_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("subflow")
        .to_string_lossy()
        .to_string()
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("subflow")
        .join("config.json")
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            serde_json::from_str(&contents).map_err(|e| SubflowError::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
}
