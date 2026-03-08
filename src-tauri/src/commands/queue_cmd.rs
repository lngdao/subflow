use tauri::State;

use crate::config::AppConfig;
use crate::error::SubflowError;
use crate::queue::orchestrator::Orchestrator;
use crate::queue::task::Task;
use crate::queue::types::ProcessingMode;

/// Spawn a task for background processing and store its abort handle.
fn spawn_task(orch: Orchestrator, task_id: String, app_handle: tauri::AppHandle, fetch_meta_url: Option<String>) {
    let tid = task_id.clone();
    let orch_clone = orch.clone();
    let handle = tokio::spawn(async move {
        // Fetch metadata in background (don't block the command response)
        if let Some(url) = fetch_meta_url {
            if let Ok(Some(meta)) = crate::source::provider::get_metadata(&url).await {
                let title = meta.title.clone();
                let mut tasks = orch.tasks_lock().await;
                if let Some(t) = tasks.get_mut(&tid) {
                    t.video_title = Some(meta.title);
                    t.video_id = Some(meta.id);
                }
                drop(tasks);
                // Notify frontend of title update
                let _ = tauri::Emitter::emit(&app_handle, "task-event", &crate::queue::types::TaskEvent {
                    task_id: tid.clone(),
                    status: crate::queue::types::TaskStatus::Queued,
                    progress: 0.0,
                    message: "Queued".to_string(),
                    current_lang: None,
                    video_title: Some(title),
                });
            }
        }

        if let Err(e) = orch.process_task(&tid, app_handle.clone()).await {
            match &e {
                crate::error::SubflowError::TaskCancelled => {
                    tracing::info!("Task {} was cancelled", tid);
                }
                _ => {
                    tracing::error!("Task {} failed: {}", tid, e);
                    let mut tasks = orch.tasks_lock().await;
                    if let Some(t) = tasks.get_mut(&tid) {
                        t.status = crate::queue::types::TaskStatus::Failed;
                        t.error = Some(e.to_string());
                        t.message = "Failed".to_string();
                    }
                    drop(tasks);
                    let _ = tauri::Emitter::emit(&app_handle, "task-event", &crate::queue::types::TaskEvent {
                        task_id: tid.clone(),
                        status: crate::queue::types::TaskStatus::Failed,
                        progress: 0.0,
                        message: e.to_string(),
                        current_lang: None,
                        video_title: None,
                    });
                }
            }
        }
    });

    // Store abort handle so cancel/remove can abort immediately
    let abort_handle = handle.abort_handle();
    tokio::spawn(async move {
        orch_clone.store_handle(&task_id, abort_handle).await;
    });
}

#[tauri::command]
pub async fn add_task(
    orchestrator: State<'_, Orchestrator>,
    app_handle: tauri::AppHandle,
    url: Option<String>,
    file_path: Option<String>,
    source_lang: Option<String>,
    target_langs: Option<Vec<String>>,
    mode: Option<String>,
    use_yt_translation: Option<bool>,
) -> Result<String, SubflowError> {
    let config = AppConfig::load()?;
    let source_lang = source_lang.unwrap_or(config.translation.source_lang);
    let target_langs = target_langs.unwrap_or(config.translation.target_langs);
    let processing_mode = match mode.as_deref() {
        Some("sub_only") => ProcessingMode::SubOnly,
        Some("sub_translate") => ProcessingMode::SubTranslate,
        _ => ProcessingMode::SubTranslateTts,
    };

    let task = if let Some(ref url) = url {
        Task::new_from_url(url, &source_lang, target_langs, processing_mode, use_yt_translation.unwrap_or(false))
    } else if let Some(path) = file_path {
        Task::new_from_file(&path, &source_lang, target_langs, processing_mode)
    } else {
        return Err(SubflowError::Queue(
            "Either url or file_path must be provided".to_string(),
        ));
    };

    let task_id = orchestrator.add_task(task).await;
    let orch = orchestrator.inner().clone();
    spawn_task(orch, task_id.clone(), app_handle, url);

    Ok(task_id)
}

#[tauri::command]
pub async fn cancel_task(
    orchestrator: State<'_, Orchestrator>,
    task_id: String,
) -> Result<(), SubflowError> {
    orchestrator.cancel_task(&task_id).await
}

#[tauri::command]
pub async fn pause_task(
    orchestrator: State<'_, Orchestrator>,
    task_id: String,
) -> Result<(), SubflowError> {
    orchestrator.pause_task(&task_id).await
}

#[tauri::command]
pub async fn resume_task(
    orchestrator: State<'_, Orchestrator>,
    task_id: String,
) -> Result<(), SubflowError> {
    orchestrator.resume_task(&task_id).await
}

#[tauri::command]
pub async fn retry_task(
    orchestrator: State<'_, Orchestrator>,
    app_handle: tauri::AppHandle,
    task_id: String,
) -> Result<(), SubflowError> {
    orchestrator.retry_task(&task_id).await?;
    let orch = orchestrator.inner().clone();
    spawn_task(orch, task_id, app_handle, None);
    Ok(())
}

#[tauri::command]
pub async fn remove_task(
    orchestrator: State<'_, Orchestrator>,
    task_id: String,
) -> Result<(), SubflowError> {
    orchestrator.remove_task(&task_id).await
}

#[tauri::command]
pub async fn get_tasks(
    orchestrator: State<'_, Orchestrator>,
) -> Result<Vec<Task>, SubflowError> {
    Ok(orchestrator.get_tasks().await)
}
