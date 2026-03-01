use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::subtitle::types::SubtitleFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub subtitle: SubtitleFile,
    pub source_lang: String,
    pub target_lang: String,
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(&self, texts: &[String], source_lang: &str, target_lang: &str)
        -> Result<Vec<String>>;
    async fn test_connection(&self) -> Result<bool>;
    fn name(&self) -> &str;
}

pub fn create_provider(
    provider_name: &str,
    api_key: &str,
    base_url: Option<&str>,
    model: Option<&str>,
) -> Result<Box<dyn TranslationProvider>> {
    match provider_name {
        "claude" => Ok(Box::new(super::claude::ClaudeProvider::new(
            api_key,
            base_url,
            model,
        ))),
        "openai" => Ok(Box::new(super::openai::OpenAIProvider::new(
            api_key,
            base_url,
            model,
        ))),
        "gemini" => Ok(Box::new(super::gemini::GeminiProvider::new(
            api_key, model,
        ))),
        "glm" => Ok(Box::new(super::glm::GlmProvider::new(
            api_key,
            base_url,
            model,
        ))),
        "deepl" => Ok(Box::new(super::deepl::DeepLProvider::new(api_key))),
        _ => Err(crate::error::SubflowError::Translation(format!(
            "Unknown provider: {}",
            provider_name
        ))),
    }
}
