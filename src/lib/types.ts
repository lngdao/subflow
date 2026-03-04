export interface TimeStamp {
  hours: number;
  minutes: number;
  seconds: number;
  milliseconds: number;
}

export interface SubtitleEntry {
  index: number;
  start: TimeStamp | null;
  end: TimeStamp | null;
  text: string;
}

export interface SubtitleFile {
  format: "Srt" | "Vtt" | "Txt";
  entries: SubtitleEntry[];
}

export interface VideoMetadata {
  id: string;
  title: string;
  duration: number | null;
  thumbnail: string | null;
  channel: string | null;
  upload_date: string | null;
}

export type TaskStatus =
  | "Queued"
  | "Downloading"
  | "Translating"
  | "GeneratingTts"
  | "Completed"
  | "Failed"
  | "Cancelled"
  | "Paused";

export interface TaskEvent {
  task_id: string;
  status: TaskStatus;
  progress: number;
  message: string;
  current_lang: string | null;
  video_title?: string | null;
}

export type ProcessingMode = "SubOnly" | "SubTranslate" | "SubTranslateTts";

export interface Task {
  id: string;
  url: string | null;
  file_path: string | null;
  video_title: string | null;
  video_id: string | null;
  source_lang: string;
  target_langs: string[];
  mode: ProcessingMode;
  status: TaskStatus;
  progress: number;
  message: string;
  current_lang: string | null;
  output_dir: string | null;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  error: string | null;
}

export interface TranslationConfig {
  provider: string;
  base_url: string | null;
  model: string | null;
  source_lang: string;
  target_langs: string[];
}

export interface TtsConfig {
  provider: string;
  voices: Record<string, string>;
  voice?: string | null;
  speed: number;
}

export interface OutputConfig {
  format: string;
  folder: string;
}

export interface QueueConfig {
  parallel_jobs: number;
  parallel_langs: number;
  pipeline_tts: boolean;
  tts_chunk_size: number;
}

export interface NotificationConfig {
  enabled: boolean;
}

export interface AppConfig {
  translation: TranslationConfig;
  tts: TtsConfig;
  output: OutputConfig;
  queue: QueueConfig;
  notifications: NotificationConfig;
}

export interface VoiceInfo {
  id: string;
  name: string;
  language: string;
  gender: string | null;
}

export interface BinaryStatus {
  ytdlp_available: boolean;
  ffmpeg_available: boolean;
  ytdlp_path: string | null;
  ffmpeg_path: string | null;
  nllb_600m_available: boolean;
  nllb_600m_path: string | null;
  nllb_1_3b_available: boolean;
  nllb_1_3b_path: string | null;
}

export interface ModelDownloadProgress {
  model: string;
  file: string;
  bytes_downloaded: number;
  bytes_total: number | null;
  percent: number;
  status: "downloading" | "completed" | "failed";
}
