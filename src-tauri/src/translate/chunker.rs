use crate::subtitle::types::SubtitleEntry;

const MAX_ENTRIES_PER_CHUNK: usize = 50;

pub struct Chunk {
    pub entries: Vec<(usize, String)>, // (original_index, text)
}

pub fn chunk_entries(entries: &[SubtitleEntry], max_per_chunk: Option<usize>) -> Vec<Chunk> {
    let max = max_per_chunk.unwrap_or(MAX_ENTRIES_PER_CHUNK);
    entries
        .chunks(max)
        .map(|chunk| Chunk {
            entries: chunk
                .iter()
                .map(|e| (e.index as usize, e.text.clone()))
                .collect(),
        })
        .collect()
}

pub fn build_prompt(texts: &[String], source_lang: &str, target_lang: &str) -> String {
    let numbered_lines: Vec<String> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| format!("{}. {}", i + 1, text))
        .collect();

    let source_instruction = if source_lang == "auto" {
        "the auto-detected source language".to_string()
    } else {
        source_lang.to_string()
    };

    format!(
        "You are a professional subtitle translator. Translate the following subtitles from {} to {}.\n\n\
        IMPORTANT RULES:\n\
        1. Keep the same number of lines as input\n\
        2. Each line corresponds to one subtitle segment\n\
        3. Maintain natural speech patterns in {}\n\
        4. Keep timing-appropriate length (don't make subtitles too long)\n\
        5. Preserve any formatting markers like [Music], [Applause], etc.\n\
        6. Output ONLY the translated lines, one per line, numbered to match input.\n\n\
        Input subtitles:\n{}\n\n\
        Output ONLY the translated lines, numbered to match:",
        source_instruction,
        target_lang,
        target_lang,
        numbered_lines.join("\n")
    )
}

pub fn parse_response(response: &str, expected_count: usize) -> Vec<String> {
    let lines: Vec<String> = response
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            // Strip leading number and dot/period: "1. translated text" -> "translated text"
            let trimmed = line.trim();
            if let Some(pos) = trimmed.find(". ") {
                let prefix = &trimmed[..pos];
                if prefix.parse::<usize>().is_ok() {
                    return trimmed[pos + 2..].to_string();
                }
            }
            trimmed.to_string()
        })
        .collect();

    // If count matches, return as-is
    if lines.len() == expected_count {
        return lines;
    }

    // If we got fewer, pad with empty strings
    if lines.len() < expected_count {
        let mut result = lines;
        result.resize(expected_count, String::new());
        return result;
    }

    // If we got more, truncate
    lines[..expected_count].to_vec()
}
