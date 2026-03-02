import { useCallback, useState } from "react";
import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useTaskStore } from "@/stores/useTaskStore";
import { TaskCard } from "./TaskCard";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function QueueView() {
  const { t } = useTranslation();
  const tasks = useTaskStore((s) => s.tasks);
  const addTask = useTaskStore((s) => s.addTask);
  const [showInput, setShowInput] = useState(false);
  const [urlInput, setUrlInput] = useState("");

  const handleAdd = useCallback(async () => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    const lines = trimmed.split("\n").map((l) => l.trim()).filter(Boolean);
    for (const line of lines) {
      try {
        new URL(line);
        await addTask(line);
      } catch {
        // skip invalid URLs
      }
    }
    setUrlInput("");
    setShowInput(false);
  }, [urlInput, addTask]);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-border">
        <h2 className="text-sm font-semibold text-foreground">{t("queue.title")}</h2>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setShowInput(!showInput)}
          className="text-xs gap-1"
        >
          <Plus className="w-3.5 h-3.5" />
          {t("source.add")}
        </Button>
      </div>

      {/* Quick URL Input */}
      {showInput && (
        <div className="px-5 py-2.5 border-b border-border">
          <div className="flex gap-2">
            <Input
              value={urlInput}
              onChange={(e) => setUrlInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAdd();
              }}
              placeholder={t("source.placeholder")}
              autoFocus
              className="font-mono text-xs"
            />
            <Button size="sm" onClick={handleAdd} disabled={!urlInput.trim()}>
              {t("source.add")}
            </Button>
          </div>
        </div>
      )}

      {/* Task List */}
      <div className="flex-1 overflow-y-auto px-5 py-3 space-y-2">
        {tasks.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-8">
            {t("queue.empty")}
          </p>
        ) : (
          tasks.map((task) => (
            <TaskCard key={task.id} task={task} />
          ))
        )}
      </div>
    </div>
  );
}
