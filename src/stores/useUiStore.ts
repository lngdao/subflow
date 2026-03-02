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
  appReady: boolean;
  splashVisible: boolean;
  tabActionTrigger: number;
  addActionEnabled: boolean;
  openSettings: () => void;
  closeSettings: () => void;
  toggleSettings: () => void;
  toggleLog: () => void;
  setActiveTab: (tab: "source" | "files" | "queue") => void;
  addLog: (entry: LogEntry) => void;
  clearLogs: () => void;
  setAppReady: () => void;
  triggerTabAction: () => void;
  setAddActionEnabled: (enabled: boolean) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  isSettingsOpen: false,
  isLogOpen: false,
  activeTab: "source",
  logs: [],
  appReady: false,
  splashVisible: true,
  tabActionTrigger: 0,
  addActionEnabled: false,
  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),
  toggleSettings: () => set((s) => ({ isSettingsOpen: !s.isSettingsOpen })),
  toggleLog: () => set((s) => ({ isLogOpen: !s.isLogOpen })),
  setActiveTab: (tab) => set({ activeTab: tab }),
  addLog: (entry) =>
    set((s) => ({ logs: [...s.logs.slice(-199), entry] })),
  clearLogs: () => set({ logs: [] }),
  setAppReady: () => {
    set({ appReady: true });
    setTimeout(() => set({ splashVisible: false }), 600);
  },
  triggerTabAction: () => set((s) => ({ tabActionTrigger: s.tabActionTrigger + 1 })),
  setAddActionEnabled: (enabled) => set({ addActionEnabled: enabled }),
}));
