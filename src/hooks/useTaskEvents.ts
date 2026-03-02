import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import type { TaskEvent } from "@/lib/types";
import { useTaskStore } from "@/stores/useTaskStore";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { useUiStore } from "@/stores/useUiStore";

export function useTaskEvents() {
  const updateFromEvent = useTaskStore((s) => s.updateFromEvent);
  const loadTasks = useTaskStore((s) => s.loadTasks);
  const addLog = useUiStore((s) => s.addLog);

  useEffect(() => {
    const unlisten = listen<TaskEvent>("task-event", async (event) => {
      updateFromEvent(event.payload);
      addLog({
        timestamp: new Date().toLocaleTimeString(),
        message: `[${event.payload.status}] ${event.payload.message}`,
        taskId: event.payload.task_id,
      });

      // Send system notification for completed/failed tasks
      const { status, message } = event.payload;
      if (status === "Completed" || status === "Failed") {
        const settings = useSettingsStore.getState().settings;
        if (!settings?.notifications?.enabled) return;

        try {
          let granted = await isPermissionGranted();
          if (!granted) {
            const permission = await requestPermission();
            granted = permission === "granted";
          }
          if (granted) {
            sendNotification({
              title: status === "Completed" ? "Task Completed" : "Task Failed",
              body: message,
            });
          }
        } catch {
          // Notification not available
        }
      }
    });

    // Initial load
    loadTasks();

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [updateFromEvent, loadTasks, addLog]);
}
