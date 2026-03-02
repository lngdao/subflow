use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::youtube::metadata::VideoMetadata;

#[async_trait]
pub trait SourceProvider: Send + Sync {
    async fn download_subtitle(&self, url: &str, output_dir: &Path, lang: &str) -> Result<PathBuf>;
    async fn get_metadata(&self, url: &str) -> Result<Option<VideoMetadata>>;
    fn can_handle(&self, url: &str) -> bool;
    fn name(&self) -> &str;
}

/// Detect which provider should handle the given URL.
/// YouTube is checked first (direct Innertube API), then generic (yt-dlp) as fallback.
pub fn detect_provider() -> Vec<Box<dyn SourceProvider>> {
    vec![
        Box::new(super::youtube::YouTubeProvider::new()),
        Box::new(super::generic::GenericProvider::new()),
    ]
}

/// Find the appropriate provider for a URL and download subtitle.
pub async fn download_subtitle(url: &str, output_dir: &Path, lang: &str) -> Result<PathBuf> {
    for provider in detect_provider() {
        if provider.can_handle(url) {
            return provider.download_subtitle(url, output_dir, lang).await;
        }
    }
    // Should never happen since GenericProvider.can_handle() always returns true
    Err(crate::error::SubflowError::YouTube(
        "No suitable source provider found".to_string(),
    ))
}

/// Fetch metadata for a URL using the appropriate provider.
pub async fn get_metadata(url: &str) -> Result<Option<VideoMetadata>> {
    for provider in detect_provider() {
        if provider.can_handle(url) {
            return provider.get_metadata(url).await;
        }
    }
    Ok(None)
}
