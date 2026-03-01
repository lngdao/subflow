use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::AbortHandle;
use tauri::Emitter;

use crate::config::AppConfig;
use crate::error::{Result, SubflowError};
use crate::queue::task::Task;
use crate::queue::types::{ProcessingMode, TaskEvent, TaskStatus};
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
    handles: Arc<Mutex<HashMap<String, AbortHandle>>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(2)),
            cancelled: Arc::new(Mutex::new(std::collections::HashSet::new())),
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, task: Task) -> String {
        let id = task.id.clone();
        self.tasks.lock().await.insert(id.clone(), task);
        id
    }

    pub async fn store_handle(&self, task_id: &str, handle: AbortHandle) {
        self.handles.lock().await.insert(task_id.to_string(), handle);
    }

    pub async fn get_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().await;
        let mut list: Vec<Task> = tasks.values().cloned().collect();
        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        list
    }

    pub async fn tasks_lock(&self) -> tokio::sync::MutexGuard<'_, HashMap<String, Task>> {
        self.tasks.lock().await
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        self.cancelled.lock().await.insert(task_id.to_string());
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            task.message = "Cancelled".to_string();
            drop(tasks);
            // Abort the running tokio task to release semaphore immediately
            if let Some(handle) = self.handles.lock().await.remove(task_id) {
                handle.abort();
            }
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

    pub async fn retry_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if task.status != TaskStatus::Failed {
                return Err(SubflowError::Queue("Only failed tasks can be retried".to_string()));
            }
            task.status = TaskStatus::Queued;
            task.progress = 0.0;
            task.message = "Queued (retry)".to_string();
            task.error = None;
            // Remove from cancelled set
            drop(tasks);
            self.cancelled.lock().await.remove(task_id);
            Ok(())
        } else {
            Err(SubflowError::TaskNotFound(task_id.to_string()))
        }
    }

    pub async fn remove_task(&self, task_id: &str) -> Result<()> {
        // Abort if running
        if let Some(handle) = self.handles.lock().await.remove(task_id) {
            handle.abort();
        }
        self.cancelled.lock().await.insert(task_id.to_string());
        self.tasks.lock().await.remove(task_id)
            .ok_or_else(|| SubflowError::TaskNotFound(task_id.to_string()))?;
        Ok(())
    }

    /// Check if the task has been cancelled. Returns Err(TaskCancelled) if so.
    async fn check_cancelled(&self, task_id: &str) -> Result<()> {
        let cancelled = self.cancelled.lock().await;
        if cancelled.contains(task_id) {
            return Err(SubflowError::TaskCancelled);
        }
        // Also check task status directly
        drop(cancelled);
        let tasks = self.tasks.lock().await;
        if let Some(t) = tasks.get(task_id) {
            if t.status == TaskStatus::Cancelled {
                return Err(SubflowError::TaskCancelled);
            }
        }
        Ok(())
    }

    /// Wait if the task is paused. Returns Err(TaskCancelled) if cancelled while waiting.
    async fn wait_if_paused(&self, task_id: &str) -> Result<()> {
        loop {
            let tasks = self.tasks.lock().await;
            match tasks.get(task_id).map(|t| &t.status) {
                Some(TaskStatus::Paused) => {
                    drop(tasks);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Some(TaskStatus::Cancelled) => return Err(SubflowError::TaskCancelled),
                _ => return Ok(()),
            }
        }
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
        self.emit_event(&app_handle, task_id, TaskStatus::Downloading, 0.05, "Getting subtitle...", None).await;

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

        // Check cancellation + pause
        self.check_cancelled(task_id).await?;
        self.wait_if_paused(task_id).await?;

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

        // SubOnly mode: just save original and finish
        if task.mode == ProcessingMode::SubOnly {
            self.emit_event(&app_handle, task_id, TaskStatus::Completed, 1.0, "Completed", None).await;
            {
                let mut tasks = self.tasks.lock().await;
                if let Some(t) = tasks.get_mut(task_id) {
                    t.status = TaskStatus::Completed;
                    t.progress = 1.0;
                    t.message = "Completed".to_string();
                    t.completed_at = Some(chrono::Utc::now());
                }
            }
            return Ok(());
        }

        let total_langs = task.target_langs.len();
        let api_key = self.get_api_key(&config.translation.provider).await;

        // Step 2: Translate for each target language
        for (lang_idx, target_lang) in task.target_langs.iter().enumerate() {
            // Check cancellation + pause before each language
            self.check_cancelled(task_id).await?;
            self.wait_if_paused(task_id).await?;

            let base_progress = 0.1 + (lang_idx as f32 / total_langs as f32) * 0.7;
            self.emit_event(
                &app_handle,
                task_id,
                TaskStatus::Translating,
                base_progress,
                &format!("Translating to {}...", target_lang),
                Some(target_lang),
            ).await;

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
                // Check cancellation + pause before each chunk
                self.check_cancelled(task_id).await?;
                self.wait_if_paused(task_id).await?;

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
                ).await;
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

            // Check cancellation + pause before TTS
            self.check_cancelled(task_id).await?;
            self.wait_if_paused(task_id).await?;

            // Step 3: Generate TTS (only in SubTranslateTts mode)
            if task.mode == ProcessingMode::SubTranslateTts {
                self.emit_event(
                    &app_handle,
                    task_id,
                    TaskStatus::GeneratingTts,
                    base_progress + 0.7 / total_langs as f32 * 0.6,
                    &format!("Generating TTS for {}...", target_lang),
                    Some(target_lang),
                ).await;

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
            }

            let lang_done_progress = 0.1 + ((lang_idx + 1) as f32 / total_langs as f32) * 0.8;
            self.emit_event(
                &app_handle,
                task_id,
                if task.mode == ProcessingMode::SubTranslateTts { TaskStatus::GeneratingTts } else { TaskStatus::Translating },
                lang_done_progress,
                &format!("Completed {} ({}/{})", target_lang, lang_idx + 1, total_langs),
                Some(target_lang),
            ).await;
        }

        // Step 4: Complete
        self.emit_event(&app_handle, task_id, TaskStatus::Completed, 1.0, "Completed", None).await;
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

    async fn emit_event(
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
            video_title: None,
        };

        // Update internal state synchronously — no tokio::spawn to avoid race conditions
        let mut tasks = self.tasks.lock().await;
        if let Some(t) = tasks.get_mut(task_id) {
            // Don't overwrite Cancelled/Paused status
            if t.status == TaskStatus::Cancelled || t.status == TaskStatus::Paused {
                return;
            }
            t.status = status;
            t.progress = progress;
            t.message = message.to_string();
            t.current_lang = current_lang.map(String::from);
        }
        drop(tasks);

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
        // First check per-language voices map
        if let Some(voice) = config.tts.voices.get(lang) {
            if !voice.is_empty() {
                return voice.clone();
            }
        }
        // Fallback to old single voice field
        if let Some(voice) = &config.tts.voice {
            if !voice.is_empty() {
                return voice.clone();
            }
        }
        // Default voices by language
        match lang {
            "vi" => "vi-VN-HoaiMyNeural".to_string(),
            "ja" | "jp" => "ja-JP-NanamiNeural".to_string(),
            "ko" | "kr" => "ko-KR-SunHiNeural".to_string(),
            "zh" | "cn" => "zh-CN-XiaoxiaoNeural".to_string(),
            "es" => "es-ES-ElviraNeural".to_string(),
            "fr" => "fr-FR-DeniseNeural".to_string(),
            "de" => "de-DE-KatjaNeural".to_string(),
            "pt" => "pt-BR-FranciscaNeural".to_string(),
            "ru" => "ru-RU-SvetlanaNeural".to_string(),
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
