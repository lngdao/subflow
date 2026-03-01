use tauri::State;

use crate::config::AppConfig;
use crate::error::SubflowError;
use crate::queue::orchestrator::Orchestrator;
use crate::queue::task::Task;

#[tauri::command]
pub async fn add_task(
    orchestrator: State<'_, Orchestrator>,
    app_handle: tauri::AppHandle,
    url: Option<String>,
    file_path: Option<String>,
    source_lang: Option<String>,
    target_langs: Option<Vec<String>>,
) -> Result<String, SubflowError> {
    let config = AppConfig::load()?;
    let source_lang = source_lang.unwrap_or(config.translation.source_lang);
    let target_langs = target_langs.unwrap_or(config.translation.target_langs);

    let task = if let Some(url) = url {
        let mut task = Task::new_from_url(&url, &source_lang, target_langs);
        if let Ok(meta) = crate::youtube::metadata::get_metadata(&url).await {
            task.video_title = Some(meta.title);
            task.video_id = Some(meta.id);
        }
        task
    } else if let Some(path) = file_path {
        Task::new_from_file(&path, &source_lang, target_langs)
    } else {
        return Err(SubflowError::Queue(
            "Either url or file_path must be provided".to_string(),
        ));
    };

    let task_id = orchestrator.add_task(task).await;

    let orch = orchestrator.inner().clone();
    let tid = task_id.clone();
    tokio::spawn(async move {
        if let Err(e) = orch.process_task(&tid, app_handle).await {
            tracing::error!("Task {} failed: {}", tid, e);
        }
    });

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
pub async fn get_tasks(
    orchestrator: State<'_, Orchestrator>,
) -> Result<Vec<Task>, SubflowError> {
    Ok(orchestrator.get_tasks().await)
}
