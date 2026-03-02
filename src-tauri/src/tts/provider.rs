use async_trait::async_trait;
use std::path::Path;

use crate::error::Result;
use crate::tts::types::VoiceInfo;

#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, text: &str, voice: &str, speed: f32, output_path: &Path)
        -> Result<()>;
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>>;
    fn name(&self) -> &str;
}

pub fn create_provider(
    provider_name: &str,
    api_key: Option<&str>,
) -> Result<Box<dyn TtsProvider>> {
    create_provider_with_opts(provider_name, api_key, None)
}

pub fn create_provider_with_opts(
    provider_name: &str,
    api_key: Option<&str>,
    tts_chunk_size: Option<usize>,
) -> Result<Box<dyn TtsProvider>> {
    match provider_name {
        "edge" => {
            let provider = match tts_chunk_size {
                Some(size) => super::edge::EdgeTtsProvider::with_chunk_size(size),
                None => super::edge::EdgeTtsProvider::new(),
            };
            Ok(Box::new(provider))
        }
        "openai" => {
            let key = api_key.ok_or_else(|| {
                crate::error::SubflowError::Tts("OpenAI TTS requires an API key".to_string())
            })?;
            Ok(Box::new(super::openai_tts::OpenAITtsProvider::new(key)))
        }
        _ => Err(crate::error::SubflowError::Tts(format!(
            "Unknown TTS provider: {}",
            provider_name
        ))),
    }
}
