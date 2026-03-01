use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

use crate::error::{Result, SubflowError};
use crate::subtitle::types::*;

static SRT_TIMESTAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{2}):(\d{2}):(\d{2})[,.](\d{3})\s*-->\s*(\d{2}):(\d{2}):(\d{2})[,.](\d{3})")
        .unwrap()
});

static SRT_INDEX_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+\s*$").unwrap());

pub fn detect_format(path: &Path) -> Option<SubtitleFormat> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(SubtitleFormat::from_extension)
}

pub fn detect_format_from_content(content: &str) -> SubtitleFormat {
    let trimmed = content.trim();
    if trimmed.starts_with("WEBVTT") {
        SubtitleFormat::Vtt
    } else if SRT_TIMESTAMP_RE.is_match(trimmed) {
        SubtitleFormat::Srt
    } else {
        SubtitleFormat::Txt
    }
}

pub fn parse(content: &str, format: &SubtitleFormat) -> Result<SubtitleFile> {
    match format {
        SubtitleFormat::Srt => parse_srt(content),
        SubtitleFormat::Vtt => parse_vtt(content),
        SubtitleFormat::Txt => parse_txt(content),
    }
}

pub fn parse_auto(content: &str) -> Result<SubtitleFile> {
    let format = detect_format_from_content(content);
    parse(content, &format)
}

fn parse_srt(content: &str) -> Result<SubtitleFile> {
    let mut entries = Vec::new();
    let mut index: u32 = 0;
    let mut current_start: Option<TimeStamp> = None;
    let mut current_end: Option<TimeStamp> = None;
    let mut current_text = String::new();
    let mut in_entry = false;

    for line in content.lines() {
        let line = line.trim_end();

        if line.is_empty() {
            if in_entry {
                index += 1;
                entries.push(SubtitleEntry {
                    index,
                    start: current_start.take(),
                    end: current_end.take(),
                    text: current_text.trim().to_string(),
                });
                current_text.clear();
                in_entry = false;
            }
            continue;
        }

        if let Some(caps) = SRT_TIMESTAMP_RE.captures(line) {
            current_start = Some(TimeStamp::new(
                caps[1].parse().unwrap(),
                caps[2].parse().unwrap(),
                caps[3].parse().unwrap(),
                caps[4].parse().unwrap(),
            ));
            current_end = Some(TimeStamp::new(
                caps[5].parse().unwrap(),
                caps[6].parse().unwrap(),
                caps[7].parse().unwrap(),
                caps[8].parse().unwrap(),
            ));
            in_entry = true;
            continue;
        }

        if SRT_INDEX_RE.is_match(line) && !in_entry {
            continue;
        }

        if in_entry {
            if !current_text.is_empty() {
                current_text.push('\n');
            }
            current_text.push_str(line);
        }
    }

    // Handle last entry without trailing newline
    if in_entry && !current_text.trim().is_empty() {
        index += 1;
        entries.push(SubtitleEntry {
            index,
            start: current_start,
            end: current_end,
            text: current_text.trim().to_string(),
        });
    }

    if entries.is_empty() {
        return Err(SubflowError::SubtitleParse(
            "No subtitle entries found".to_string(),
        ));
    }

    Ok(SubtitleFile::new(SubtitleFormat::Srt, entries))
}

fn parse_vtt(content: &str) -> Result<SubtitleFile> {
    let content = content.trim();
    let content = if let Some(rest) = content.strip_prefix("WEBVTT") {
        // Skip header lines until first blank line
        let mut found_blank = false;
        let mut lines = Vec::new();
        for line in rest.lines() {
            if found_blank {
                lines.push(line);
            } else if line.trim().is_empty() {
                found_blank = true;
            }
        }
        if found_blank {
            lines.join("\n")
        } else {
            rest.to_string()
        }
    } else {
        content.to_string()
    };

    // VTT is very similar to SRT, reuse parsing with minor adjustments
    let srt_result = parse_srt(&content)?;
    Ok(SubtitleFile::new(SubtitleFormat::Vtt, srt_result.entries))
}

fn parse_txt(content: &str) -> Result<SubtitleFile> {
    let entries: Vec<SubtitleEntry> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(i, line)| SubtitleEntry {
            index: (i + 1) as u32,
            start: None,
            end: None,
            text: line.trim().to_string(),
        })
        .collect();

    if entries.is_empty() {
        return Err(SubflowError::SubtitleParse(
            "No text lines found".to_string(),
        ));
    }

    Ok(SubtitleFile::new(SubtitleFormat::Txt, entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_srt() {
        let srt = "1\n00:00:01,000 --> 00:00:04,000\nHello world.\n\n2\n00:00:04,500 --> 00:00:08,000\nThis is a test.\n";
        let file = parse_srt(srt).unwrap();
        assert_eq!(file.entries.len(), 2);
        assert_eq!(file.entries[0].text, "Hello world.");
        assert_eq!(file.entries[0].start.as_ref().unwrap().total_ms(), 1000);
        assert_eq!(file.entries[1].text, "This is a test.");
    }

    #[test]
    fn test_parse_vtt() {
        let vtt = "WEBVTT\n\n00:00:01.000 --> 00:00:04.000\nHello world.\n\n00:00:04.500 --> 00:00:08.000\nThis is a test.\n";
        let file = parse_vtt(vtt).unwrap();
        assert_eq!(file.entries.len(), 2);
        assert_eq!(file.entries[0].text, "Hello world.");
    }

    #[test]
    fn test_parse_txt() {
        let txt = "Hello world.\nThis is a test.\n";
        let file = parse_txt(txt).unwrap();
        assert_eq!(file.entries.len(), 2);
        assert_eq!(file.entries[0].text, "Hello world.");
        assert!(file.entries[0].start.is_none());
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(detect_format_from_content("WEBVTT\n\n"), SubtitleFormat::Vtt);
        assert_eq!(
            detect_format_from_content("1\n00:00:01,000 --> 00:00:04,000\nHello"),
            SubtitleFormat::Srt
        );
        assert_eq!(
            detect_format_from_content("Just some text\n"),
            SubtitleFormat::Txt
        );
    }

    #[test]
    fn test_roundtrip_srt() {
        let srt = "1\n00:00:01,000 --> 00:00:04,000\nHello world.\n\n2\n00:00:04,500 --> 00:00:08,000\nThis is a test.\n";
        let file = parse_srt(srt).unwrap();
        let output = super::super::writer::write(&file);
        let reparsed = parse_srt(&output).unwrap();
        assert_eq!(file.entries.len(), reparsed.entries.len());
        for (a, b) in file.entries.iter().zip(reparsed.entries.iter()) {
            assert_eq!(a.text, b.text);
            assert_eq!(a.start.as_ref().unwrap().total_ms(), b.start.as_ref().unwrap().total_ms());
            assert_eq!(a.end.as_ref().unwrap().total_ms(), b.end.as_ref().unwrap().total_ms());
        }
    }
}
