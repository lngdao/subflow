import { useCallback, useState } from "react";
import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useTaskStore } from "@/stores/useTaskStore";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { ProcessingConfig } from "@/components/config/ProcessingConfig";
import { YOUTUBE_URL_REGEX } from "@/lib/constants";

const SOURCE_MODES = [
  { value: "sub_only", key: "config.subOnly" },
  { value: "sub_translate", key: "config.subTranslate" },
  { value: "sub_translate_tts", key: "config.subTranslateTts" },
] as const;

export function SourceTab() {
  const { t } = useTranslation();
  const addTask = useTaskStore((s) => s.addTask);
  const [urlInput, setUrlInput] = useState("");
  const [mode, setMode] = useState("sub_translate_tts");

  const handleAdd = useCallback(async () => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    const lines = trimmed
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean);
    let added = 0;
    for (const line of lines) {
      if (YOUTUBE_URL_REGEX.test(line)) {
        try {
          await addTask(line, undefined, mode);
          added++;
        } catch (e) {
          console.error("Failed to add task:", e);
          toast.error(`Failed to add: ${e}`);
        }
      }
    }
    if (added > 0) {
      setUrlInput("");
      toast.success(`Added ${added} task${added > 1 ? "s" : ""}`);
    } else if (lines.length > 0) {
      toast.error("No valid YouTube URLs found");
    }
  }, [urlInput, addTask, mode]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleAdd();
      }
    },
    [handleAdd],
  );

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      {/* URL Input */}
      <div className="px-5 py-4 border-b border-border">
        <div className="flex gap-2">
          <Textarea
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("source.placeholder")}
            rows={2}
            className="flex-1 resize-none font-mono text-sm min-h-0"
          />
          <Button
            onClick={handleAdd}
            disabled={!urlInput.trim()}
            className="self-end"
          >
            <Plus className="w-4 h-4" />
            {t("source.add")}
          </Button>
        </div>

        {/* Mode Selection */}
        <div className="mt-4">
          <Label className="text-xs text-muted-foreground mb-3">
            {t("config.mode")}
          </Label>
          <div className="flex gap-2">
            {SOURCE_MODES.map((m) => (
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
        </div>
      </div>

      {/* Processing Config */}
      <ProcessingConfig />
    </div>
  );
}
