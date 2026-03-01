import { useCallback, useState } from "react";
import { Plus } from "lucide-react";
import { useTaskStore } from "@/stores/useTaskStore";
import { TaskCard } from "./TaskCard";
import { Button } from "@/components/ui/Button";
import { YOUTUBE_URL_REGEX } from "@/lib/constants";

export function QueueView() {
  const tasks = useTaskStore((s) => s.tasks);
  const addTask = useTaskStore((s) => s.addTask);
  const [showInput, setShowInput] = useState(false);
  const [urlInput, setUrlInput] = useState("");

  const handleAdd = useCallback(async () => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    const lines = trimmed.split("\n").map((l) => l.trim()).filter(Boolean);
    for (const line of lines) {
      if (YOUTUBE_URL_REGEX.test(line)) {
        await addTask(line);
      }
    }
    setUrlInput("");
    setShowInput(false);
  }, [urlInput, addTask]);

  const handleBrowse = useCallback(async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const result = await open({
        multiple: true,
        filters: [{ name: "Subtitles", extensions: ["srt", "vtt", "txt"] }],
      });
      if (result) {
        const paths = Array.isArray(result) ? result : [result];
        for (const p of paths) {
          await addTask(undefined, p);
        }
      }
    } catch {
      // Cancelled
    }
  }, [addTask]);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-subtle">
        <h2 className="text-lg font-semibold text-text-primary">Queue</h2>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={handleBrowse}>
            Browse
          </Button>
          <Button size="sm" onClick={() => setShowInput(!showInput)}>
            <Plus className="w-4 h-4" />
            Add
          </Button>
        </div>
      </div>

      {/* Quick URL Input */}
      {showInput && (
        <div className="px-6 py-3 border-b border-border-subtle bg-bg-secondary/50">
          <div className="flex gap-2">
            <input
              value={urlInput}
              onChange={(e) => setUrlInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAdd();
              }}
              placeholder="Paste YouTube URL..."
              autoFocus
              className="flex-1 rounded-[10px] bg-bg-secondary border border-border-subtle px-3 py-1.5 text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-border-focus transition-colors font-mono"
            />
            <Button size="sm" onClick={handleAdd} disabled={!urlInput.trim()}>
              Add
            </Button>
          </div>
        </div>
      )}

      {/* Task List */}
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-3">
        {tasks.map((task) => (
          <TaskCard key={task.id} task={task} />
        ))}
      </div>
    </div>
  );
}
