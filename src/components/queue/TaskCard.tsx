import {
  CheckCircle2,
  Clock,
  Download,
  FolderOpen,
  Globe,
  Mic,
  Pause,
  Play,
  X,
  XCircle,
  AlertCircle,
} from "lucide-react";
import type { Task, TaskStatus } from "@/lib/types";
import { Card } from "@/components/ui/Card";
import { Progress } from "@/components/ui/Progress";
import { Badge } from "@/components/ui/Badge";
import { Button } from "@/components/ui/Button";
import { useTaskStore } from "@/stores/useTaskStore";

function statusIcon(status: TaskStatus) {
  switch (status) {
    case "Queued":
      return <Clock className="w-5 h-5 text-text-tertiary" strokeWidth={1.5} />;
    case "Downloading":
      return <Download className="w-5 h-5 text-accent-warning animate-pulse" strokeWidth={1.5} />;
    case "Translating":
      return <Globe className="w-5 h-5 text-accent-primary animate-pulse" strokeWidth={1.5} />;
    case "GeneratingTts":
      return <Mic className="w-5 h-5 text-accent-primary animate-pulse" strokeWidth={1.5} />;
    case "Completed":
      return <CheckCircle2 className="w-5 h-5 text-accent-success" strokeWidth={1.5} />;
    case "Failed":
      return <XCircle className="w-5 h-5 text-accent-error" strokeWidth={1.5} />;
    case "Cancelled":
      return <X className="w-5 h-5 text-text-tertiary" strokeWidth={1.5} />;
    case "Paused":
      return <Pause className="w-5 h-5 text-accent-warning" strokeWidth={1.5} />;
  }
}

function statusBadge(status: TaskStatus) {
  const variant =
    status === "Completed"
      ? "success"
      : status === "Failed"
        ? "error"
        : status === "Queued" || status === "Cancelled"
          ? "default"
          : "warning";
  return <Badge variant={variant}>{status}</Badge>;
}

interface TaskCardProps {
  task: Task;
}

export function TaskCard({ task }: TaskCardProps) {
  const { cancelTask, pauseTask, resumeTask } = useTaskStore();
  const isActive = ["Downloading", "Translating", "GeneratingTts"].includes(task.status);

  const handleOpenFolder = async () => {
    if (task.output_dir) {
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        await open(task.output_dir);
      } catch {
        // Ignore
      }
    }
  };

  return (
    <Card className="p-4">
      <div className="flex items-start gap-3">
        {/* Status Icon */}
        <div className="mt-0.5">{statusIcon(task.status)}</div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="text-sm font-medium text-text-primary truncate">
              {task.video_title || task.url || task.file_path || "Untitled"}
            </h3>
            {statusBadge(task.status)}
          </div>

          {task.url && (
            <p className="text-xs text-text-tertiary font-mono truncate mb-2">
              {task.url}
            </p>
          )}

          <div className="flex items-center gap-2 text-xs text-text-secondary mb-2">
            <span>{task.source_lang.toUpperCase()}</span>
            <span className="text-text-tertiary">→</span>
            <span>{task.target_langs.map((l) => l.toUpperCase()).join(", ")}</span>
          </div>

          {/* Progress */}
          {isActive && (
            <div className="mb-2">
              <Progress value={task.progress} showLabel />
              <p className="text-xs text-text-tertiary mt-1">{task.message}</p>
            </div>
          )}

          {task.status === "Failed" && task.error && (
            <div className="flex items-start gap-1.5 mt-1">
              <AlertCircle className="w-3.5 h-3.5 text-accent-error mt-0.5 shrink-0" />
              <p className="text-xs text-accent-error">{task.error}</p>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1">
          {isActive && (
            <>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => pauseTask(task.id)}
                title="Pause"
              >
                <Pause className="w-4 h-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => cancelTask(task.id)}
                title="Cancel"
              >
                <X className="w-4 h-4" />
              </Button>
            </>
          )}
          {task.status === "Paused" && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => resumeTask(task.id)}
              title="Resume"
            >
              <Play className="w-4 h-4" />
            </Button>
          )}
          {task.status === "Completed" && task.output_dir && (
            <Button variant="ghost" size="sm" onClick={handleOpenFolder} title="Open Folder">
              <FolderOpen className="w-4 h-4" />
            </Button>
          )}
        </div>
      </div>
    </Card>
  );
}
