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
        "claude" | "anthropic" => Ok(Box::new(super::claude::ClaudeProvider::new(
            api_key,
            base_url,
            model,
        ))),
        "openai" | "openai_compatible" => Ok(Box::new(super::openai::OpenAIProvider::new(
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
        "libretranslate" => Ok(Box::new(super::libretranslate::LibreTranslateProvider::new(
            base_url,
            Some(api_key).filter(|k| !k.is_empty()),
        ))),
        #[cfg(feature = "nllb-native")]
        "nllb" => {
            // Native CTranslate2 provider
            let variant = model
                .map(crate::model_manager::NllbModelVariant::from_str)
                .unwrap_or(crate::model_manager::NllbModelVariant::Distilled600M);
            if !crate::model_manager::is_model_ready(variant) {
                return Err(crate::error::SubflowError::Translation(format!(
                    "{} not downloaded. Open Dependencies to download.",
                    variant.display_name()
                )));
            }
            let model_dir = crate::model_manager::nllb_model_dir(variant);
            Ok(Box::new(super::nllb_native::NllbNativeLazyProvider { model_dir }))
        }
        #[cfg(not(feature = "nllb-native"))]
        "nllb" => Err(crate::error::SubflowError::Translation(
            "NLLB native translation is not available on this platform. Use NLLB-200 (Server) instead.".into(),
        )),
        "nllb_api" => Ok(Box::new(super::nllb::NllbProvider::new(base_url))),
        _ => Err(crate::error::SubflowError::Translation(format!(
            "Unknown provider: {}",
            provider_name
        ))),
    }
}
