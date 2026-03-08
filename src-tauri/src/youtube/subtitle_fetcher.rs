use std::path::{Path, PathBuf};

use regex::Regex;
use reqwest::Client;
use serde_json::json;

use crate::error::{Result, SubflowError};

/// Directly fetch subtitles from YouTube's Innertube API (like downsub.com).
/// Much faster and more reliable than yt-dlp for subtitle-only downloads.
pub async fn fetch_subtitle_direct(
    video_id: &str,
    output_dir: &Path,
    sub_lang: &str,
) -> Result<PathBuf> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| SubflowError::YouTube(format!("HTTP client error: {}", e)))?;

    // Use Innertube API to get player response with fresh caption URLs
    let caption_tracks = get_caption_tracks_innertube(&client, video_id).await?;

    let effective_lang = if sub_lang == "auto" { "en" } else { sub_lang };

    // Find the best matching caption track
    let track_url = find_best_track(&caption_tracks, effective_lang)?;

    // Fetch subtitle content - try json3 first, detect XML fallback
    let json3_url = if track_url.contains('?') {
        format!("{}&fmt=json3", track_url)
    } else {
        format!("{}?fmt=json3", track_url)
    };

    tracing::debug!("Fetching subtitle from: {}", json3_url);

    let response = client
        .get(&json3_url)
        .send()
        .await
        .map_err(|e| SubflowError::YouTube(format!("Failed to fetch subtitle: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| SubflowError::YouTube(format!("Failed to read subtitle: {}", e)))?;

    if !status.is_success() {
        return Err(SubflowError::YouTube(format!(
            "Subtitle fetch returned HTTP {}: {}",
            status,
            &body[..body.len().min(200)]
        )));
    }

    if body.trim().is_empty() {
        return Err(SubflowError::YouTube(
            "Subtitle response is empty".to_string(),
        ));
    }

    // Detect format and convert to SRT
    let trimmed = body.trim();
    let srt_content = if trimmed.starts_with('{') {
        // JSON format
        json3_to_srt(&body)?
    } else if trimmed.starts_with('<') {
        // XML format (srv3) - YouTube returned XML despite requesting json3
        tracing::debug!("Got XML response, parsing as srv3");
        srv3_to_srt(&body)?
    } else {
        return Err(SubflowError::YouTube(format!(
            "Unknown subtitle format (first 100 chars: {})",
            &trimmed[..trimmed.len().min(100)]
        )));
    };

    // Write to file
    std::fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join(format!("{}.{}.srt", video_id, effective_lang));
    std::fs::write(&output_path, &srt_content)?;

    Ok(output_path)
}

#[derive(Debug)]
struct CaptionTrack {
    base_url: String,
    lang_code: String,
    kind: Option<String>, // "asr" for auto-generated
}

/// Get caption tracks via YouTube Innertube API (POST request).
/// Uses ANDROID client which reliably returns caption track URLs.
async fn get_caption_tracks_innertube(
    client: &Client,
    video_id: &str,
) -> Result<Vec<CaptionTrack>> {
    let innertube_url = "https://www.youtube.com/youtubei/v1/player";

    let payload = json!({
        "videoId": video_id,
        "context": {
            "client": {
                "clientName": "ANDROID",
                "clientVersion": "20.10.38",
                "hl": "en",
                "gl": "US"
            }
        }
    });

    let response = client
        .post(innertube_url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| SubflowError::YouTube(format!("Innertube API request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(SubflowError::YouTube(format!(
            "Innertube API returned HTTP {}",
            response.status()
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| SubflowError::YouTube(format!("Failed to parse Innertube response: {}", e)))?;

    let tracks = json
        .pointer("/captions/playerCaptionsTracklistRenderer/captionTracks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            SubflowError::YouTube("No caption tracks found for this video".to_string())
        })?;

    let mut result = Vec::new();
    for track in tracks {
        if let (Some(base_url), Some(lang_code)) = (
            track["baseUrl"].as_str(),
            track["languageCode"].as_str(),
        ) {
            result.push(CaptionTrack {
                base_url: base_url.to_string(),
                lang_code: lang_code.to_string(),
                kind: track["kind"].as_str().map(String::from),
            });
        }
    }

    if result.is_empty() {
        return Err(SubflowError::YouTube(
            "No caption tracks found for this video".to_string(),
        ));
    }

    tracing::debug!(
        "Found {} caption tracks: {:?}",
        result.len(),
        result.iter().map(|t| format!("{}({})", t.lang_code, t.kind.as_deref().unwrap_or("manual"))).collect::<Vec<_>>()
    );

    Ok(result)
}

fn find_best_track(tracks: &[CaptionTrack], lang: &str) -> Result<String> {
    // Priority: exact manual match > exact auto match > prefix manual > prefix auto > any
    let exact_manual = tracks
        .iter()
        .find(|t| t.lang_code == lang && t.kind.is_none());
    if let Some(t) = exact_manual {
        return Ok(t.base_url.clone());
    }

    let exact_auto = tracks
        .iter()
        .find(|t| t.lang_code == lang && t.kind.as_deref() == Some("asr"));
    if let Some(t) = exact_auto {
        return Ok(t.base_url.clone());
    }

    // Prefix match (e.g. "en" matches "en-US")
    let prefix_manual = tracks
        .iter()
        .find(|t| t.lang_code.starts_with(lang) && t.kind.is_none());
    if let Some(t) = prefix_manual {
        return Ok(t.base_url.clone());
    }

    let prefix_auto = tracks
        .iter()
        .find(|t| t.lang_code.starts_with(lang) && t.kind.as_deref() == Some("asr"));
    if let Some(t) = prefix_auto {
        return Ok(t.base_url.clone());
    }

    // Fall back to first available track
    tracks
        .first()
        .map(|t| t.base_url.clone())
        .ok_or_else(|| SubflowError::YouTube("No caption tracks available".to_string()))
}

/// Convert YouTube json3 subtitle format to SRT
fn json3_to_srt(json_str: &str) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        SubflowError::YouTube(format!(
            "Failed to parse subtitle JSON: {} (first 200 chars: {})",
            e,
            &json_str[..json_str.len().min(200)]
        ))
    })?;

    let events = json["events"]
        .as_array()
        .ok_or_else(|| SubflowError::YouTube("No events in subtitle JSON".to_string()))?;

    let mut srt = String::new();
    let mut index = 1u32;

    for event in events {
        // Skip events without segments (e.g. newline/format events)
        let segs = match event["segs"].as_array() {
            Some(s) if !s.is_empty() => s,
            _ => continue,
        };

        let start_ms = event["tStartMs"].as_u64().unwrap_or(0);
        let dur_ms = event["dDurationMs"].as_u64().unwrap_or(0);
        if dur_ms == 0 {
            continue;
        }
        let end_ms = start_ms + dur_ms;

        // Combine all segments into one line
        let text: String = segs
            .iter()
            .filter_map(|seg| seg["utf8"].as_str())
            .collect::<Vec<_>>()
            .join("");

        let text = text.trim();
        if text.is_empty() || text == "\n" {
            continue;
        }

        srt.push_str(&format!("{}\n", index));
        srt.push_str(&format!(
            "{} --> {}\n",
            format_srt_time(start_ms),
            format_srt_time(end_ms)
        ));
        srt.push_str(&format!("{}\n\n", text));
        index += 1;
    }

    if srt.is_empty() {
        return Err(SubflowError::YouTube(
            "No subtitle content found in response".to_string(),
        ));
    }

    Ok(srt)
}

/// Convert YouTube srv3 XML subtitle format to SRT.
/// Handles both <p t="..." d="..."> and <w t="..." d="..."> elements,
/// plus nested <s> segment elements.
fn srv3_to_srt(xml: &str) -> Result<String> {
    let mut srt = String::new();
    let mut index = 1u32;

    // Try <p> elements first (common srv3 format)
    // Pattern: <p t="start_ms" d="dur_ms" ...>text</p>
    let p_re = Regex::new(r#"<p\s[^>]*?t="(\d+)"[^>]*?d="(\d+)"[^>]*?>(.*?)</p>"#)
        .map_err(|e| SubflowError::YouTube(format!("Regex error: {}", e)))?;

    for cap in p_re.captures_iter(xml) {
        let start_ms: u64 = cap[1].parse().unwrap_or(0);
        let dur_ms: u64 = cap[2].parse().unwrap_or(0);
        if dur_ms == 0 {
            continue;
        }
        let end_ms = start_ms + dur_ms;

        let text = strip_xml_tags(&cap[3]);
        let text = html_decode(text.trim());
        if text.is_empty() {
            continue;
        }

        srt.push_str(&format!("{}\n", index));
        srt.push_str(&format!(
            "{} --> {}\n",
            format_srt_time(start_ms),
            format_srt_time(end_ms)
        ));
        srt.push_str(&format!("{}\n\n", text));
        index += 1;
    }

    // If no <p> elements, try <w> elements (newer srv3 format="3")
    if srt.is_empty() {
        let w_re = Regex::new(r#"<w\s[^>]*?t="(\d+)"[^>]*?d="(\d+)"[^>]*?>([\s\S]*?)</w>"#)
            .map_err(|e| SubflowError::YouTube(format!("Regex error: {}", e)))?;

        for cap in w_re.captures_iter(xml) {
            let start_ms: u64 = cap[1].parse().unwrap_or(0);
            let dur_ms: u64 = cap[2].parse().unwrap_or(0);
            if dur_ms == 0 {
                continue;
            }
            let end_ms = start_ms + dur_ms;

            let text = strip_xml_tags(&cap[3]);
            let text = html_decode(text.trim());
            if text.is_empty() {
                continue;
            }

            srt.push_str(&format!("{}\n", index));
            srt.push_str(&format!(
                "{} --> {}\n",
                format_srt_time(start_ms),
                format_srt_time(end_ms)
            ));
            srt.push_str(&format!("{}\n\n", text));
            index += 1;
        }
    }

    // Last resort: try <text> elements (srv1 format)
    if srt.is_empty() {
        let text_re =
            Regex::new(r#"<text\s[^>]*?start="([^"]+)"[^>]*?dur="([^"]+)"[^>]*?>(.*?)</text>"#)
                .map_err(|e| SubflowError::YouTube(format!("Regex error: {}", e)))?;

        for cap in text_re.captures_iter(xml) {
            let start_sec: f64 = cap[1].parse().unwrap_or(0.0);
            let dur_sec: f64 = cap[2].parse().unwrap_or(0.0);
            if dur_sec == 0.0 {
                continue;
            }
            let start_ms = (start_sec * 1000.0) as u64;
            let end_ms = ((start_sec + dur_sec) * 1000.0) as u64;

            let text = strip_xml_tags(&cap[3]);
            let text = html_decode(text.trim());
            if text.is_empty() {
                continue;
            }

            srt.push_str(&format!("{}\n", index));
            srt.push_str(&format!(
                "{} --> {}\n",
                format_srt_time(start_ms),
                format_srt_time(end_ms)
            ));
            srt.push_str(&format!("{}\n\n", text));
            index += 1;
        }
    }

    if srt.is_empty() {
        return Err(SubflowError::YouTube(
            "No subtitle entries found in XML".to_string(),
        ));
    }

    Ok(srt)
}

/// Strip XML/HTML tags from text, keeping only text content
fn strip_xml_tags(s: &str) -> String {
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    tag_re.replace_all(s, "").to_string()
}

fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn format_srt_time(ms: u64) -> String {
    let millis = ms % 1000;
    let total_s = ms / 1000;
    let s = total_s % 60;
    let total_m = total_s / 60;
    let m = total_m % 60;
    let h = total_m / 60;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_srt_time() {
        assert_eq!(format_srt_time(0), "00:00:00,000");
        assert_eq!(format_srt_time(1500), "00:00:01,500");
        assert_eq!(format_srt_time(61123), "00:01:01,123");
        assert_eq!(format_srt_time(3661500), "01:01:01,500");
    }

    #[test]
    fn test_json3_to_srt() {
        let json = r#"{"events":[{"tStartMs":500,"dDurationMs":2000,"segs":[{"utf8":"Hello world"}]},{"tStartMs":3000,"dDurationMs":1500,"segs":[{"utf8":"Second line"}]}]}"#;
        let srt = json3_to_srt(json).unwrap();
        assert!(srt.contains("00:00:00,500 --> 00:00:02,500"));
        assert!(srt.contains("Hello world"));
        assert!(srt.contains("00:00:03,000 --> 00:00:04,500"));
        assert!(srt.contains("Second line"));
    }

    #[test]
    fn test_json3_to_srt_multi_segment() {
        let json = r#"{"events":[{"tStartMs":0,"dDurationMs":1000,"segs":[{"utf8":"Hello "},{"utf8":"world"}]}]}"#;
        let srt = json3_to_srt(json).unwrap();
        assert!(srt.contains("Hello world"));
    }

    #[test]
    fn test_json3_to_srt_skips_empty() {
        let json = r#"{"events":[{"tStartMs":0,"dDurationMs":1000,"segs":[{"utf8":"\n"}]},{"tStartMs":1000,"dDurationMs":500,"segs":[{"utf8":"Real text"}]}]}"#;
        let srt = json3_to_srt(json).unwrap();
        assert!(!srt.contains("00:00:00,000"));
        assert!(srt.contains("Real text"));
    }

    #[test]
    fn test_srv3_p_elements() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" ?><timedtext><body><p t="500" d="2000">Hello world</p><p t="3000" d="1500">Second &amp; line</p></body></timedtext>"#;
        let srt = srv3_to_srt(xml).unwrap();
        assert!(srt.contains("00:00:00,500 --> 00:00:02,500"));
        assert!(srt.contains("Hello world"));
        assert!(srt.contains("Second & line"));
    }

    #[test]
    fn test_srv3_w_elements() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" ?><timedtext format="3"><body><w t="0" d="2000"><s>Hello</s></w><w t="2500" d="1500"><s>World</s></w></body></timedtext>"#;
        let srt = srv3_to_srt(xml).unwrap();
        assert!(srt.contains("00:00:00,000 --> 00:00:02,000"));
        assert!(srt.contains("Hello"));
        assert!(srt.contains("World"));
    }

    #[test]
    fn test_srv3_text_elements() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" ?><timedtext><body><text start="0.5" dur="2.0">Hello world</text><text start="3.0" dur="1.5">Second line</text></body></timedtext>"#;
        let srt = srv3_to_srt(xml).unwrap();
        assert!(srt.contains("00:00:00,500 --> 00:00:02,500"));
        assert!(srt.contains("Hello world"));
    }

    #[test]
    fn test_strip_xml_tags() {
        assert_eq!(strip_xml_tags("<s>hello</s> <s>world</s>"), "hello world");
        assert_eq!(strip_xml_tags("no tags"), "no tags");
    }

    #[test]
    fn test_html_decode() {
        assert_eq!(html_decode("hello &amp; world"), "hello & world");
        assert_eq!(html_decode("it&#39;s"), "it's");
    }
}
