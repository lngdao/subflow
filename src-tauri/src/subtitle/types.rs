use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubtitleFormat {
    Srt,
    Vtt,
    Txt,
}

impl SubtitleFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "srt" => Some(Self::Srt),
            "vtt" => Some(Self::Vtt),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Srt => "srt",
            Self::Vtt => "vtt",
            Self::Txt => "txt",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeStamp {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub milliseconds: u32,
}

impl TimeStamp {
    pub fn new(hours: u32, minutes: u32, seconds: u32, milliseconds: u32) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }

    pub fn total_ms(&self) -> u64 {
        (self.hours as u64 * 3_600_000)
            + (self.minutes as u64 * 60_000)
            + (self.seconds as u64 * 1_000)
            + self.milliseconds as u64
    }

    pub fn from_ms(ms: u64) -> Self {
        Self {
            hours: (ms / 3_600_000) as u32,
            minutes: ((ms % 3_600_000) / 60_000) as u32,
            seconds: ((ms % 60_000) / 1_000) as u32,
            milliseconds: (ms % 1_000) as u32,
        }
    }

    pub fn to_srt_string(&self) -> String {
        format!(
            "{:02}:{:02}:{:02},{:03}",
            self.hours, self.minutes, self.seconds, self.milliseconds
        )
    }

    pub fn to_vtt_string(&self) -> String {
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            self.hours, self.minutes, self.seconds, self.milliseconds
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleEntry {
    pub index: u32,
    pub start: Option<TimeStamp>,
    pub end: Option<TimeStamp>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleFile {
    pub format: SubtitleFormat,
    pub entries: Vec<SubtitleEntry>,
}

impl SubtitleFile {
    pub fn new(format: SubtitleFormat, entries: Vec<SubtitleEntry>) -> Self {
        Self { format, entries }
    }
}
