use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use ct2rs::tokenizers::sentencepiece;
use ct2rs::{Config, TranslationOptions, Translator};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::error::{Result, SubflowError};
use crate::translate::provider::TranslationProvider;

struct TranslateRequest {
    texts: Vec<String>,
    source_lang: String,
    target_lang: String,
    reply: oneshot::Sender<Result<Vec<String>>>,
}

pub struct NllbNativeProvider {
    sender: mpsc::Sender<TranslateRequest>,
}

static INSTANCE: OnceLock<Mutex<Option<Arc<NllbNativeProvider>>>> = OnceLock::new();

/// Get or lazily initialize the singleton NLLB native provider.
pub async fn get_or_init_provider() -> Result<Arc<NllbNativeProvider>> {
    let lock = INSTANCE.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().await;

    if let Some(ref provider) = *guard {
        return Ok(Arc::clone(provider));
    }

    let model_dir = crate::model_manager::nllb_model_dir();
    let provider = Arc::new(NllbNativeProvider::new(model_dir)?);
    *guard = Some(Arc::clone(&provider));
    Ok(provider)
}

impl NllbNativeProvider {
    pub fn new(model_dir: PathBuf) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel::<TranslateRequest>(32);

        std::thread::spawn(move || {
            worker_loop(model_dir, &mut rx);
        });

        Ok(Self { sender: tx })
    }
}

fn worker_loop(model_dir: PathBuf, receiver: &mut mpsc::Receiver<TranslateRequest>) {
    let spm_path = model_dir.join("sentencepiece.bpe.model");

    let tokenizer = match sentencepiece::Tokenizer::from_file(&spm_path, &spm_path) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to load NLLB tokenizer: {}", e);
            // Drain remaining requests with error
            while let Some(req) = receiver.blocking_recv() {
                let _ = req.reply.send(Err(SubflowError::Translation(
                    "NLLB tokenizer failed to load".into(),
                )));
            }
            return;
        }
    };

    let translator = match Translator::with_tokenizer(&model_dir, tokenizer, &Config::default()) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to load NLLB model: {}", e);
            while let Some(req) = receiver.blocking_recv() {
                let _ = req.reply.send(Err(SubflowError::Translation(
                    "NLLB model failed to load".into(),
                )));
            }
            return;
        }
    };

    tracing::info!("NLLB native translator loaded successfully");

    while let Some(req) = receiver.blocking_recv() {
        let result = do_translate(&translator, &req.texts, &req.source_lang, &req.target_lang);
        let _ = req.reply.send(result);
    }

    tracing::info!("NLLB worker thread shutting down");
}

fn do_translate(
    translator: &Translator<sentencepiece::Tokenizer>,
    texts: &[String],
    source_lang: &str,
    target_lang: &str,
) -> Result<Vec<String>> {
    let src = to_flores200(source_lang);
    let tgt = to_flores200(target_lang);

    // Prepend source language code to each input text.
    // The sentencepiece tokenizer will encode this as a special token,
    // and automatically appends </s>.
    let inputs: Vec<String> = texts.iter().map(|t| format!("{} {}", src, t)).collect();
    let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();

    // Target prefix guides the decoder to produce the target language
    let target_prefixes: Vec<Vec<&str>> = texts.iter().map(|_| vec![tgt]).collect();

    let results = translator
        .translate_batch_with_target_prefix(
            &input_refs,
            &target_prefixes,
            &TranslationOptions::<String, String>::default(),
            None,
        )
        .map_err(|e| SubflowError::Translation(format!("NLLB translation failed: {}", e)))?;

    Ok(results.into_iter().map(|(text, _)| text).collect())
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
        "auto" => "eng_Latn",
        _ => lang,
    }
}

#[async_trait]
impl TranslationProvider for NllbNativeProvider {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.sender
            .send(TranslateRequest {
                texts: texts.to_vec(),
                source_lang: source_lang.to_string(),
                target_lang: target_lang.to_string(),
                reply: reply_tx,
            })
            .await
            .map_err(|_| {
                SubflowError::Translation("NLLB worker thread is not running".into())
            })?;

        reply_rx.await.map_err(|_| {
            SubflowError::Translation("NLLB worker did not respond".into())
        })?
    }

    async fn test_connection(&self) -> Result<bool> {
        let result = self
            .translate(&["Hello".to_string()], "en", "vi")
            .await;
        Ok(result.is_ok())
    }

    fn name(&self) -> &str {
        "NLLB-200 (Native)"
    }
}

/// Lazy wrapper returned by `create_provider` (sync context).
/// Initializes the actual NLLB native provider on the first async call.
pub struct NllbNativeLazyProvider;

#[async_trait]
impl TranslationProvider for NllbNativeLazyProvider {
    async fn translate(
        &self,
        texts: &[String],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let provider = get_or_init_provider().await?;
        provider.translate(texts, source_lang, target_lang).await
    }

    async fn test_connection(&self) -> Result<bool> {
        let provider = get_or_init_provider().await?;
        provider.test_connection().await
    }

    fn name(&self) -> &str {
        "NLLB-200 (Native)"
    }
}
