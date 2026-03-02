import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useTaskStore } from "@/stores/useTaskStore";
import { useUiStore } from "@/stores/useUiStore";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { ProcessingConfig } from "@/components/config/ProcessingConfig";

const SOURCE_MODES = [
  { value: "sub_only", key: "config.subOnly" },
  { value: "sub_translate", key: "config.subTranslate" },
  { value: "sub_translate_tts", key: "config.subTranslateTts" },
] as const;

export function SourceTab() {
  const { t } = useTranslation();
  const addTask = useTaskStore((s) => s.addTask);
  const activeTab = useUiStore((s) => s.activeTab);
  const tabActionTrigger = useUiStore((s) => s.tabActionTrigger);
  const setAddActionEnabled = useUiStore((s) => s.setAddActionEnabled);
  const [urlInput, setUrlInput] = useState("");
  const [mode, setMode] = useState("sub_translate_tts");

  // Update header Add button enabled state based on input
  useEffect(() => {
    if (activeTab === "source") {
      setAddActionEnabled(urlInput.trim().length > 0);
    }
  }, [urlInput, activeTab, setAddActionEnabled]);

  const handleAdd = useCallback(async () => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    const lines = trimmed
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean);
    let added = 0;
    for (const line of lines) {
      try {
        new URL(line);
        await addTask(line, undefined, mode);
        added++;
      } catch {
        // skip invalid URLs
      }
    }
    if (added > 0) {
      setUrlInput("");
      toast.success(`Added ${added} task${added > 1 ? "s" : ""}`);
    } else if (lines.length > 0) {
      toast.error("No valid URLs found");
    }
  }, [urlInput, addTask, mode]);

  // Listen for [+] button trigger from AppShell
  useEffect(() => {
    if (tabActionTrigger > 0 && activeTab === "source") {
      handleAdd();
    }
  }, [tabActionTrigger]); // eslint-disable-line react-hooks/exhaustive-deps

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
        <Textarea
          value={urlInput}
          onChange={(e) => setUrlInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t("source.placeholder")}
          rows={3}
          className="resize-none font-mono text-sm min-h-0"
        />

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
