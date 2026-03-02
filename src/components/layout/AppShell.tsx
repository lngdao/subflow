import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { useUiStore } from "@/stores/useUiStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTaskEvents } from "@/hooks/useTaskEvents";
import { SourceTab } from "@/components/tabs/SourceTab";
import { FilesTab } from "@/components/tabs/FilesTab";
import { QueueTab } from "@/components/tabs/QueueTab";
import { SettingsPanel } from "@/components/settings/SettingsPanel";
import { BottomToolbar } from "./BottomToolbar";
import { LogPanel } from "./LogPanel";
import {
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
} from "@/components/animate-ui/components/radix/tabs";
import { Badge } from "@/components/ui/badge";

const ACTIVE_STATUSES = new Set([
  "Queued",
  "Downloading",
  "Translating",
  "GeneratingTts",
]);

export function AppShell() {
  const { t } = useTranslation();
  const isLogOpen = useUiStore((s) => s.isLogOpen);
  const activeTab = useUiStore((s) => s.activeTab);
  const setActiveTab = useUiStore((s) => s.setActiveTab);
  const loadSettings = useSettingsStore((s) => s.loadSettings);
  const setAppReady = useUiStore((s) => s.setAppReady);
  const tasks = useTaskStore((s) => s.tasks);

  const activeCount = tasks.filter((t) =>
    ACTIVE_STATUSES.has(t.status),
  ).length;

  useTaskEvents();

  useEffect(() => {
    Promise.all([
      loadSettings(),
      invoke("setup_binaries").catch(console.error),
    ]).finally(() => setAppReady());
  }, [loadSettings, setAppReady]);

  return (
    <div className="flex flex-col h-screen bg-background">
      <Tabs
        value={activeTab}
        onValueChange={(v) => setActiveTab(v as "source" | "files" | "queue")}
        className="flex flex-col flex-1 overflow-hidden"
      >
        {/* Tab Header */}
        <div className="flex items-center border-b border-border px-5 py-3">
          <TabsList className="h-8">
            <TabsTrigger value="source" className="px-4 text-sm">
              {t("tabs.source")}
            </TabsTrigger>
            <TabsTrigger value="files" className="px-4 text-sm">
              {t("tabs.files")}
            </TabsTrigger>
            <TabsTrigger value="queue" className="px-4 text-sm gap-1.5">
              {t("tabs.queue")}
              {activeCount > 0 && (
                <Badge
                  variant="default"
                  className="h-4 min-w-4 px-1 text-[10px] leading-none"
                >
                  {activeCount}
                </Badge>
              )}
            </TabsTrigger>
          </TabsList>
          {/* <div className="ml-auto">
            <h1 className="text-xs font-semibold text-muted-foreground tracking-wider">
              {t("app.name")}
            </h1>
          </div> */}
        </div>

        {/* Main Content */}
        <TabsContent value="source" className="flex-1 overflow-hidden">
          <SourceTab />
        </TabsContent>
        <TabsContent value="files" className="flex-1 overflow-hidden">
          <FilesTab />
        </TabsContent>
        <TabsContent value="queue" className="flex-1 overflow-hidden">
          <QueueTab />
        </TabsContent>
      </Tabs>

      {/* Log Panel */}
      {isLogOpen && <LogPanel />}

      {/* Bottom Toolbar */}
      <BottomToolbar />

      {/* Settings Sheet */}
      <SettingsPanel />
    </div>
  );
}
