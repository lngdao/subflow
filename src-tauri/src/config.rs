use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{Result, SubflowError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub translation: TranslationConfig,
    pub tts: TtsConfig,
    pub output: OutputConfig,
    pub queue: QueueConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
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
    #[serde(default)]
    pub voices: HashMap<String, String>,
    #[serde(skip_serializing, default)]
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
    #[serde(default = "default_parallel_langs")]
    pub parallel_langs: u32,
    #[serde(default = "default_pipeline_tts")]
    pub pipeline_tts: bool,
    #[serde(default = "default_tts_chunk_size")]
    pub tts_chunk_size: u32,
}

fn default_parallel_langs() -> u32 { 2 }
fn default_pipeline_tts() -> bool { true }
fn default_tts_chunk_size() -> u32 { 500 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            translation: TranslationConfig {
                provider: "claude".to_string(),
                base_url: None,
                model: Some("claude-haiku-4-5-20251001".to_string()),
                source_lang: "auto".to_string(),
                target_langs: vec!["vi".to_string()],
            },
            tts: TtsConfig {
                provider: "edge".to_string(),
                voices: HashMap::new(),
                voice: None,
                speed: 1.0,
            },
            output: OutputConfig {
                format: "srt".to_string(),
                folder: default_output_folder(),
            },
            queue: QueueConfig {
                parallel_jobs: 2,
                parallel_langs: 2,
                pipeline_tts: true,
                tts_chunk_size: 500,
            },
            notifications: NotificationConfig::default(),
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

fn infer_lang_from_voice(voice: &str) -> &str {
    // Voice names follow pattern: "xx-XX-NameNeural"
    // Extract the language prefix (e.g., "vi" from "vi-VN-HoaiMyNeural")
    if let Some(dash) = voice.find('-') {
        &voice[..dash]
    } else {
        "en"
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let mut config: AppConfig =
                serde_json::from_str(&contents).map_err(|e| SubflowError::Config(e.to_string()))?;

            // Migrate old single-voice to per-language voices map
            if config.tts.voices.is_empty() {
                if let Some(ref voice) = config.tts.voice {
                    let lang = infer_lang_from_voice(voice);
                    config.tts.voices.insert(lang.to_string(), voice.clone());
                }
            }

            Ok(config)
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
