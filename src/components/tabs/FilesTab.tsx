import { useCallback, useState } from "react";
import { FileDown, FileText, Play, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { useDropZone } from "@/hooks/useDropZone";
import { useTaskStore } from "@/stores/useTaskStore";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { ProcessingConfig } from "@/components/config/ProcessingConfig";

const FILE_MODES = [
  { value: "sub_translate", key: "config.translateOnly" },
  { value: "sub_translate_tts", key: "config.translateTts" },
] as const;

export function FilesTab() {
  const { t } = useTranslation();
  const { isDragging, handleDragOver, handleDragLeave, extractPaths } =
    useDropZone();
  const addTask = useTaskStore((s) => s.addTask);
  const [stagedFiles, setStagedFiles] = useState<string[]>([]);
  const [mode, setMode] = useState("sub_translate_tts");

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      const paths = extractPaths(e);
      if (paths.length > 0) {
        setStagedFiles((prev) => {
          const existing = new Set(prev);
          return [...prev, ...paths.filter((p) => !existing.has(p))];
        });
      }
    },
    [extractPaths],
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
        setStagedFiles((prev) => {
          const existing = new Set(prev);
          return [...prev, ...paths.filter((p) => !existing.has(p))];
        });
      }
    } catch {
      // Dialog cancelled
    }
  }, []);

  const handleRemove = useCallback((path: string) => {
    setStagedFiles((prev) => prev.filter((p) => p !== path));
  }, []);

  const handleStart = useCallback(async () => {
    let added = 0;
    for (const p of stagedFiles) {
      try {
        await addTask(undefined, p, mode);
        added++;
      } catch (e) {
        console.error("Failed to add file:", e);
        toast.error(`Failed: ${p.split("/").pop()}`);
      }
    }
    if (added > 0) {
      toast.success(`Added ${added} file${added > 1 ? "s" : ""} to queue`);
      setStagedFiles([]);
    }
  }, [stagedFiles, addTask, mode]);

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      {/* Drop Zone */}
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        className={cn(
          "mx-5 mt-4 border-2 border-dashed rounded-xl p-6 text-center transition-all duration-200",
          isDragging
            ? "border-primary bg-primary/5 scale-[1.01]"
            : "border-border hover:border-ring",
        )}
      >
        <FileDown
          className="w-8 h-8 text-muted-foreground mx-auto mb-2"
          strokeWidth={1.5}
        />
        <p className="text-foreground text-sm mb-1">
          {t("dropzone.dropFiles")}
        </p>
        <p className="text-muted-foreground text-xs mb-3">
          {t("dropzone.supported")}
        </p>
        <Button variant="outline" size="sm" onClick={handleBrowse}>
          {t("source.browse")}
        </Button>
      </div>

      {/* Staged Files */}
      {stagedFiles.length > 0 && (
        <div className="mx-5 mt-3 space-y-2">
          {stagedFiles.map((path) => (
            <div
              key={path}
              className="flex items-center gap-2 rounded-lg border border-border bg-card/50 px-3 py-2"
            >
              <FileText className="w-4 h-4 text-muted-foreground shrink-0" strokeWidth={1.5} />
              <span className="text-sm text-foreground truncate flex-1">
                {path.split("/").pop()}
              </span>
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={() => handleRemove(path)}
              >
                <Trash2 className="w-3.5 h-3.5 text-muted-foreground" />
              </Button>
            </div>
          ))}

          {/* Mode + Start */}
          <div className="pt-2">
            <Label className="text-xs text-muted-foreground mb-2">
              {t("config.mode")}
            </Label>
            <div className="flex gap-2">
              {FILE_MODES.map((m) => (
                <button
                  key={m.value}
                  onClick={() => setMode(m.value)}
                  className={`px-2.5 py-1 text-xs rounded-full border transition-colors cursor-pointer ${
                    mode === m.value
                      ? "border-primary bg-primary/20 text-primary"
                      : "border-border text-muted-foreground hover:border-ring"
                  }`}
                >
                  {t(m.key)}
                </button>
              ))}
            </div>
            <Button
              className="w-full mt-5"
              size="sm"
              onClick={handleStart}
            >
              <Play className="w-3.5 h-3.5" />
              {t("source.add")} ({stagedFiles.length})
            </Button>
          </div>
        </div>
      )}

      {/* Processing Config */}
      <div className="mt-4">
        <ProcessingConfig />
      </div>
    </div>
  );
}
