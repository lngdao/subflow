import { create } from "zustand";

export interface LogEntry {
  timestamp: string;
  message: string;
  taskId?: string;
}

interface UiStore {
  isSettingsOpen: boolean;
  isLogOpen: boolean;
  activeTab: "source" | "files" | "queue";
  logs: LogEntry[];
  openSettings: () => void;
  closeSettings: () => void;
  toggleSettings: () => void;
  toggleLog: () => void;
  setActiveTab: (tab: "source" | "files" | "queue") => void;
  addLog: (entry: LogEntry) => void;
  clearLogs: () => void;
}

export const useUiStore = create<UiStore>((set) => ({
  isSettingsOpen: false,
  isLogOpen: false,
  activeTab: "source",
  logs: [],
  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),
  toggleSettings: () => set((s) => ({ isSettingsOpen: !s.isSettingsOpen })),
  toggleLog: () => set((s) => ({ isLogOpen: !s.isLogOpen })),
  setActiveTab: (tab) => set({ activeTab: tab }),
  addLog: (entry) =>
    set((s) => ({ logs: [...s.logs.slice(-199), entry] })),
  clearLogs: () => set({ logs: [] }),
}));
