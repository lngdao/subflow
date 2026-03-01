import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { TaskEvent } from "@/lib/types";
import { useTaskStore } from "@/stores/useTaskStore";
import { useUiStore } from "@/stores/useUiStore";

export function useTaskEvents() {
  const updateFromEvent = useTaskStore((s) => s.updateFromEvent);
  const loadTasks = useTaskStore((s) => s.loadTasks);
  const addLog = useUiStore((s) => s.addLog);

  useEffect(() => {
    const unlisten = listen<TaskEvent>("task-event", (event) => {
      updateFromEvent(event.payload);
      addLog({
        timestamp: new Date().toLocaleTimeString(),
        message: `[${event.payload.status}] ${event.payload.message}`,
        taskId: event.payload.task_id,
      });
    });

    // Initial load
    loadTasks();

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [updateFromEvent, loadTasks, addLog]);
}
