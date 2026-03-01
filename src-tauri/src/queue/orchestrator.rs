use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tauri::Emitter;

use crate::config::AppConfig;
use crate::error::{Result, SubflowError};
use crate::queue::task::Task;
use crate::queue::types::{TaskEvent, TaskStatus};
use crate::subtitle::parser;
use crate::subtitle::types::SubtitleFormat;
use crate::subtitle::writer;
use crate::translate::chunker;
use crate::translate::provider as translate_provider;
use crate::tts::provider as tts_provider;

#[derive(Clone)]
pub struct Orchestrator {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
    semaphore: Arc<Semaphore>,
    cancelled: Arc<Mutex<std::collections::HashSet<String>>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(2)),
            cancelled: Arc::new(Mutex::new(std::collections::HashSet::new())),
        }
    }

    pub async fn add_task(&self, task: Task) -> String {
        let id = task.id.clone();
        self.tasks.lock().await.insert(id.clone(), task);
        id
    }

    pub async fn get_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().await;
        let mut list: Vec<Task> = tasks.values().cloned().collect();
        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        list
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        self.cancelled.lock().await.insert(task_id.to_string());
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            task.message = "Cancelled".to_string();
            Ok(())
        } else {
            Err(SubflowError::TaskNotFound(task_id.to_string()))
        }
    }

    pub async fn pause_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Paused;
            task.message = "Paused".to_string();
            Ok(())
        } else {
            Err(SubflowError::TaskNotFound(task_id.to_string()))
        }
    }

    pub async fn resume_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Queued;
            task.message = "Resumed - Queued".to_string();
            Ok(())
        } else {
            Err(SubflowError::TaskNotFound(task_id.to_string()))
        }
    }

    fn is_cancelled(&self, cancelled: &std::collections::HashSet<String>, task_id: &str) -> bool {
        cancelled.contains(task_id)
    }

    pub async fn process_task(
        &self,
        task_id: &str,
        app_handle: tauri::AppHandle,
    ) -> Result<()> {
        let config = AppConfig::load()?;

        let _permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| SubflowError::Queue("Failed to acquire semaphore".to_string()))?;

        let task = {
            let tasks = self.tasks.lock().await;
            tasks
                .get(task_id)
                .cloned()
                .ok_or_else(|| SubflowError::TaskNotFound(task_id.to_string()))?
        };

        // Step 1: Get subtitle content
        self.emit_event(&app_handle, task_id, TaskStatus::Downloading, 0.05, "Getting subtitle...", None);

        let subtitle_content = if let Some(url) = &task.url {
            // Download from YouTube
            let output_dir = PathBuf::from(&config.output.folder).join(
                task.video_id.as_deref().unwrap_or("temp"),
            );

            let sub_path = crate::youtube::downloader::download_subtitle(
                url,
                &output_dir,
                &task.source_lang,
            )
            .await?;

            std::fs::read_to_string(&sub_path)?
        } else if let Some(path) = &task.file_path {
            std::fs::read_to_string(path)?
        } else {
            return Err(SubflowError::Queue(
                "Task has no URL or file path".to_string(),
            ));
        };

        // Check cancellation
        {
            let cancelled = self.cancelled.lock().await;
            if self.is_cancelled(&cancelled, task_id) {
                return Err(SubflowError::TaskCancelled);
            }
        }

        // Parse subtitle
        let subtitle = parser::parse_auto(&subtitle_content)?;
        let video_id = task.video_id.as_deref().unwrap_or(&task.id);
        let output_dir = PathBuf::from(&config.output.folder).join(video_id);
        std::fs::create_dir_all(&output_dir)?;

        // Save original
        let original_path = output_dir.join(format!("original.{}", config.output.format));
        let out_format = SubtitleFormat::from_extension(&config.output.format)
            .unwrap_or(SubtitleFormat::Srt);
        std::fs::write(&original_path, writer::write_as(&subtitle, &out_format))?;

        // Update task with output dir
        {
            let mut tasks = self.tasks.lock().await;
            if let Some(t) = tasks.get_mut(task_id) {
                t.output_dir = Some(output_dir.to_string_lossy().to_string());
            }
        }

        let total_langs = task.target_langs.len();
        let api_key = self.get_api_key(&config.translation.provider).await;

        // Step 2: Translate for each target language
        for (lang_idx, target_lang) in task.target_langs.iter().enumerate() {
            // Check cancellation
            {
                let cancelled = self.cancelled.lock().await;
                if self.is_cancelled(&cancelled, task_id) {
                    return Err(SubflowError::TaskCancelled);
                }
            }

            let base_progress = 0.1 + (lang_idx as f32 / total_langs as f32) * 0.7;
            self.emit_event(
                &app_handle,
                task_id,
                TaskStatus::Translating,
                base_progress,
                &format!("Translating to {}...", target_lang),
                Some(target_lang),
            );

            // Create translation provider
            let provider = translate_provider::create_provider(
                &config.translation.provider,
                &api_key,
                config.translation.base_url.as_deref(),
                config.translation.model.as_deref(),
            )?;

            // Chunk and translate
            let chunks = chunker::chunk_entries(&subtitle.entries, None);
            let mut translated_texts = Vec::new();

            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                let texts: Vec<String> = chunk.entries.iter().map(|(_, t)| t.clone()).collect();
                let result = provider
                    .translate(&texts, &task.source_lang, target_lang)
                    .await?;
                translated_texts.extend(result);

                let chunk_progress = base_progress
                    + ((chunk_idx + 1) as f32 / chunks.len() as f32)
                        * (0.7 / total_langs as f32)
                        * 0.6;
                self.emit_event(
                    &app_handle,
                    task_id,
                    TaskStatus::Translating,
                    chunk_progress,
                    &format!(
                        "Translating to {} ({}/{} chunks)...",
                        target_lang,
                        chunk_idx + 1,
                        chunks.len()
                    ),
                    Some(target_lang),
                );
            }

            // Build translated subtitle file
            let mut translated_sub = subtitle.clone();
            translated_sub.format = out_format.clone();
            for (entry, text) in translated_sub.entries.iter_mut().zip(translated_texts.iter()) {
                entry.text = text.clone();
            }

            // Save translated subtitle
            let sub_path = output_dir.join(format!("{}.{}", target_lang, config.output.format));
            std::fs::write(&sub_path, writer::write_as(&translated_sub, &out_format))?;

            // Step 3: Generate TTS
            self.emit_event(
                &app_handle,
                task_id,
                TaskStatus::GeneratingTts,
                base_progress + 0.7 / total_langs as f32 * 0.6,
                &format!("Generating TTS for {}...", target_lang),
                Some(target_lang),
            );

            let tts_api_key = if config.tts.provider == "openai" {
                Some(self.get_api_key("openai_tts").await)
            } else {
                None
            };

            let tts = tts_provider::create_provider(
                &config.tts.provider,
                tts_api_key.as_deref(),
            )?;

            // Concatenate all text for single TTS file
            let full_text: String = translated_sub
                .entries
                .iter()
                .map(|e| e.text.as_str())
                .collect::<Vec<_>>()
                .join(". ");

            let voice = self.get_voice_for_lang(target_lang, &config);
            let audio_path = output_dir.join(format!("{}.mp3", target_lang));
            tts.synthesize(&full_text, &voice, config.tts.speed, &audio_path)
                .await?;

            let lang_done_progress = 0.1 + ((lang_idx + 1) as f32 / total_langs as f32) * 0.8;
            self.emit_event(
                &app_handle,
                task_id,
                TaskStatus::GeneratingTts,
                lang_done_progress,
                &format!("Completed {} ({}/{})", target_lang, lang_idx + 1, total_langs),
                Some(target_lang),
            );
        }

        // Step 4: Complete
        self.emit_event(&app_handle, task_id, TaskStatus::Completed, 1.0, "Completed", None);
        {
            let mut tasks = self.tasks.lock().await;
            if let Some(t) = tasks.get_mut(task_id) {
                t.status = TaskStatus::Completed;
                t.progress = 1.0;
                t.message = "Completed".to_string();
                t.completed_at = Some(chrono::Utc::now());
            }
        }

        Ok(())
    }

    fn emit_event(
        &self,
        app_handle: &tauri::AppHandle,
        task_id: &str,
        status: TaskStatus,
        progress: f32,
        message: &str,
        current_lang: Option<&str>,
    ) {
        let event = TaskEvent {
            task_id: task_id.to_string(),
            status: status.clone(),
            progress,
            message: message.to_string(),
            current_lang: current_lang.map(String::from),
        };

        // Update internal state
        let tasks = self.tasks.clone();
        let task_id_owned = task_id.to_string();
        let message_owned = message.to_string();
        let current_lang_owned = current_lang.map(String::from);
        tokio::spawn(async move {
            let mut tasks = tasks.lock().await;
            if let Some(t) = tasks.get_mut(&task_id_owned) {
                t.status = status;
                t.progress = progress;
                t.message = message_owned;
                t.current_lang = current_lang_owned;
            }
        });

        let _ = app_handle.emit("task-event", &event);
    }

    async fn get_api_key(&self, provider: &str) -> String {
        // Try to get from OS keychain, fallback to empty string
        let service = format!("subflow_{}", provider);
        match keyring_get(&service) {
            Ok(key) => key,
            Err(_) => String::new(),
        }
    }

    fn get_voice_for_lang(&self, lang: &str, config: &AppConfig) -> String {
        if let Some(voice) = &config.tts.voice {
            return voice.clone();
        }
        // Default voices by language
        match lang {
            "vi" => "vi-VN-HoaiMyNeural".to_string(),
            "ja" | "jp" => "ja-JP-NanamiNeural".to_string(),
            "ko" | "kr" => "ko-KR-SunHiNeural".to_string(),
            "zh" | "cn" => "zh-CN-XiaoxiaoNeural".to_string(),
            "en" => "en-US-AriaNeural".to_string(),
            _ => "en-US-AriaNeural".to_string(),
        }
    }
}

fn keyring_get(service: &str) -> std::result::Result<String, String> {
    // Simple file-based key storage as fallback
    let key_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("subflow")
        .join("keys")
        .join(service);
    std::fs::read_to_string(&key_path).map_err(|e| e.to_string())
}

pub fn keyring_get_pub(service: &str) -> std::result::Result<String, String> {
    keyring_get(service)
}

pub fn keyring_set(service: &str, value: &str) -> std::result::Result<(), String> {
    let key_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("subflow")
        .join("keys");
    std::fs::create_dir_all(&key_dir).map_err(|e| e.to_string())?;
    let key_path = key_dir.join(service);
    std::fs::write(&key_path, value).map_err(|e| e.to_string())
}
