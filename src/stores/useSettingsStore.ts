import { create } from "zustand";
import type { AppConfig, VoiceInfo } from "@/lib/types";
import * as api from "@/lib/tauri";

interface SettingsStore {
  settings: AppConfig | null;
  loading: boolean;
  voiceList: VoiceInfo[];
  voiceListLoaded: boolean;
  loadSettings: () => Promise<void>;
  saveSettings: (config: AppConfig) => Promise<void>;
  testConnection: (
    provider: string,
    apiKey: string,
    baseUrl?: string,
    model?: string,
  ) => Promise<boolean>;
  saveApiKey: (provider: string, apiKey: string) => Promise<void>;
  loadVoiceList: () => Promise<void>;
}

const DEFAULT_SETTINGS: AppConfig = {
  translation: {
    provider: "claude",
    base_url: null,
    model: "claude-haiku-4-5-20251001",
    source_lang: "auto",
    target_langs: ["vi"],
  },
  tts: {
    provider: "edge",
    voices: {},
    speed: 1.0,
  },
  output: {
    format: "srt",
    folder: "",
  },
  queue: {
    parallel_jobs: 2,
  },
};

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: null,
  loading: false,
  voiceList: [],
  voiceListLoaded: false,

  loadSettings: async () => {
    set({ loading: true });
    try {
      const settings = await api.getSettings();
      // Ensure voices field exists (migration from old config)
      if (!settings.tts.voices) {
        settings.tts.voices = {};
      }
      set({ settings, loading: false });
    } catch {
      set({ settings: DEFAULT_SETTINGS, loading: false });
    }
  },

  saveSettings: async (config) => {
    await api.saveSettings(config);
    set({ settings: config });
  },

  testConnection: async (provider, apiKey, baseUrl, model) => {
    return api.testProviderConnection(provider, apiKey, baseUrl, model);
  },

  saveApiKey: async (provider, apiKey) => {
    await api.saveApiKey(provider, apiKey);
  },

  loadVoiceList: async () => {
    if (get().voiceListLoaded) return;
    try {
      const voices = await api.listTtsVoices();
      set({ voiceList: voices, voiceListLoaded: true });
    } catch {
      set({ voiceList: [], voiceListLoaded: true });
    }
  },
}));
