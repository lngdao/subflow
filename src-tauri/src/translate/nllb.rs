use async_trait::async_trait;
use reqwest::Client;

use crate::error::{Result, SubflowError};
use crate::translate::provider::TranslationProvider;

pub struct NllbProvider {
    client: Client,
    base_url: String,
}

impl NllbProvider {
    pub fn new(base_url: Option<&str>) -> Self {
        let base_url = base_url
            .filter(|u| !u.is_empty())
            .unwrap_or("http://localhost:7860")
            .trim_end_matches('/')
            .to_string();
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

/// Map ISO 639-1 codes to FLORES-200 codes used by NLLB.
fn to_flores200(lang: &str) -> &str {
    match lang {
        "en" => "eng_Latn",
        "vi" => "vie_Latn",
        "ja" | "jp" => "jpn_Jpan",
        "ko" | "kr" => "kor_Hang",
        "zh" | "cn" => "zho_Hans",
        "es" => "spa_Latn",
        "fr" => "fra_Latn",
        "de" => "deu_Latn",
        "pt" => "por_Latn",
        "ru" => "rus_Cyrl",
        "ar" => "arb_Arab",
        "hi" => "hin_Deva",
        "th" => "tha_Thai",
        "id" => "ind_Latn",
        "tr" => "tur_Latn",
        "pl" => "pol_Latn",
        "nl" => "nld_Latn",
        "it" => "ita_Latn",
        "auto" => "eng_Latn", // fallback for auto-detect
        _ => lang,
    }
}

#[async_trait]
impl TranslationProvider for NllbProvider {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let source = to_flores200(source_lang);
        let target = to_flores200(target_lang);

        let mut results = Vec::with_capacity(texts.len());

        // NLLB-API processes one text at a time via GET request
        for text in texts {
            let url = format!("{}/api/v4/translator", self.base_url);

            let mut last_error = None;
            let mut translated = String::new();

            for attempt in 0..3 {
                if attempt > 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
                }

                let result = self
                    .client
                    .get(&url)
                    .query(&[
                        ("text", text.as_str()),
                        ("source", source),
                        ("target", target),
                    ])
                    .send()
                    .await;

                match result {
                    Ok(response) if response.status().is_success() => {
                        let json: serde_json::Value = response.json().await?;
                        translated = json["translatedText"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        last_error = None;
                        break;
                    }
                    Ok(response) => {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        last_error = Some(SubflowError::Translation(format!(
                            "NLLB API error {}: {}",
                            status, body
                        )));
                    }
                    Err(e) => {
                        last_error = Some(SubflowError::Translation(e.to_string()));
                    }
                }
            }

            if let Some(err) = last_error {
                return Err(err);
            }

            results.push(translated);
        }

        Ok(results)
    }

    async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/api/v4/translator", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("text", "hello"),
                ("source", "eng_Latn"),
                ("target", "vie_Latn"),
            ])
            .send()
            .await
            .map_err(|e| SubflowError::Translation(e.to_string()))?;

        Ok(response.status().is_success())
    }

    fn name(&self) -> &str {
        "NLLB-200"
    }
}
