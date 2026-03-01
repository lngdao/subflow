import { useEffect, useRef } from "react";
import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUiStore } from "@/stores/useUiStore";
import { Button } from "@/components/ui/button";

export function LogPanel() {
  const { t } = useTranslation();
  const logs = useUiStore((s) => s.logs);
  const clearLogs = useUiStore((s) => s.clearLogs);
  const toggleLog = useUiStore((s) => s.toggleLog);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div className="border-t border-border bg-card/80 backdrop-blur-sm">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <span className="text-xs font-medium text-secondary-foreground">
          {t("log.title")}
        </span>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="xs" onClick={clearLogs}>
            <span className="text-xs">{t("log.clear")}</span>
          </Button>
          <button
            onClick={toggleLog}
            className="text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Log entries */}
      <div
        ref={scrollRef}
        className="h-48 overflow-y-auto px-4 py-2 space-y-0.5"
      >
        {logs.length === 0 ? (
          <p className="text-xs text-muted-foreground text-center py-4">
            {t("log.empty")}
          </p>
        ) : (
          logs.map((log, i) => (
            <div key={i} className="flex gap-2 text-xs font-mono">
              <span className="text-muted-foreground shrink-0">
                {log.timestamp}
              </span>
              <span className="text-secondary-foreground">{log.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
