import { useTranslation } from "react-i18next";
import { useTaskStore } from "@/stores/useTaskStore";
import { TaskCard } from "@/components/queue/TaskCard";

export function QueueTab() {
  const { t } = useTranslation();
  const tasks = useTaskStore((s) => s.tasks);

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-y-auto px-5 py-4 space-y-3">
        {tasks.length === 0 ? (
          <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
            {t("queue.empty")}
          </div>
        ) : (
          tasks.map((task) => <TaskCard key={task.id} task={task} />)
        )}
      </div>
    </div>
  );
}
