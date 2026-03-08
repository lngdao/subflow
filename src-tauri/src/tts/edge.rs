use async_trait::async_trait;
use std::path::Path;

use crate::error::{Result, SubflowError};
use crate::tts::provider::TtsProvider;
use crate::tts::types::VoiceInfo;

const DEFAULT_CHUNK_SIZE: usize = 500;

pub struct EdgeTtsProvider {
    chunk_size: usize,
}

impl EdgeTtsProvider {
    pub fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }

    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunk_size: chunk_size.max(100),
        }
    }
}

/// Split text into chunks of approximately `max_chars` characters,
/// breaking at sentence boundaries (". ", "! ", "? ", "\n") to avoid
/// cutting mid-sentence.
fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.len() <= max_chars {
            chunks.push(remaining.to_string());
            break;
        }

        // Find a char-safe boundary at or before max_chars bytes
        let mut safe_end = max_chars;
        while safe_end > 0 && !remaining.is_char_boundary(safe_end) {
            safe_end -= 1;
        }

        // Find the best split point near safe_end
        let search_region = &remaining[..safe_end];
        let split_at = search_region
            .rfind(". ")
            .map(|i| i + 2)
            .or_else(|| search_region.rfind("! ").map(|i| i + 2))
            .or_else(|| search_region.rfind("? ").map(|i| i + 2))
            .or_else(|| search_region.rfind('\n').map(|i| i + 1))
            .or_else(|| search_region.rfind(", ").map(|i| i + 2))
            .or_else(|| search_region.rfind(' ').map(|i| i + 1))
            .unwrap_or(safe_end);

        let (chunk, rest) = remaining.split_at(split_at);
        let trimmed = chunk.trim();
        if !trimmed.is_empty() {
            chunks.push(trimmed.to_string());
        }
        remaining = rest.trim_start();
    }

    chunks
}

/// Synthesize a single chunk of text with retries.
fn synthesize_single(text: &str, voice: &str) -> std::result::Result<Vec<u8>, String> {
    use msedge_tts::tts::client::connect as tts_connect;
    use msedge_tts::tts::SpeechConfig;

    let max_retries = 3;
    let mut last_err = None;

    for attempt in 0..max_retries {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(1000 * attempt as u64));
        }

        let mut tts = match tts_connect() {
            Ok(t) => t,
            Err(e) => {
                last_err = Some(format!("Edge TTS connection failed: {}", e));
                continue;
            }
        };

        let config = SpeechConfig {
            voice_name: voice.to_string(),
            audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
            pitch: 0,
            rate: 0,
            volume: 0,
        };

        match tts.synthesize(text, &config) {
            Ok(audio) => return Ok(audio.audio_bytes),
            Err(e) => {
                last_err = Some(format!("Edge TTS synthesis failed: {}", e));
                continue;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "Edge TTS failed after retries".to_string()))
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
        let chunks = chunk_text(text, self.chunk_size);
        let voice = voice.to_string();
        let output_path = output_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut all_audio = Vec::new();
            for chunk in &chunks {
                let audio_bytes = synthesize_single(chunk, &voice)
                    .map_err(SubflowError::Tts)?;
                all_audio.extend(audio_bytes);
            }

            std::fs::write(&output_path, &all_audio)?;
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
