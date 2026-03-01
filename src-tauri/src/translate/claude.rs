use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::error::{Result, SubflowError};
use crate::translate::chunker;
use crate::translate::provider::TranslationProvider;

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: &str, base_url: Option<&str>, model: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            base_url: base_url
                .unwrap_or("https://api.anthropic.com")
                .trim_end_matches('/')
                .to_string(),
            model: model.unwrap_or("claude-haiku-4-5-20251001").to_string(),
        }
    }

    async fn call_api(&self, prompt: &str) -> Result<String> {
        // Use Anthropic Messages API format (/v1/messages)
        let url = format!("{}/v1/messages", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 4096,
            "temperature": 0.3
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SubflowError::Translation(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SubflowError::Translation(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response.json().await?;

        // Anthropic response format: { "content": [{ "type": "text", "text": "..." }] }
        let content = json["content"]
            .as_array()
            .and_then(|arr| arr.iter().find(|block| block["type"] == "text"))
            .and_then(|block| block["text"].as_str())
            .ok_or_else(|| SubflowError::Translation("No content in response".to_string()))?;

        Ok(content.to_string())
    }
}

#[async_trait]
impl TranslationProvider for ClaudeProvider {
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
        "Claude"
    }
}
