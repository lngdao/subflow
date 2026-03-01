use async_trait::async_trait;
use std::path::Path;

use crate::error::{Result, SubflowError};
use crate::tts::provider::TtsProvider;
use crate::tts::types::VoiceInfo;

pub struct EdgeTtsProvider;

impl EdgeTtsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TtsProvider for EdgeTtsProvider {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        _speed: f32,
        output_path: &Path,
    ) -> Result<()> {
        let text = text.to_string();
        let voice = voice.to_string();
        let output_path = output_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            use msedge_tts::tts::client::connect as tts_connect;
            use msedge_tts::tts::SpeechConfig;

            let mut tts = tts_connect()
                .map_err(|e| SubflowError::Tts(format!("Edge TTS connection failed: {}", e)))?;

            let config = SpeechConfig {
                voice_name: voice,
                audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
                pitch: 0,
                rate: 0,
                volume: 0,
            };
            let audio = tts
                .synthesize(&text, &config)
                .map_err(|e| SubflowError::Tts(format!("Edge TTS synthesis failed: {}", e)))?;

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&output_path, &audio.audio_bytes)?;

            Ok(())
        })
        .await
        .map_err(|e| SubflowError::Tts(format!("TTS task join error: {}", e)))?
    }

    async fn list_voices(&self) -> Result<Vec<VoiceInfo>> {
        tokio::task::spawn_blocking(|| {
            use msedge_tts::voice::get_voices_list;

            let voices = get_voices_list()
                .map_err(|e| SubflowError::Tts(format!("Failed to get voices: {}", e)))?;

            Ok(voices
                .into_iter()
                .map(|v| {
                    let id = v.short_name.clone().unwrap_or_default();
                    VoiceInfo {
                        name: v.friendly_name.unwrap_or_else(|| id.clone()),
                        id,
                        language: v.locale.unwrap_or_default(),
                        gender: v.gender,
                    }
                })
                .collect())
        })
        .await
        .map_err(|e| SubflowError::Tts(format!("Task join error: {}", e)))?
    }

    fn name(&self) -> &str {
        "Edge TTS"
    }
}
