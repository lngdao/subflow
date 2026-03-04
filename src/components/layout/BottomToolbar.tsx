import { useCallback, useEffect, useState } from "react";
import {
  FolderOpen,
  ScrollText,
  Settings,
  AlertTriangle,
  Check,
  Download,
  Loader2,
  X,
  ExternalLink,
  RefreshCw,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useUiStore } from "@/stores/useUiStore";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { useUpdateChecker } from "@/hooks/useUpdateChecker";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { BinaryStatus, ModelDownloadProgress } from "@/lib/types";
import * as api from "@/lib/tauri";
import { cn } from "@/lib/utils";

export function BottomToolbar() {
  const { t } = useTranslation();
  const toggleSettings = useUiStore((s) => s.toggleSettings);
  const toggleLog = useUiStore((s) => s.toggleLog);
  const settings = useSettingsStore((s) => s.settings);
  const [binStatus, setBinStatus] = useState<BinaryStatus | null>(null);
  const [showDepsModal, setShowDepsModal] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [nllbDownloading, setNllbDownloading] = useState<string | null>(null);
  const [nllbProgress, setNllbProgress] = useState<ModelDownloadProgress | null>(null);
  const { update, checking, checkNow } = useUpdateChecker();

  const allOk =
    binStatus !== null &&
    binStatus.ytdlp_available &&
    binStatus.ffmpeg_available;
  const hasMissing =
    binStatus !== null &&
    (!binStatus.ytdlp_available || !binStatus.ffmpeg_available);

  // Load status on mount
  useEffect(() => {
    api.getBinaryStatus().then(setBinStatus).catch(console.error);
  }, []);

  // Listen for NLLB model download progress
  useEffect(() => {
    const unlisten = listen<ModelDownloadProgress>("model-download-progress", (event) => {
      setNllbProgress(event.payload);
      if (event.payload.status === "completed" && event.payload.file === "config.json") {
        // Last file completed, refresh status
        setNllbDownloading(null);
        setNllbProgress(null);
        api.getBinaryStatus().then(setBinStatus).catch(console.error);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleInstall = useCallback(async () => {
    setInstalling(true);
    try {
      const status = await api.setupBinaries();
      setBinStatus(status);
      if (status.ytdlp_available && status.ffmpeg_available) {
        toast.success(t("deps.allGood"));
      }
    } catch (e) {
      console.error("Install failed:", e);
      toast.error(`Install failed: ${e}`);
    } finally {
      setInstalling(false);
    }
  }, [t]);

  const handleDownloadNllb = useCallback(async (variant: string) => {
    setNllbDownloading(variant);
    setNllbProgress(null);
    try {
      await api.downloadNllbModel(variant);
      const status = await api.getBinaryStatus();
      setBinStatus(status);
      toast.success(`NLLB ${variant} model downloaded`);
    } catch (e) {
      console.error("NLLB download failed:", e);
      toast.error(`NLLB download failed: ${e}`);
    } finally {
      setNllbDownloading(null);
      setNllbProgress(null);
    }
  }, []);

  const handleDeleteNllb = useCallback(async (variant: string) => {
    try {
      await api.deleteNllbModel(variant);
      const status = await api.getBinaryStatus();
      setBinStatus(status);
      toast.success(`NLLB ${variant} model deleted`);
    } catch (e) {
      console.error("NLLB delete failed:", e);
      toast.error(`Delete failed: ${e}`);
    }
  }, []);

  const handleOpenFolder = async () => {
    const folder = settings?.output.folder;
    if (!folder) {
      toast.error(t("toolbar.noOutputFolder"));
      return;
    }
    try {
      await invoke("open_folder", { path: folder });
    } catch (e) {
      console.error("Failed to open folder:", e);
      toast.error(`Failed to open folder: ${e}`);
    }
  };

  const handleDownloadUpdate = async () => {
    if (!update?.url) return;
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open(update.url);
    } catch {
      // Fallback
      window.open(update.url, "_blank");
    }
  };

  return (
    <>
      <div className="flex items-center justify-between px-6 py-2 border-t border-border bg-card/50">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="sm"
            onClick={toggleSettings}
            className="text-muted-foreground hover:text-foreground text-xs gap-1.5"
          >
            <Settings className="w-4 h-4" strokeWidth={1.5} />
            {t("toolbar.settings")}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleOpenFolder}
            className="text-muted-foreground hover:text-foreground text-xs gap-1.5"
          >
            <FolderOpen className="w-4 h-4" strokeWidth={1.5} />
            {t("toolbar.output")}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={toggleLog}
            className="text-muted-foreground hover:text-foreground text-xs gap-1.5"
          >
            <ScrollText className="w-4 h-4" strokeWidth={1.5} />
            {t("toolbar.log")}
          </Button>
        </div>

        {/* Version + Status */}
        <div className="flex items-center gap-2">
          {update?.available && (
            <Badge
              variant="outline"
              className="cursor-pointer text-[10px] h-5 px-1.5 gap-1 border-orange-500/50 text-orange-500 hover:bg-orange-500/10"
              onClick={() => setShowDepsModal(true)}
            >
              {t("update.available", { version: update.version })}
            </Badge>
          )}
          {hasMissing && (
            <Badge
              variant="outline"
              className="cursor-pointer text-[10px] h-5 px-1.5 gap-1 border-destructive/50 text-destructive hover:bg-destructive/10"
              onClick={() => setShowDepsModal(true)}
            >
              <AlertTriangle className="w-3 h-3" />
              {t("deps.missingDeps")}
            </Badge>
          )}
          <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
            <span>SubFlow v{__APP_VERSION__}</span>
            <span
              onClick={() => setShowDepsModal(true)}
              className={cn(
                "inline-block w-2 h-2 rounded-full cursor-pointer",
                binStatus === null && "bg-muted-foreground/50",
                allOk && !update?.available && "bg-emerald-500 animate-pulse",
                allOk && update?.available && "bg-orange-500 animate-pulse",
                hasMissing && "bg-red-500 animate-pulse",
              )}
            />
          </div>
        </div>
      </div>

      {/* Dependencies Modal */}
      {showDepsModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div
            className="absolute inset-0 bg-black/50"
            onClick={() => setShowDepsModal(false)}
          />
          <div className="relative z-10 bg-card border border-border rounded-xl p-5 w-[460px] shadow-2xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-foreground">
                {t("deps.title")}
              </h3>
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={() => setShowDepsModal(false)}
              >
                <X className="w-3.5 h-3.5" />
              </Button>
            </div>

            {/* Update Section */}
            {update?.available && (
              <div className="mb-4 rounded-lg bg-orange-500/10 border border-orange-500/20 px-3 py-2.5">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-foreground">
                      {t("update.available", { version: update.version })}
                    </p>
                    {update.changelog && (
                      <p className="text-[11px] text-muted-foreground mt-1 line-clamp-3">
                        {update.changelog.slice(0, 200)}
                        {update.changelog.length > 200 ? "..." : ""}
                      </p>
                    )}
                  </div>
                  <Button
                    size="sm"
                    className="ml-3 shrink-0"
                    onClick={handleDownloadUpdate}
                  >
                    <ExternalLink className="w-3.5 h-3.5" />
                    {t("update.download")}
                  </Button>
                </div>
              </div>
            )}

            <div className="space-y-3">
              {/* yt-dlp */}
              <DepRow
                name="yt-dlp"
                description="YouTube subtitle downloader"
                available={binStatus?.ytdlp_available ?? false}
                path={binStatus?.ytdlp_path}
                t={t}
              />
              {/* ffmpeg */}
              <DepRow
                name="ffmpeg"
                description="Media processing"
                available={binStatus?.ffmpeg_available ?? false}
                path={binStatus?.ffmpeg_path}
                t={t}
              />

              {/* NLLB 600M Model */}
              <NllbModelRow
                name={t("deps.nllb600m")}
                description={t("deps.nllb600mDesc")}
                available={binStatus?.nllb_600m_available ?? false}
                path={binStatus?.nllb_600m_path}
                downloading={nllbDownloading === "600M"}
                progress={nllbDownloading === "600M" ? nllbProgress : null}
                onDownload={() => handleDownloadNllb("600M")}
                onDelete={() => handleDeleteNllb("600M")}
                t={t}
              />

              {/* NLLB 1.3B Model */}
              <NllbModelRow
                name={t("deps.nllb1_3b")}
                description={t("deps.nllb1_3bDesc")}
                available={binStatus?.nllb_1_3b_available ?? false}
                path={binStatus?.nllb_1_3b_path}
                downloading={nllbDownloading === "1.3B"}
                progress={nllbDownloading === "1.3B" ? nllbProgress : null}
                onDownload={() => handleDownloadNllb("1.3B")}
                onDelete={() => handleDeleteNllb("1.3B")}
                t={t}
              />
            </div>

            {hasMissing && (
              <Button
                className="w-full mt-4"
                size="sm"
                onClick={handleInstall}
                disabled={installing}
              >
                {installing ? (
                  <>
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    {t("deps.installing")}
                  </>
                ) : (
                  <>
                    <Download className="w-3.5 h-3.5" />
                    {t("deps.install")}
                  </>
                )}
              </Button>
            )}

            {allOk && !update?.available && (
              <p className="text-xs text-muted-foreground text-center mt-4">
                {t("deps.allGood")}
              </p>
            )}

            {/* Check for updates button */}
            <Button
              variant="ghost"
              size="sm"
              className="w-full mt-3 text-xs text-muted-foreground"
              onClick={checkNow}
              disabled={checking}
            >
              {checking ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  {t("update.checking")}
                </>
              ) : (
                <>
                  <RefreshCw className="w-3.5 h-3.5" />
                  {t("update.checkForUpdates")}
                </>
              )}
            </Button>
          </div>
        </div>
      )}
    </>
  );
}

function DepRow({
  name,
  description,
  available,
  path,
  t,
}: {
  name: string;
  description: string;
  available: boolean;
  path: string | null | undefined;
  t: (key: string) => string;
}) {
  return (
    <div className="flex items-center justify-between rounded-lg bg-secondary/50 px-3 py-2.5">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-foreground">{name}</span>
          <span className="text-[10px] text-muted-foreground">
            {description}
          </span>
        </div>
        {path && (
          <p className="text-[10px] text-muted-foreground truncate mt-0.5">
            {path}
          </p>
        )}
      </div>
      <div className="ml-3 flex-shrink-0">
        {available ? (
          <div className="flex items-center gap-1 text-emerald-500">
            <Check className="w-3.5 h-3.5" />
            <span className="text-[10px] font-medium">{t("deps.installed")}</span>
          </div>
        ) : (
          <div className="flex items-center gap-1 text-destructive">
            <AlertTriangle className="w-3.5 h-3.5" />
            <span className="text-[10px] font-medium">{t("deps.missing")}</span>
          </div>
        )}
      </div>
    </div>
  );
}

function NllbModelRow({
  name,
  description,
  available,
  path,
  downloading,
  progress,
  onDownload,
  onDelete,
  t,
}: {
  name: string;
  description: string;
  available: boolean;
  path: string | null | undefined;
  downloading: boolean;
  progress: ModelDownloadProgress | null;
  onDownload: () => void;
  onDelete: () => void;
  t: (key: string) => string;
}) {
  return (
    <div className="rounded-lg bg-secondary/50 px-3 py-2.5">
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-foreground">{name}</span>
            <span className="text-[10px] text-muted-foreground">{description}</span>
          </div>
          {path && (
            <p className="text-[10px] text-muted-foreground truncate mt-0.5">
              {path}
            </p>
          )}
          {downloading && progress && (
            <div className="mt-1.5">
              <div className="flex items-center justify-between text-[10px] text-muted-foreground mb-1">
                <span>{progress.file}</span>
                <span>{Math.round(progress.percent)}%</span>
              </div>
              <div className="w-full h-1.5 bg-secondary rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary rounded-full transition-all duration-300"
                  style={{ width: `${progress.percent}%` }}
                />
              </div>
            </div>
          )}
        </div>
        <div className="ml-3 flex-shrink-0">
          {available ? (
            <div className="flex items-center gap-2">
              <div className="flex items-center gap-1 text-emerald-500">
                <Check className="w-3.5 h-3.5" />
                <span className="text-[10px] font-medium">{t("deps.nllbReady")}</span>
              </div>
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={onDelete}
                className="text-muted-foreground hover:text-destructive"
              >
                <Trash2 className="w-3 h-3" />
              </Button>
            </div>
          ) : downloading ? (
            <div className="flex items-center gap-1 text-muted-foreground">
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              <span className="text-[10px] font-medium">{t("deps.nllbDownloading")}</span>
            </div>
          ) : (
            <Button
              variant="outline"
              size="sm"
              className="h-6 text-[10px] px-2"
              onClick={onDownload}
            >
              <Download className="w-3 h-3" />
              {t("deps.nllbDownload")}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
