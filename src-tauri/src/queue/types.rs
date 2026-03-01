use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Queued,
    Downloading,
    Translating,
    GeneratingTts,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingMode {
    SubOnly,
    SubTranslate,
    SubTranslateTts,
}

impl Default for ProcessingMode {
    fn default() -> Self {
        Self::SubTranslateTts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub task_id: String,
    pub status: TaskStatus,
    pub progress: f32,
    pub message: String,
    pub current_lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_title: Option<String>,
}
