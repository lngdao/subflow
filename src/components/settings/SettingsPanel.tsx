import { useCallback, useEffect, useState } from "react";
import { Check, Loader2, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { useUiStore } from "@/stores/useUiStore";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/animate-ui/components/radix/sheet";
import { TRANSLATION_PROVIDERS, TTS_PROVIDERS, NLLB_MODELS } from "@/lib/constants";
import { getApiKeyPreview } from "@/lib/tauri";
import type { AppConfig } from "@/lib/types";

const APP_LANGUAGES = [
  { value: "vi", label: "Tiếng Việt" },
  { value: "en", label: "English" },
];

export function SettingsPanel() {
  const { t, i18n } = useTranslation();
  const { settings, loadSettings, saveSettings, testConnection, saveApiKey } =
    useSettingsStore();
  const isSettingsOpen = useUiStore((s) => s.isSettingsOpen);
  const closeSettings = useUiStore((s) => s.closeSettings);
  const [local, setLocal] = useState<AppConfig | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [ttsApiKey, setTtsApiKey] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [saving, setSaving] = useState(false);
  const [apiKeyPreview, setApiKeyPreview] = useState<string | null>(null);
  const [ttsKeyPreview, setTtsKeyPreview] = useState<string | null>(null);
  const [editingApiKey, setEditingApiKey] = useState(false);
  const [editingTtsKey, setEditingTtsKey] = useState(false);

  const selectedProvider = TRANSLATION_PROVIDERS.find(
    (p) => p.id === local?.translation.provider,
  );

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    if (settings) setLocal(settings);
  }, [settings]);

  useEffect(() => {
    if (local) {
      getApiKeyPreview(local.translation.provider)
        .then(setApiKeyPreview)
        .catch(() => setApiKeyPreview(null));
      if (local.tts.provider !== "edge") {
        getApiKeyPreview(`${local.tts.provider}_tts`)
          .then(setTtsKeyPreview)
          .catch(() => setTtsKeyPreview(null));
      }
    }
  }, [local?.translation.provider, local?.tts.provider]);

  const handleSave = useCallback(async () => {
    if (!local) return;
    setSaving(true);
    try {
      await saveSettings(local);
      if (apiKey) {
        await saveApiKey(local.translation.provider, apiKey);
      }
      if (ttsApiKey && local.tts.provider !== "edge") {
        await saveApiKey(`${local.tts.provider}_tts`, ttsApiKey);
      }
      closeSettings();
    } finally {
      setSaving(false);
    }
  }, [local, apiKey, ttsApiKey, saveSettings, saveApiKey, closeSettings]);

  const handleTest = useCallback(async () => {
    if (!local) return;
    // For providers without API key requirement, test with empty key
    const testKey = selectedProvider?.hasApiKey === false ? "" : apiKey;
    if (selectedProvider?.hasApiKey !== false && !testKey) return;
    setTesting(true);
    setTestResult(null);
    try {
      const ok = await testConnection(
        local.translation.provider,
        testKey,
        local.translation.base_url || undefined,
        local.translation.model || undefined,
      );
      setTestResult(ok);
    } catch {
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  }, [local, apiKey, testConnection, selectedProvider]);

  const handleLanguageChange = useCallback(
    (lang: string) => {
      i18n.changeLanguage(lang);
      localStorage.setItem("subflow_language", lang);
    },
    [i18n],
  );

  if (!local) return null;

  return (
    <Sheet open={isSettingsOpen} onOpenChange={(open) => !open && closeSettings()}>
      <SheetContent side="right" showCloseButton={false} className="w-full max-w-md overflow-y-auto">
        <SheetHeader className="flex-row items-center justify-between">
          <SheetTitle>{t("settings.title")}</SheetTitle>
          <SheetDescription className="sr-only">
            Configure application settings
          </SheetDescription>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon-xs"
              onClick={handleSave}
              disabled={saving}
              title={t("settings.done")}
            >
              {saving ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Check className="w-4 h-4" />
              )}
            </Button>
            <Button
              variant="ghost"
              size="icon-xs"
              onClick={closeSettings}
            >
              <X className="w-4 h-4" />
            </Button>
          </div>
        </SheetHeader>

        <div className="px-4 pb-6 space-y-6">
          {/* Language Switcher */}
          <section>
            <Label className="text-xs text-muted-foreground uppercase tracking-wider mb-3">
              {t("settings.language")}
            </Label>
            <Select
              value={i18n.language}
              onValueChange={handleLanguageChange}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {APP_LANGUAGES.map((l) => (
                  <SelectItem key={l.value} value={l.value}>
                    {l.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </section>

          <Separator />

          {/* Notifications */}
          <section>
            <Label className="text-xs text-muted-foreground uppercase tracking-wider mb-3">
              {t("settings.notifications")}
            </Label>
            <div
              className="flex items-center justify-between rounded-lg bg-secondary/50 px-3 py-2.5 cursor-pointer"
              onClick={() =>
                setLocal({
                  ...local,
                  notifications: {
                    ...local.notifications,
                    enabled: !local.notifications.enabled,
                  },
                })
              }
            >
              <span className="text-sm text-foreground">
                {t("settings.notifyOnComplete")}
              </span>
              <div
                className={`w-8 h-5 rounded-full transition-colors relative ${
                  local.notifications.enabled
                    ? "bg-primary"
                    : "bg-muted-foreground/30"
                }`}
              >
                <div
                  className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                    local.notifications.enabled
                      ? "translate-x-3.5"
                      : "translate-x-0.5"
                  }`}
                />
              </div>
            </div>
          </section>

          <Separator />

          {/* Translation Section */}
          <section>
            <Label className="text-xs text-muted-foreground uppercase tracking-wider mb-3">
              {t("settings.translation")}
            </Label>

            <div className="space-y-4">
              <div>
                <Label className="text-sm text-secondary-foreground mb-1.5">
                  {t("settings.provider")}
                </Label>
                <Select
                  value={local.translation.provider}
                  onValueChange={(v) => {
                    setLocal({
                      ...local,
                      translation: { ...local.translation, provider: v },
                    });
                    setApiKey("");
                    setEditingApiKey(false);
                  }}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent searchable>
                    {TRANSLATION_PROVIDERS.map((p) => (
                      <SelectItem key={p.id} value={p.id}>
                        {p.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label className="text-sm text-secondary-foreground mb-1.5">
                  {t("settings.apiKey")}
                </Label>
                {selectedProvider?.hasApiKey === false ? (
                  <p className="text-xs text-muted-foreground italic">
                    No API key required
                  </p>
                ) : apiKeyPreview && !editingApiKey ? (
                  <div className="flex items-center gap-2">
                    <div className="flex-1 rounded-md bg-secondary border border-border px-3 py-2 text-sm text-muted-foreground font-mono">
                      {apiKeyPreview}
                    </div>
                    <Button
                      variant="outline"
                      onClick={() => setEditingApiKey(true)}
                    >
                      {t("settings.change")}
                    </Button>
                  </div>
                ) : (
                  <div className="flex gap-2">
                    <Input
                      type="password"
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                      placeholder="Enter API key..."
                    />
                    <Button
                      variant="outline"
                      onClick={handleTest}
                      disabled={testing || !apiKey}
                    >
                      {testing ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : testResult === true ? (
                        <Check className="w-4 h-4 text-accent-success" />
                      ) : (
                        t("settings.test")
                      )}
                    </Button>
                  </div>
                )}
                {testResult === false && (
                  <p className="text-xs text-destructive mt-1">
                    Connection failed
                  </p>
                )}
              </div>

              {selectedProvider?.hasBaseUrl && (
                <div>
                  <Label className="text-sm text-secondary-foreground mb-1.5">
                    {t("settings.baseUrl")}{" "}
                    {selectedProvider?.hasApiKey !== false && (
                      <span className="text-muted-foreground">(optional)</span>
                    )}
                  </Label>
                  <div className="flex gap-2">
                    <Input
                      value={local.translation.base_url || ""}
                      onChange={(e) =>
                        setLocal({
                          ...local,
                          translation: {
                            ...local.translation,
                            base_url: e.target.value || null,
                          },
                        })
                      }
                      placeholder="https://api.example.com"
                    />
                    {selectedProvider?.hasApiKey === false && (
                      <Button
                        variant="outline"
                        onClick={handleTest}
                        disabled={testing}
                      >
                        {testing ? (
                          <Loader2 className="w-4 h-4 animate-spin" />
                        ) : testResult === true ? (
                          <Check className="w-4 h-4 text-accent-success" />
                        ) : (
                          t("settings.test")
                        )}
                      </Button>
                    )}
                  </div>
                  {selectedProvider?.hasApiKey === false && testResult === false && (
                    <p className="text-xs text-destructive mt-1">
                      Connection failed
                    </p>
                  )}
                </div>
              )}

              {selectedProvider?.hasModel && (
                <div>
                  <Label className="text-sm text-secondary-foreground mb-1.5">
                    {t("settings.model")}
                  </Label>
                  {local.translation.provider === "nllb" ? (
                    <Select
                      value={local.translation.model || "600M"}
                      onValueChange={(v) =>
                        setLocal({
                          ...local,
                          translation: {
                            ...local.translation,
                            model: v,
                          },
                        })
                      }
                    >
                      <SelectTrigger className="w-full">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {NLLB_MODELS.map((m) => (
                          <SelectItem key={m.id} value={m.id}>
                            {m.name} ({m.size})
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  ) : (
                    <Input
                      value={local.translation.model || ""}
                      onChange={(e) =>
                        setLocal({
                          ...local,
                          translation: {
                            ...local.translation,
                            model: e.target.value || null,
                          },
                        })
                      }
                      placeholder="Model name..."
                    />
                  )}
                </div>
              )}

              {/* Standalone Test button for providers with no API key and no base URL (e.g., NLLB local) */}
              {selectedProvider?.hasApiKey === false &&
                !selectedProvider?.hasBaseUrl && (
                  <div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleTest}
                      disabled={testing}
                      className="w-full"
                    >
                      {testing ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : testResult === true ? (
                        <Check className="w-4 h-4 text-accent-success" />
                      ) : (
                        t("settings.test")
                      )}
                    </Button>
                    {testResult === false && (
                      <p className="text-xs text-destructive mt-1">
                        Connection failed
                      </p>
                    )}
                  </div>
                )}
            </div>
          </section>

          <Separator />

          {/* TTS Section */}
          <section>
            <Label className="text-xs text-muted-foreground uppercase tracking-wider mb-3">
              {t("settings.tts")}
            </Label>

            <div className="space-y-4">
              <div>
                <Label className="text-sm text-secondary-foreground mb-1.5">
                  {t("settings.provider")}
                </Label>
                <Select
                  value={local.tts.provider}
                  onValueChange={(v) =>
                    setLocal({
                      ...local,
                      tts: { ...local.tts, provider: v },
                    })
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {TTS_PROVIDERS.map((p) => (
                      <SelectItem key={p.id} value={p.id}>
                        {p.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {local.tts.provider !== "edge" && (
                <div>
                  <Label className="text-sm text-secondary-foreground mb-1.5">
                    {t("settings.apiKey")}
                  </Label>
                  {ttsKeyPreview && !editingTtsKey ? (
                    <div className="flex items-center gap-2">
                      <div className="flex-1 rounded-md bg-secondary border border-border px-3 py-2 text-sm text-muted-foreground font-mono">
                        {ttsKeyPreview}
                      </div>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setEditingTtsKey(true)}
                      >
                        {t("settings.change")}
                      </Button>
                    </div>
                  ) : (
                    <Input
                      type="password"
                      value={ttsApiKey}
                      onChange={(e) => setTtsApiKey(e.target.value)}
                      placeholder="Enter TTS API key..."
                    />
                  )}
                </div>
              )}
            </div>
          </section>

          <Separator />

          {/* Performance Section */}
          <section>
            <Label className="text-xs text-muted-foreground uppercase tracking-wider mb-3">
              {t("settings.performance")}
            </Label>

            <div className="space-y-4">
              <div>
                <Label className="text-sm text-secondary-foreground mb-1.5">
                  {t("settings.parallelLangs")}
                </Label>
                <Select
                  value={String(local.queue.parallel_langs ?? 2)}
                  onValueChange={(v) =>
                    setLocal({
                      ...local,
                      queue: {
                        ...local.queue,
                        parallel_langs: parseInt(v),
                      },
                    })
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {[1, 2, 3, 4, 5].map((n) => (
                      <SelectItem key={n} value={String(n)}>
                        {n}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div
                className="flex items-center justify-between rounded-lg bg-secondary/50 px-3 py-2.5 cursor-pointer"
                onClick={() =>
                  setLocal({
                    ...local,
                    queue: {
                      ...local.queue,
                      pipeline_tts: !(local.queue.pipeline_tts ?? true),
                    },
                  })
                }
              >
                <span className="text-sm text-foreground">
                  {t("settings.pipelineTts")}
                </span>
                <div
                  className={`w-8 h-5 rounded-full transition-colors relative ${
                    (local.queue.pipeline_tts ?? true)
                      ? "bg-primary"
                      : "bg-muted-foreground/30"
                  }`}
                >
                  <div
                    className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                      (local.queue.pipeline_tts ?? true)
                        ? "translate-x-3.5"
                        : "translate-x-0.5"
                    }`}
                  />
                </div>
              </div>

              <div>
                <Label className="text-sm text-secondary-foreground mb-1.5">
                  {t("settings.ttsChunkSize")}
                </Label>
                <Select
                  value={String(local.queue.tts_chunk_size ?? 500)}
                  onValueChange={(v) =>
                    setLocal({
                      ...local,
                      queue: {
                        ...local.queue,
                        tts_chunk_size: parseInt(v),
                      },
                    })
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {[200, 300, 500, 750, 1000].map((n) => (
                      <SelectItem key={n} value={String(n)}>
                        {n} chars
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </section>

          <Separator />

          {/* Output Section */}
          <section>
            <Label className="text-xs text-muted-foreground uppercase tracking-wider mb-3">
              {t("settings.output")}
            </Label>

            <div className="space-y-4">
              <div>
                <Label className="text-sm text-secondary-foreground mb-1.5">
                  {t("settings.parallelJobs")}
                </Label>
                <Select
                  value={String(local.queue.parallel_jobs)}
                  onValueChange={(v) =>
                    setLocal({
                      ...local,
                      queue: {
                        ...local.queue,
                        parallel_jobs: parseInt(v),
                      },
                    })
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {[1, 2, 3, 4, 6, 8].map((n) => (
                      <SelectItem key={n} value={String(n)}>
                        {n}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label className="text-sm text-secondary-foreground mb-1.5">
                  {t("settings.folder")}
                </Label>
                <div className="flex gap-2">
                  <Input
                    value={local.output.folder}
                    onChange={(e) =>
                      setLocal({
                        ...local,
                        output: {
                          ...local.output,
                          folder: e.target.value,
                        },
                      })
                    }
                    className="font-mono text-xs"
                  />
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={async () => {
                      try {
                        const { open } = await import(
                          "@tauri-apps/plugin-dialog"
                        );
                        const dir = await open({ directory: true });
                        if (dir) {
                          setLocal({
                            ...local,
                            output: {
                              ...local.output,
                              folder: dir as string,
                            },
                          });
                        }
                      } catch {
                        // Cancelled
                      }
                    }}
                  >
                    Browse
                  </Button>
                </div>
              </div>
            </div>
          </section>
        </div>
      </SheetContent>
    </Sheet>
  );
}
