import { create } from "zustand";
import type { AppConfig } from "@/lib/types";
import * as api from "@/lib/tauri";

interface SettingsStore {
  settings: AppConfig | null;
  loading: boolean;
  loadSettings: () => Promise<void>;
  saveSettings: (config: AppConfig) => Promise<void>;
  testConnection: (
    provider: string,
    apiKey: string,
    baseUrl?: string,
    model?: string,
  ) => Promise<boolean>;
  saveApiKey: (provider: string, apiKey: string) => Promise<void>;
}

const DEFAULT_SETTINGS: AppConfig = {
  translation: {
    provider: "claude",
    base_url: null,
    model: "claude-haiku-4-5-20251001",
    source_lang: "en",
    target_langs: ["vi"],
  },
  tts: {
    provider: "edge",
    voice: "en-US-AriaNeural",
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

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: null,
  loading: false,

  loadSettings: async () => {
    set({ loading: true });
    try {
      const settings = await api.getSettings();
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
}));
