import { create } from "zustand";
import type { Task, TaskEvent } from "@/lib/types";
import * as api from "@/lib/tauri";

interface TaskStore {
  tasks: Task[];
  loading: boolean;
  addTask: (url?: string, filePath?: string, mode?: string, useYtTranslation?: boolean) => Promise<void>;
  loadTasks: () => Promise<void>;
  updateFromEvent: (event: TaskEvent) => void;
  cancelTask: (taskId: string) => Promise<void>;
  pauseTask: (taskId: string) => Promise<void>;
  resumeTask: (taskId: string) => Promise<void>;
  retryTask: (taskId: string) => Promise<void>;
  removeTask: (taskId: string) => Promise<void>;
}

export const useTaskStore = create<TaskStore>((set, get) => ({
  tasks: [],
  loading: false,

  addTask: async (url, filePath, mode, useYtTranslation) => {
    await api.addTask(url, filePath, undefined, undefined, mode, useYtTranslation);
    // Reload tasks to get the new task
    await get().loadTasks();
  },

  loadTasks: async () => {
    set({ loading: true });
    try {
      const tasks = await api.getTasks();
      set({ tasks, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  updateFromEvent: (event: TaskEvent) => {
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === event.task_id
          ? {
              ...t,
              status: event.status,
              progress: event.progress,
              message: event.message,
              current_lang: event.current_lang,
              ...(event.video_title ? { video_title: event.video_title } : {}),
            }
          : t,
      ),
    }));
  },

  cancelTask: async (taskId) => {
    await api.cancelTask(taskId);
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === taskId ? { ...t, status: "Cancelled" as const, message: "Cancelled" } : t,
      ),
    }));
  },

  pauseTask: async (taskId) => {
    await api.pauseTask(taskId);
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === taskId ? { ...t, status: "Paused" as const, message: "Paused" } : t,
      ),
    }));
  },

  resumeTask: async (taskId) => {
    await api.resumeTask(taskId);
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === taskId ? { ...t, status: "Queued" as const, message: "Resumed" } : t,
      ),
    }));
  },

  retryTask: async (taskId) => {
    await api.retryTask(taskId);
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === taskId
          ? { ...t, status: "Queued" as const, progress: 0, message: "Queued (retry)", error: null }
          : t,
      ),
    }));
  },

  removeTask: async (taskId) => {
    await api.removeTask(taskId);
    set((state) => ({
      tasks: state.tasks.filter((t) => t.id !== taskId),
    }));
  },
}));
