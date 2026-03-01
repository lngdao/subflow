use crate::subtitle::types::*;

pub fn write(file: &SubtitleFile) -> String {
    match file.format {
        SubtitleFormat::Srt => write_srt(file),
        SubtitleFormat::Vtt => write_vtt(file),
        SubtitleFormat::Txt => write_txt(file),
    }
}

pub fn write_as(file: &SubtitleFile, format: &SubtitleFormat) -> String {
    match format {
        SubtitleFormat::Srt => write_srt(file),
        SubtitleFormat::Vtt => write_vtt(file),
        SubtitleFormat::Txt => write_txt(file),
    }
}

fn write_srt(file: &SubtitleFile) -> String {
    let mut output = String::new();
    for entry in &file.entries {
        output.push_str(&entry.index.to_string());
        output.push('\n');
        if let (Some(start), Some(end)) = (&entry.start, &entry.end) {
            output.push_str(&format!("{} --> {}", start.to_srt_string(), end.to_srt_string()));
        }
        output.push('\n');
        output.push_str(&entry.text);
        output.push_str("\n\n");
    }
    output
}

fn write_vtt(file: &SubtitleFile) -> String {
    let mut output = String::from("WEBVTT\n\n");
    for entry in &file.entries {
        if let (Some(start), Some(end)) = (&entry.start, &entry.end) {
            output.push_str(&format!("{} --> {}", start.to_vtt_string(), end.to_vtt_string()));
        }
        output.push('\n');
        output.push_str(&entry.text);
        output.push_str("\n\n");
    }
    output
}

fn write_txt(file: &SubtitleFile) -> String {
    file.entries
        .iter()
        .map(|e| e.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
