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

            let max_retries = 3;
            let mut last_err = None;

            for attempt in 0..max_retries {
                if attempt > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1000 * attempt as u64));
                }

                let connect_result = tts_connect();
                let mut tts = match connect_result {
                    Ok(t) => t,
                    Err(e) => {
                        last_err = Some(format!("Edge TTS connection failed: {}", e));
                        continue;
                    }
                };

                let config = SpeechConfig {
                    voice_name: voice.clone(),
                    audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
                    pitch: 0,
                    rate: 0,
                    volume: 0,
                };

                match tts.synthesize(&text, &config) {
                    Ok(audio) => {
                        if let Some(parent) = output_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&output_path, &audio.audio_bytes)?;
                        return Ok(());
                    }
                    Err(e) => {
                        last_err = Some(format!("Edge TTS synthesis failed: {}", e));
                        continue;
                    }
                }
            }

            Err(SubflowError::Tts(last_err.unwrap_or_else(|| "Edge TTS failed after retries".to_string())))
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
