import {
  CheckCircle2,
  Clock,
  Download,
  FileText,
  FolderOpen,
  Globe,
  Mic,
  Pause,
  Play,
  RefreshCw,
  Trash2,
  X,
  XCircle,
  AlertCircle,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Task, TaskStatus, ProcessingMode } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Progress } from "@/components/animate-ui/components/radix/progress";
import { useTaskStore } from "@/stores/useTaskStore";
import { invoke } from "@tauri-apps/api/core";

function formatTimeAgo(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return "just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDay = Math.floor(diffHr / 24);
  return `${diffDay}d ago`;
}

function formatDuration(startStr: string, endStr: string): string {
  const start = new Date(startStr);
  const end = new Date(endStr);
  const diffSec = Math.round((end.getTime() - start.getTime()) / 1000);
  if (diffSec < 60) return `${diffSec}s`;
  const min = Math.floor(diffSec / 60);
  const sec = diffSec % 60;
  if (min < 60) return sec > 0 ? `${min}m ${sec}s` : `${min}m`;
  const hr = Math.floor(min / 60);
  const remMin = min % 60;
  return remMin > 0 ? `${hr}h ${remMin}m` : `${hr}h`;
}

function statusIcon(status: TaskStatus, isFile: boolean) {
  switch (status) {
    case "Queued":
      return <Clock className="w-4 h-4 text-muted-foreground" strokeWidth={1.5} />;
    case "Downloading":
      return <Download className="w-4 h-4 text-accent-warning animate-pulse" strokeWidth={1.5} />;
    case "Translating":
      return isFile
        ? <FileText className="w-4 h-4 text-primary animate-pulse" strokeWidth={1.5} />
        : <Globe className="w-4 h-4 text-primary animate-pulse" strokeWidth={1.5} />;
    case "GeneratingTts":
      return <Mic className="w-4 h-4 text-primary animate-pulse" strokeWidth={1.5} />;
    case "Completed":
      return <CheckCircle2 className="w-4 h-4 text-accent-success" strokeWidth={1.5} />;
    case "Failed":
      return <XCircle className="w-4 h-4 text-destructive" strokeWidth={1.5} />;
    case "Cancelled":
      return <X className="w-4 h-4 text-muted-foreground" strokeWidth={1.5} />;
    case "Paused":
      return <Pause className="w-4 h-4 text-accent-warning" strokeWidth={1.5} />;
  }
}

const STATUS_KEY_MAP: Record<TaskStatus, string> = {
  Queued: "task.queued",
  Downloading: "task.downloading",
  Translating: "task.translating",
  GeneratingTts: "task.generatingTts",
  Completed: "task.completed",
  Failed: "task.failed",
  Cancelled: "task.cancelled",
  Paused: "task.paused",
};

function statusBadgeVariant(
  status: TaskStatus,
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "Completed":
      return "default";
    case "Failed":
      return "destructive";
    case "Queued":
    case "Cancelled":
      return "secondary";
    default:
      return "outline";
  }
}

const MODE_KEY_MAP: Record<ProcessingMode, string> = {
  SubOnly: "task.modeSubOnly",
  SubTranslate: "task.modeSubTranslate",
  SubTranslateTts: "task.modeSubTranslateTts",
};

interface TaskCardProps {
  task: Task;
}

export function TaskCard({ task }: TaskCardProps) {
  const { t } = useTranslation();
  const { cancelTask, pauseTask, resumeTask, retryTask, removeTask } = useTaskStore();
  const isActive = ["Downloading", "Translating", "GeneratingTts"].includes(
    task.status,
  );
  const isFile = !task.url && !!task.file_path;

  const handleOpenFolder = async () => {
    if (task.output_dir) {
      try {
        await invoke("open_folder", { path: task.output_dir });
      } catch {
        // Ignore
      }
    }
  };

  const title = task.video_title || (task.file_path ? task.file_path.split("/").pop() : task.url) || "Untitled";

  const timeAgo = formatTimeAgo(task.created_at);

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div className="flex items-start gap-3 rounded-lg border border-border bg-card/50 px-3 py-2.5 cursor-default">
          {/* Status Icon */}
          <div className="mt-0.5 shrink-0">
            {isFile && !isActive && task.status !== "Failed"
              ? <FileText className="w-4 h-4 text-muted-foreground" strokeWidth={1.5} />
              : statusIcon(task.status, isFile)
            }
          </div>

          {/* Content */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-foreground truncate">
                {title}
              </span>
              <Badge variant={statusBadgeVariant(task.status)} className="shrink-0 text-[10px] h-5 px-1.5">
                {t(STATUS_KEY_MAP[task.status])}
              </Badge>
              <span className="text-[10px] text-muted-foreground/60 shrink-0 ml-auto">
                {task.status === "Completed" && task.started_at && task.completed_at
                  ? formatDuration(task.started_at, task.completed_at)
                  : timeAgo}
              </span>
            </div>

            <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
              <span className="text-[10px] text-muted-foreground/70 border border-border rounded px-1 py-0.5">
                {t(MODE_KEY_MAP[task.mode])}
              </span>
              <span>{task.source_lang.toUpperCase()}</span>
              <span>&rarr;</span>
              <span>{task.target_langs.map((l) => l.toUpperCase()).join(", ")}</span>
              {task.current_lang && isActive && (
                <>
                  <span className="text-border">|</span>
                  <span className="text-primary">{task.current_lang.toUpperCase()}</span>
                </>
              )}
            </div>

            {/* Progress */}
            {isActive && (
              <div className="mt-2">
                <Progress value={task.progress * 100} className="h-1" />
                <p className="text-[11px] text-muted-foreground mt-1">
                  {task.message}
                </p>
              </div>
            )}

            {task.status === "Failed" && task.error && (
              <div className="flex items-start gap-1.5 mt-1.5">
                <AlertCircle className="w-3 h-3 text-destructive mt-0.5 shrink-0" />
                <p className="text-[11px] text-destructive line-clamp-2">{task.error}</p>
              </div>
            )}
          </div>

          {/* Inline Actions */}
          <div className="flex items-center gap-0.5 shrink-0">
            {isActive && (
              <>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={() => pauseTask(task.id)}
                  title={t("task.pause")}
                >
                  <Pause className="w-3.5 h-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={() => cancelTask(task.id)}
                  title={t("task.cancel")}
                >
                  <X className="w-3.5 h-3.5" />
                </Button>
              </>
            )}
            {task.status === "Paused" && (
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={() => resumeTask(task.id)}
                title={t("task.resume")}
              >
                <Play className="w-3.5 h-3.5" />
              </Button>
            )}
            {task.status === "Failed" && (
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={() => retryTask(task.id)}
                title={t("task.retry")}
              >
                <RefreshCw className="w-3.5 h-3.5" />
              </Button>
            )}
            {task.status === "Completed" && task.output_dir && (
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={handleOpenFolder}
                title={t("task.openFolder")}
              >
                <FolderOpen className="w-3.5 h-3.5" />
              </Button>
            )}
          </div>
        </div>
      </ContextMenuTrigger>

      <ContextMenuContent>
        <ContextMenuItem onClick={handleOpenFolder} disabled={!task.output_dir}>
          <FolderOpen className="w-4 h-4" />
          {t("task.openFolder")}
        </ContextMenuItem>
        {task.status === "Failed" && (
          <ContextMenuItem onClick={() => retryTask(task.id)}>
            <RefreshCw className="w-4 h-4" />
            {t("task.retry")}
          </ContextMenuItem>
        )}
        {task.status === "Paused" && (
          <ContextMenuItem onClick={() => resumeTask(task.id)}>
            <Play className="w-4 h-4" />
            {t("task.resume")}
          </ContextMenuItem>
        )}
        {isActive && (
          <>
            <ContextMenuItem onClick={() => pauseTask(task.id)}>
              <Pause className="w-4 h-4" />
              {t("task.pause")}
            </ContextMenuItem>
            <ContextMenuItem onClick={() => cancelTask(task.id)}>
              <X className="w-4 h-4" />
              {t("task.cancel")}
            </ContextMenuItem>
          </>
        )}
        <ContextMenuSeparator />
        <ContextMenuItem variant="destructive" onClick={() => removeTask(task.id)}>
          <Trash2 className="w-4 h-4" />
          {t("task.remove")}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
