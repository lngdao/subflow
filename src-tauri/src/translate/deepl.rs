use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::error::{Result, SubflowError};
use crate::translate::provider::TranslationProvider;

pub struct DeepLProvider {
    client: Client,
    api_key: String,
}

impl DeepLProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    fn base_url(&self) -> &str {
        // Free API keys end with ":fx"
        if self.api_key.ends_with(":fx") {
            "https://api-free.deepl.com"
        } else {
            "https://api.deepl.com"
        }
    }

    fn map_lang_code(lang: &str) -> &str {
        match lang.to_lowercase().as_str() {
            "en" => "EN",
            "vi" => "VI",
            "ja" | "jp" => "JA",
            "ko" | "kr" => "KO",
            "zh" | "cn" => "ZH",
            "de" => "DE",
            "fr" => "FR",
            "es" => "ES",
            "pt" => "PT",
            "ru" => "RU",
            other => {
                // Return as uppercase for unknown codes
                // Note: this leaks but is fine for a small set
                Box::leak(other.to_uppercase().into_boxed_str())
            }
        }
    }
}

#[async_trait]
impl TranslationProvider for DeepLProvider {
    async fn translate(
        &self,
        texts: &[String],
        _source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let url = format!("{}/v2/translate", self.base_url());

        let body = json!({
            "text": texts,
            "target_lang": Self::map_lang_code(target_lang),
        });

        let mut last_error = None;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
            }

            let result = self
                .client
                .post(&url)
                .header(
                    "Authorization",
                    format!("DeepL-Auth-Key {}", self.api_key),
                )
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            match result {
                Ok(response) if response.status().is_success() => {
                    let json: serde_json::Value = response.json().await?;
                    let translations: Vec<String> = json["translations"]
                        .as_array()
                        .ok_or_else(|| {
                            SubflowError::Translation(
                                "Invalid DeepL response format".to_string(),
                            )
                        })?
                        .iter()
                        .map(|t| {
                            t["text"]
                                .as_str()
                                .unwrap_or("")
                                .to_string()
                        })
                        .collect();
                    return Ok(translations);
                }
                Ok(response) => {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    last_error = Some(SubflowError::Translation(format!(
                        "DeepL API error {}: {}",
                        status, text
                    )));
                }
                Err(e) => {
                    last_error = Some(SubflowError::Translation(e.to_string()));
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/v2/usage", self.base_url());
        let response = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("DeepL-Auth-Key {}", self.api_key),
            )
            .send()
            .await
            .map_err(|e| SubflowError::Translation(e.to_string()))?;

        Ok(response.status().is_success())
    }

    fn name(&self) -> &str {
        "DeepL"
    }
}
