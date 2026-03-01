use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;

use crate::error::{Result, SubflowError};
use crate::tts::provider::TtsProvider;
use crate::tts::types::VoiceInfo;

pub struct OpenAITtsProvider {
    client: Client,
    api_key: String,
}

impl OpenAITtsProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl TtsProvider for OpenAITtsProvider {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        speed: f32,
        output_path: &Path,
    ) -> Result<()> {
        let body = serde_json::json!({
            "model": "tts-1",
            "input": text,
            "voice": voice,
            "speed": speed,
            "response_format": "mp3"
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/audio/speech")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SubflowError::Tts(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SubflowError::Tts(format!(
                "OpenAI TTS error {}: {}",
                status, text
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| SubflowError::Tts(e.to_string()))?;

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, &bytes)?;

        Ok(())
    }

    async fn list_voices(&self) -> Result<Vec<VoiceInfo>> {
        Ok(vec![
            VoiceInfo { id: "alloy".into(), name: "Alloy".into(), language: "en".into(), gender: Some("neutral".into()) },
            VoiceInfo { id: "echo".into(), name: "Echo".into(), language: "en".into(), gender: Some("male".into()) },
            VoiceInfo { id: "fable".into(), name: "Fable".into(), language: "en".into(), gender: Some("male".into()) },
            VoiceInfo { id: "onyx".into(), name: "Onyx".into(), language: "en".into(), gender: Some("male".into()) },
            VoiceInfo { id: "nova".into(), name: "Nova".into(), language: "en".into(), gender: Some("female".into()) },
            VoiceInfo { id: "shimmer".into(), name: "Shimmer".into(), language: "en".into(), gender: Some("female".into()) },
        ])
    }

    fn name(&self) -> &str {
        "OpenAI TTS"
    }
}
