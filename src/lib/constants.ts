export const LANGUAGES = [
  { code: "vi", name: "Vietnamese" },
  { code: "ja", name: "Japanese" },
  { code: "ko", name: "Korean" },
  { code: "zh", name: "Chinese" },
  { code: "es", name: "Spanish" },
  { code: "fr", name: "French" },
  { code: "de", name: "German" },
  { code: "pt", name: "Portuguese" },
  { code: "ru", name: "Russian" },
  { code: "en", name: "English" },
] as const;

export const TRANSLATION_PROVIDERS = [
  { id: "claude", name: "Claude Haiku", hasApiKey: true, hasBaseUrl: true, hasModel: true },
  { id: "glm", name: "GLM-5 (z.ai)", hasApiKey: true, hasBaseUrl: true, hasModel: true },
  { id: "openai", name: "OpenAI GPT-4o", hasApiKey: true, hasBaseUrl: true, hasModel: true },
  { id: "gemini", name: "Gemini Flash", hasApiKey: true, hasBaseUrl: false, hasModel: true },
  { id: "deepl", name: "DeepL", hasApiKey: true, hasBaseUrl: false, hasModel: false },
] as const;

export const TTS_PROVIDERS = [
  { id: "edge", name: "Edge TTS (Free)", hasApiKey: false },
  { id: "openai", name: "OpenAI TTS", hasApiKey: true },
] as const;

export const OUTPUT_FORMATS = [
  { id: "srt", name: "SRT" },
  { id: "vtt", name: "VTT" },
  { id: "txt", name: "TXT" },
] as const;

export const YOUTUBE_URL_REGEX =
  /(?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/(?:watch\?v=|shorts\/)|youtu\.be\/)([a-zA-Z0-9_-]{11})/;
