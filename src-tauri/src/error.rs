use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubflowError {
    #[error("Subtitle parse error: {0}")]
    SubtitleParse(String),

    #[error("YouTube error: {0}")]
    YouTube(String),

    #[error("Translation error: {0}")]
    Translation(String),

    #[error("TTS error: {0}")]
    Tts(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Queue error: {0}")]
    Queue(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("yt-dlp not found. Please install yt-dlp: https://github.com/yt-dlp/yt-dlp")]
    YtDlpNotFound,

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Task cancelled")]
    TaskCancelled,
}

impl Serialize for SubflowError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SubflowError>;
