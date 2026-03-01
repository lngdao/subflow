use crate::config::AppConfig;
use crate::error::SubflowError;
use crate::subtitle::types::SubtitleFile;
use crate::translate::chunker;
use crate::translate::provider as translate_provider;

#[tauri::command]
pub async fn translate_subtitle(
    subtitle_json: SubtitleFile,
    source_lang: String,
    target_lang: String,
    provider_name: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<SubtitleFile, SubflowError> {
    let config = AppConfig::load()?;
    let provider_name = provider_name.unwrap_or(config.translation.provider);

    let api_key = api_key.unwrap_or_else(|| {
        let service = format!("subflow_{}", provider_name);
        crate::queue::orchestrator::keyring_get_pub(&service).unwrap_or_default()
    });

    let provider = translate_provider::create_provider(
        &provider_name,
        &api_key,
        base_url.as_deref().or(config.translation.base_url.as_deref()),
        model.as_deref().or(config.translation.model.as_deref()),
    )?;

    let chunks = chunker::chunk_entries(&subtitle_json.entries, None);
    let mut translated_texts = Vec::new();

    for chunk in &chunks {
        let texts: Vec<String> = chunk.entries.iter().map(|(_, t)| t.clone()).collect();
        let result = provider
            .translate(&texts, &source_lang, &target_lang)
            .await?;
        translated_texts.extend(result);
    }

    let mut result = subtitle_json;
    for (entry, text) in result.entries.iter_mut().zip(translated_texts.iter()) {
        entry.text = text.clone();
    }

    Ok(result)
}
