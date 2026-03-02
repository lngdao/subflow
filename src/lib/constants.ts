export const LANGUAGES = [
  { code: "auto", name: "Auto Detect" },
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
  { id: "libretranslate", name: "LibreTranslate", hasApiKey: false, hasBaseUrl: true, hasModel: false },
  { id: "nllb", name: "NLLB-200 (Local)", hasApiKey: false, hasBaseUrl: false, hasModel: false },
  { id: "nllb_api", name: "NLLB-200 (Server)", hasApiKey: false, hasBaseUrl: true, hasModel: false },
  { id: "openai_compatible", name: "OpenAI Compatible", hasApiKey: true, hasBaseUrl: true, hasModel: true },
  { id: "anthropic", name: "Anthropic Messages", hasApiKey: true, hasBaseUrl: true, hasModel: true },
] as const;

export const TTS_PROVIDERS = [
  { id: "edge", name: "Edge TTS", hasApiKey: false },
  { id: "openai", name: "OpenAI TTS", hasApiKey: true },
] as const;

export const OUTPUT_FORMATS = [
  { id: "srt", name: "SRT" },
  { id: "vtt", name: "VTT" },
  { id: "txt", name: "TXT" },
] as const;
