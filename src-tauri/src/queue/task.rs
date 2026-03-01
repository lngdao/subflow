use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::queue::types::{ProcessingMode, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub video_title: Option<String>,
    pub video_id: Option<String>,
    pub source_lang: String,
    pub target_langs: Vec<String>,
    pub mode: ProcessingMode,
    pub status: TaskStatus,
    pub progress: f32,
    pub message: String,
    pub current_lang: Option<String>,
    pub output_dir: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl Task {
    pub fn new_from_url(url: &str, source_lang: &str, target_langs: Vec<String>, mode: ProcessingMode) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url: Some(url.to_string()),
            file_path: None,
            video_title: None,
            video_id: None,
            source_lang: source_lang.to_string(),
            target_langs,
            mode,
            status: TaskStatus::Queued,
            progress: 0.0,
            message: "Queued".to_string(),
            current_lang: None,
            output_dir: None,
            created_at: Utc::now(),
            completed_at: None,
            error: None,
        }
    }

    pub fn new_from_file(
        file_path: &str,
        source_lang: &str,
        target_langs: Vec<String>,
        mode: ProcessingMode,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url: None,
            file_path: Some(file_path.to_string()),
            video_title: None,
            video_id: None,
            source_lang: source_lang.to_string(),
            target_langs,
            mode,
            status: TaskStatus::Queued,
            progress: 0.0,
            message: "Queued".to_string(),
            current_lang: None,
            output_dir: None,
            created_at: Utc::now(),
            completed_at: None,
            error: None,
        }
    }
}
