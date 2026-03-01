use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::error::{Result, SubflowError};
use crate::translate::chunker;
use crate::translate::provider::TranslationProvider;

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str, model: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.unwrap_or("gemini-2.0-flash").to_string(),
        }
    }

    async fn call_api(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = json!({
            "contents": [
                {
                    "parts": [{"text": prompt}]
                }
            ],
            "generationConfig": {
                "temperature": 0.3,
                "maxOutputTokens": 4096
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SubflowError::Translation(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SubflowError::Translation(format!(
                "Gemini API error {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response.json().await?;
        let content = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| SubflowError::Translation("No content in Gemini response".to_string()))?;

        Ok(content.to_string())
    }
}

#[async_trait]
impl TranslationProvider for GeminiProvider {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let prompt = chunker::build_prompt(texts, source_lang, target_lang);

        let mut last_error = None;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
            }
            match self.call_api(&prompt).await {
                Ok(response) => {
                    return Ok(chunker::parse_response(&response, texts.len()));
                }
                Err(e) => {
                    tracing::warn!("Translation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn test_connection(&self) -> Result<bool> {
        match self.call_api("Say 'ok' and nothing else.").await {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    fn name(&self) -> &str {
        "Gemini"
    }
}
