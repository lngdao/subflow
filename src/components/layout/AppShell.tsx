import { useEffect } from "react";
import { useTaskStore } from "@/stores/useTaskStore";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { useUiStore } from "@/stores/useUiStore";
import { useTaskEvents } from "@/hooks/useTaskEvents";
import { DropZone } from "@/components/empty-state/DropZone";
import { QueueView } from "@/components/queue/QueueView";
import { SettingsPanel } from "@/components/settings/SettingsPanel";
import { BottomToolbar } from "./BottomToolbar";

export function AppShell() {
  const tasks = useTaskStore((s) => s.tasks);
  const isSettingsOpen = useUiStore((s) => s.isSettingsOpen);
  const loadSettings = useSettingsStore((s) => s.loadSettings);

  useTaskEvents();

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const hasTasks = tasks.length > 0;

  return (
    <div className="flex flex-col h-screen bg-bg-primary">
      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        {hasTasks ? <QueueView /> : <DropZone />}
      </div>

      {/* Bottom Toolbar */}
      <BottomToolbar />

      {/* Settings Panel */}
      {isSettingsOpen && <SettingsPanel />}
    </div>
  );
}
