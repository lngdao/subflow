use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::error::{Result, SubflowError};
use crate::translate::chunker;
use crate::translate::provider::TranslationProvider;

pub struct GlmProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl GlmProvider {
    pub fn new(api_key: &str, base_url: Option<&str>, model: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            base_url: base_url
                .unwrap_or("https://open.bigmodel.cn/api/paas")
                .trim_end_matches('/')
                .to_string(),
            model: model.unwrap_or("glm-5").to_string(),
        }
    }

    async fn call_api(&self, prompt: &str) -> Result<String> {
        // GLM uses OpenAI-compatible API
        let url = format!("{}/v4/chat/completions", self.base_url);

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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SubflowError::Translation(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SubflowError::Translation(format!(
                "GLM API error {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| SubflowError::Translation("No content in GLM response".to_string()))?;

        Ok(content.to_string())
    }
}

#[async_trait]
impl TranslationProvider for GlmProvider {
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
        "GLM"
    }
}
