use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::error::{Result, SubflowError};
use crate::translate::provider::TranslationProvider;

pub struct LibreTranslateProvider {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl LibreTranslateProvider {
    pub fn new(base_url: Option<&str>, api_key: Option<&str>) -> Self {
        let base_url = base_url
            .filter(|u| !u.is_empty())
            .unwrap_or("http://localhost:5000")
            .trim_end_matches('/')
            .to_string();
        Self {
            client: Client::new(),
            base_url,
            api_key: api_key.filter(|k| !k.is_empty()).map(String::from),
        }
    }
}

#[async_trait]
impl TranslationProvider for LibreTranslateProvider {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let url = format!("{}/translate", self.base_url);
        let source = if source_lang == "auto" { "auto" } else { source_lang };

        // LibreTranslate supports batch via array in "q" field
        let mut body = json!({
            "q": texts,
            "source": source,
            "target": target_lang,
            "format": "text",
        });

        if let Some(ref key) = self.api_key {
            body["api_key"] = json!(key);
        }

        let mut last_error = None;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
            }

            let result = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            match result {
                Ok(response) if response.status().is_success() => {
                    let json: serde_json::Value = response.json().await?;

                    // Response format: {"translatedText": ["text1", "text2"]} for batch
                    // or {"translatedText": "text"} for single
                    if let Some(arr) = json["translatedText"].as_array() {
                        let translations: Vec<String> = arr
                            .iter()
                            .map(|v| v.as_str().unwrap_or("").to_string())
                            .collect();
                        return Ok(translations);
                    } else if let Some(text) = json["translatedText"].as_str() {
                        return Ok(vec![text.to_string()]);
                    }

                    return Err(SubflowError::Translation(
                        "Invalid LibreTranslate response format".to_string(),
                    ));
                }
                Ok(response) => {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    last_error = Some(SubflowError::Translation(format!(
                        "LibreTranslate API error {}: {}",
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
        let url = format!("{}/languages", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SubflowError::Translation(e.to_string()))?;

        Ok(response.status().is_success())
    }

    fn name(&self) -> &str {
        "LibreTranslate"
    }
}
