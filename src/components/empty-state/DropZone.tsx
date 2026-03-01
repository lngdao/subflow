import { useCallback, useState } from "react";
import { FileDown, Plus } from "lucide-react";
import { clsx } from "clsx";
import { useDropZone } from "@/hooks/useDropZone";
import { useTaskStore } from "@/stores/useTaskStore";
import { Button } from "@/components/ui/button";
import { YOUTUBE_URL_REGEX } from "@/lib/constants";

export function DropZone() {
  const { isDragging, handleDragOver, handleDragLeave, extractPaths } = useDropZone();
  const addTask = useTaskStore((s) => s.addTask);
  const [urlInput, setUrlInput] = useState("");

  const handleSubmit = useCallback(async () => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;

    const lines = trimmed.split("\n").map((l) => l.trim()).filter(Boolean);
    for (const line of lines) {
      if (YOUTUBE_URL_REGEX.test(line)) {
        await addTask(line);
      }
    }
    setUrlInput("");
  }, [urlInput, addTask]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSubmit();
      }
    },
    [handleSubmit],
  );

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
      // Dialog cancelled
    }
  }, [addTask]);

  return (
    <div className="flex flex-col items-center justify-center h-full px-8">
      {/* Branding */}
      <div className="text-center mb-8">
        <h1 className="text-2xl font-bold text-text-primary tracking-tight">subflow</h1>
        <p className="text-text-tertiary text-sm mt-1">subtitle studio</p>
      </div>

      {/* Drop Zone */}
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={(e) => {
          const paths = extractPaths(e);
          for (const p of paths) {
            addTask(undefined, p);
          }
          // Check for text (URL)
          const text = e.dataTransfer?.getData("text/plain");
          if (text && YOUTUBE_URL_REGEX.test(text)) {
            addTask(text);
          }
        }}
        className={clsx(
          "w-full max-w-lg border-2 border-dashed rounded-[16px] p-8 text-center transition-all duration-200",
          isDragging
            ? "border-accent-primary bg-accent-primary/5 scale-[1.02]"
            : "border-border-subtle hover:border-border-focus",
        )}
      >
        <FileDown className="w-10 h-10 text-text-tertiary mx-auto mb-4" strokeWidth={1.5} />
        <p className="text-text-secondary text-sm mb-1">Drop files here</p>
        <p className="text-text-tertiary text-xs mb-4">or paste YouTube URLs</p>

        {/* URL Input */}
        <textarea
          value={urlInput}
          onChange={(e) => setUrlInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="https://youtube.com/watch?v=..."
          rows={2}
          className="w-full rounded-[10px] bg-bg-secondary border border-border-subtle px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-border-focus transition-colors resize-none font-mono"
        />

        <div className="flex items-center justify-center gap-2 mt-3">
          <Button size="sm" onClick={handleSubmit} disabled={!urlInput.trim()}>
            <Plus className="w-4 h-4" />
            Add
          </Button>
        </div>

        <div className="mt-4 pt-4 border-t border-border-subtle">
          <p className="text-text-tertiary text-xs">
            supported: .srt .vtt .txt
          </p>
        </div>
      </div>

      {/* Browse Button */}
      <Button variant="secondary" className="mt-4" onClick={handleBrowse}>
        Browse Files
      </Button>
    </div>
  );
}
