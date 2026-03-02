use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::source::provider::SourceProvider;
use crate::youtube::metadata::VideoMetadata;

pub struct YouTubeProvider;

impl YouTubeProvider {
    pub fn new() -> Self {
        Self
    }
}

fn is_youtube_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("youtube.com/") || lower.contains("youtu.be/")
}

#[async_trait]
impl SourceProvider for YouTubeProvider {
    async fn download_subtitle(&self, url: &str, output_dir: &Path, lang: &str) -> Result<PathBuf> {
        crate::youtube::downloader::download_subtitle(url, output_dir, lang).await
    }

    async fn get_metadata(&self, url: &str) -> Result<Option<VideoMetadata>> {
        crate::youtube::metadata::get_metadata(url).await.map(Some)
    }

    fn can_handle(&self, url: &str) -> bool {
        is_youtube_url(url)
    }

    fn name(&self) -> &str {
        "YouTube"
    }
}
