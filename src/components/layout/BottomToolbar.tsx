import { FolderOpen, Settings } from "lucide-react";
import { useUiStore } from "@/stores/useUiStore";
import { useSettingsStore } from "@/stores/useSettingsStore";

export function BottomToolbar() {
  const toggleSettings = useUiStore((s) => s.toggleSettings);
  const settings = useSettingsStore((s) => s.settings);

  const handleOpenFolder = async () => {
    if (settings?.output.folder) {
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        await open(settings.output.folder);
      } catch {
        // Ignore
      }
    }
  };

  return (
    <div className="flex items-center justify-between px-6 py-2 border-t border-border-subtle bg-bg-secondary/50">
      <div className="flex items-center gap-4">
        <button
          onClick={toggleSettings}
          className="flex items-center gap-1.5 text-text-tertiary hover:text-text-primary transition-colors text-xs"
        >
          <Settings className="w-4 h-4" strokeWidth={1.5} />
          Settings
        </button>
        <button
          onClick={handleOpenFolder}
          className="flex items-center gap-1.5 text-text-tertiary hover:text-text-primary transition-colors text-xs"
        >
          <FolderOpen className="w-4 h-4" strokeWidth={1.5} />
          Output
        </button>
      </div>
      <div className="text-text-tertiary text-xs">SubFlow v0.1.0</div>
    </div>
  );
}
