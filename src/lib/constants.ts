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

export interface TranslationProviderDef {
  id: string;
  name: string;
  hasApiKey: boolean;
  hasBaseUrl: boolean;
  hasModel: boolean;
  defaultModel?: string;
  modelPlaceholder?: string;
  baseUrlPlaceholder?: string;
}

export const TRANSLATION_PROVIDERS: TranslationProviderDef[] = [
  { id: "anthropic", name: "Anthropic (Claude)", hasApiKey: true, hasBaseUrl: true, hasModel: true,
    defaultModel: "claude-haiku-4-5-20251001", modelPlaceholder: "claude-haiku-4-5-20251001",
    baseUrlPlaceholder: "https://api.anthropic.com" },
  { id: "openai", name: "OpenAI", hasApiKey: true, hasBaseUrl: true, hasModel: true,
    defaultModel: "gpt-4o-mini", modelPlaceholder: "gpt-4o-mini",
    baseUrlPlaceholder: "https://api.openai.com" },
  { id: "gemini", name: "Google Gemini", hasApiKey: true, hasBaseUrl: false, hasModel: true,
    defaultModel: "gemini-2.0-flash", modelPlaceholder: "gemini-2.0-flash" },
  { id: "glm", name: "GLM (Zhipu AI)", hasApiKey: true, hasBaseUrl: true, hasModel: true,
    defaultModel: "glm-5", modelPlaceholder: "glm-5",
    baseUrlPlaceholder: "https://open.bigmodel.cn/api/paas" },
  { id: "deepl", name: "DeepL", hasApiKey: true, hasBaseUrl: false, hasModel: false },
  { id: "openai_compatible", name: "OpenAI Compatible", hasApiKey: true, hasBaseUrl: true, hasModel: true,
    modelPlaceholder: "model-name", baseUrlPlaceholder: "https://api.example.com" },
  { id: "libretranslate", name: "LibreTranslate", hasApiKey: false, hasBaseUrl: true, hasModel: false,
    baseUrlPlaceholder: "http://localhost:5000" },
  { id: "nllb", name: "NLLB-200 (Local)", hasApiKey: false, hasBaseUrl: false, hasModel: true },
  { id: "nllb_api", name: "NLLB-200 (Server)", hasApiKey: false, hasBaseUrl: true, hasModel: false,
    baseUrlPlaceholder: "http://localhost:7860" },
];

export const NLLB_MODELS = [
  { id: "600M", name: "NLLB 600M", size: "~2.5 GB" },
  { id: "1.3B", name: "NLLB 1.3B", size: "~5.5 GB" },
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
