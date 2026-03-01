import { useCallback, useEffect, useState } from "react";
import { ArrowLeft, Check, Loader2 } from "lucide-react";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { useUiStore } from "@/stores/useUiStore";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { TRANSLATION_PROVIDERS, TTS_PROVIDERS, OUTPUT_FORMATS, LANGUAGES } from "@/lib/constants";
import type { AppConfig } from "@/lib/types";

export function SettingsPanel() {
  const { settings, loadSettings, saveSettings, testConnection, saveApiKey } = useSettingsStore();
  const closeSettings = useUiStore((s) => s.closeSettings);
  const [local, setLocal] = useState<AppConfig | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [ttsApiKey, setTtsApiKey] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    if (settings) setLocal(settings);
  }, [settings]);

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
    if (!local || !apiKey) return;
    setTesting(true);
    setTestResult(null);
    try {
      const ok = await testConnection(
        local.translation.provider,
        apiKey,
        local.translation.base_url || undefined,
        local.translation.model || undefined,
      );
      setTestResult(ok);
    } catch {
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  }, [local, apiKey, testConnection]);

  if (!local) return null;

  const selectedProvider = TRANSLATION_PROVIDERS.find((p) => p.id === local.translation.provider);

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50" onClick={closeSettings} />

      {/* Panel */}
      <div className="relative w-full max-w-md bg-bg-primary border-l border-border-subtle overflow-y-auto">
        {/* Header */}
        <div className="sticky top-0 bg-bg-primary/95 backdrop-blur-sm border-b border-border-subtle px-6 py-4 flex items-center justify-between z-10">
          <button onClick={closeSettings} className="flex items-center gap-2 text-text-secondary hover:text-text-primary transition-colors">
            <ArrowLeft className="w-4 h-4" />
            <span className="text-sm">Settings</span>
          </button>
          <Button size="sm" onClick={handleSave} disabled={saving}>
            {saving ? <Loader2 className="w-4 h-4 animate-spin" /> : "Done"}
          </Button>
        </div>

        <div className="px-6 py-6 space-y-8">
          {/* Translation Section */}
          <section>
            <h3 className="text-xs font-semibold text-text-tertiary uppercase tracking-wider mb-4">
              Translation
            </h3>

            <div className="space-y-4">
              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">Provider</label>
                <Select
                  value={local.translation.provider}
                  onChange={(e) =>
                    setLocal({
                      ...local,
                      translation: { ...local.translation, provider: e.target.value },
                    })
                  }
                  options={TRANSLATION_PROVIDERS.map((p) => ({ value: p.id, label: p.name }))}
                />
              </div>

              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">API Key</label>
                <div className="flex gap-2">
                  <Input
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    placeholder="Enter API key..."
                  />
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={handleTest}
                    disabled={testing || !apiKey}
                  >
                    {testing ? (
                      <Loader2 className="w-4 h-4 animate-spin" />
                    ) : testResult === true ? (
                      <Check className="w-4 h-4 text-accent-success" />
                    ) : (
                      "Test"
                    )}
                  </Button>
                </div>
                {testResult === false && (
                  <p className="text-xs text-accent-error mt-1">Connection failed</p>
                )}
              </div>

              {selectedProvider?.hasBaseUrl && (
                <div>
                  <label className="text-sm text-text-secondary mb-1.5 block">
                    Base URL <span className="text-text-tertiary">(optional)</span>
                  </label>
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
                </div>
              )}

              {selectedProvider?.hasModel && (
                <div>
                  <label className="text-sm text-text-secondary mb-1.5 block">Model</label>
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
                </div>
              )}

              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">Source Language</label>
                <Select
                  value={local.translation.source_lang}
                  onChange={(e) =>
                    setLocal({
                      ...local,
                      translation: { ...local.translation, source_lang: e.target.value },
                    })
                  }
                  options={LANGUAGES.map((l) => ({ value: l.code, label: l.name }))}
                />
              </div>

              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">Target Languages</label>
                <div className="flex flex-wrap gap-2">
                  {LANGUAGES.filter((l) => l.code !== local.translation.source_lang).map((lang) => {
                    const selected = local.translation.target_langs.includes(lang.code);
                    return (
                      <button
                        key={lang.code}
                        onClick={() => {
                          const langs = selected
                            ? local.translation.target_langs.filter((c) => c !== lang.code)
                            : [...local.translation.target_langs, lang.code];
                          setLocal({
                            ...local,
                            translation: { ...local.translation, target_langs: langs },
                          });
                        }}
                        className={`px-3 py-1 text-xs rounded-full border transition-colors ${
                          selected
                            ? "border-accent-primary bg-accent-primary/20 text-accent-primary"
                            : "border-border-subtle text-text-tertiary hover:border-border-focus"
                        }`}
                      >
                        {lang.name}
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>
          </section>

          <hr className="border-border-subtle" />

          {/* TTS Section */}
          <section>
            <h3 className="text-xs font-semibold text-text-tertiary uppercase tracking-wider mb-4">
              TTS
            </h3>

            <div className="space-y-4">
              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">Provider</label>
                <Select
                  value={local.tts.provider}
                  onChange={(e) =>
                    setLocal({ ...local, tts: { ...local.tts, provider: e.target.value } })
                  }
                  options={TTS_PROVIDERS.map((p) => ({ value: p.id, label: p.name }))}
                />
              </div>

              {local.tts.provider !== "edge" && (
                <div>
                  <label className="text-sm text-text-secondary mb-1.5 block">API Key</label>
                  <Input
                    type="password"
                    value={ttsApiKey}
                    onChange={(e) => setTtsApiKey(e.target.value)}
                    placeholder="Enter TTS API key..."
                  />
                </div>
              )}

              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">Voice</label>
                <Input
                  value={local.tts.voice || ""}
                  onChange={(e) =>
                    setLocal({ ...local, tts: { ...local.tts, voice: e.target.value || null } })
                  }
                  placeholder="e.g., en-US-AriaNeural"
                />
              </div>

              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">
                  Speed ({local.tts.speed.toFixed(1)}x)
                </label>
                <input
                  type="range"
                  min="0.5"
                  max="2.0"
                  step="0.1"
                  value={local.tts.speed}
                  onChange={(e) =>
                    setLocal({ ...local, tts: { ...local.tts, speed: parseFloat(e.target.value) } })
                  }
                  className="w-full accent-accent-primary"
                />
              </div>
            </div>
          </section>

          <hr className="border-border-subtle" />

          {/* Output Section */}
          <section>
            <h3 className="text-xs font-semibold text-text-tertiary uppercase tracking-wider mb-4">
              Output
            </h3>

            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-sm text-text-secondary mb-1.5 block">Format</label>
                  <Select
                    value={local.output.format}
                    onChange={(e) =>
                      setLocal({ ...local, output: { ...local.output, format: e.target.value } })
                    }
                    options={OUTPUT_FORMATS.map((f) => ({ value: f.id, label: f.name }))}
                  />
                </div>
                <div>
                  <label className="text-sm text-text-secondary mb-1.5 block">Parallel Jobs</label>
                  <Select
                    value={String(local.queue.parallel_jobs)}
                    onChange={(e) =>
                      setLocal({
                        ...local,
                        queue: { ...local.queue, parallel_jobs: parseInt(e.target.value) },
                      })
                    }
                    options={[1, 2, 3, 4, 6, 8].map((n) => ({
                      value: String(n),
                      label: String(n),
                    }))}
                  />
                </div>
              </div>

              <div>
                <label className="text-sm text-text-secondary mb-1.5 block">Output Folder</label>
                <div className="flex gap-2">
                  <Input
                    value={local.output.folder}
                    onChange={(e) =>
                      setLocal({ ...local, output: { ...local.output, folder: e.target.value } })
                    }
                    className="font-mono text-xs"
                  />
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={async () => {
                      try {
                        const { open } = await import("@tauri-apps/plugin-dialog");
                        const dir = await open({ directory: true });
                        if (dir) {
                          setLocal({ ...local, output: { ...local.output, folder: dir as string } });
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
      </div>
    </div>
  );
}
