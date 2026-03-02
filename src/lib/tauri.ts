import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, BinaryStatus, SubtitleFile, Task, VideoMetadata, VoiceInfo } from "./types";

export async function downloadSubtitle(url: string, outputDir: string, lang?: string) {
  return invoke<string>("download_subtitle", { url, outputDir, lang });
}

export async function getVideoMetadata(url: string) {
  return invoke<VideoMetadata | null>("get_video_metadata", { url });
}

export async function parseSubtitleFile(path: string) {
  return invoke<SubtitleFile>("parse_subtitle_file", { path });
}

export async function translateSubtitle(
  subtitleJson: SubtitleFile,
  sourceLang: string,
  targetLang: string,
) {
  return invoke<SubtitleFile>("translate_subtitle", { subtitleJson, sourceLang, targetLang });
}

export async function generateTts(text: string, voice: string, outputPath: string) {
  return invoke<string>("generate_tts", { text, voice, outputPath });
}

export async function listTtsVoices(providerName?: string) {
  return invoke<VoiceInfo[]>("list_tts_voices", { providerName });
}

export async function addTask(
  url?: string,
  filePath?: string,
  sourceLang?: string,
  targetLangs?: string[],
  mode?: string,
) {
  return invoke<string>("add_task", { url, filePath, sourceLang, targetLangs, mode });
}

export async function cancelTask(taskId: string) {
  return invoke<void>("cancel_task", { taskId });
}

export async function pauseTask(taskId: string) {
  return invoke<void>("pause_task", { taskId });
}

export async function resumeTask(taskId: string) {
  return invoke<void>("resume_task", { taskId });
}

export async function retryTask(taskId: string) {
  return invoke<void>("retry_task", { taskId });
}

export async function removeTask(taskId: string) {
  return invoke<void>("remove_task", { taskId });
}

export async function getTasks() {
  return invoke<Task[]>("get_tasks");
}

export async function getSettings() {
  return invoke<AppConfig>("get_settings");
}

export async function saveSettings(config: AppConfig) {
  return invoke<void>("save_settings", { config });
}

export async function saveApiKey(provider: string, apiKey: string) {
  return invoke<void>("save_api_key", { provider, apiKey });
}

export async function getApiKeyPreview(provider: string) {
  return invoke<string | null>("get_api_key_preview", { provider });
}

export async function testProviderConnection(
  provider: string,
  apiKey: string,
  baseUrl?: string,
  model?: string,
) {
  return invoke<boolean>("test_provider_connection", { provider, apiKey, baseUrl, model });
}

export async function setupBinaries() {
  return invoke<BinaryStatus>("setup_binaries");
}

export async function getBinaryStatus() {
  return invoke<BinaryStatus>("get_binary_status");
}

export async function downloadNllbModel() {
  return invoke<void>("download_nllb_model");
}

export async function deleteNllbModel() {
  return invoke<void>("delete_nllb_model");
}
