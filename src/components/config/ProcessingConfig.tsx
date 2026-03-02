import { useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "@/stores/useSettingsStore";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { LANGUAGES, OUTPUT_FORMATS } from "@/lib/constants";
import type { AppConfig } from "@/lib/types";

const SOURCE_LANGUAGES = LANGUAGES;
const TARGET_LANGUAGES = LANGUAGES.filter((l) => l.code !== "auto");

const DEFAULT_VOICES: Record<string, string> = {
  vi: "vi-VN-HoaiMyNeural",
  ja: "ja-JP-NanamiNeural",
  ko: "ko-KR-SunHiNeural",
  zh: "zh-CN-XiaoxiaoNeural",
  en: "en-US-AriaNeural",
  es: "es-ES-ElviraNeural",
  fr: "fr-FR-DeniseNeural",
  de: "de-DE-KatjaNeural",
  pt: "pt-BR-FranciscaNeural",
  ru: "ru-RU-SvetlanaNeural",
};

export function ProcessingConfig() {
  const { t } = useTranslation();
  const { settings, saveSettings, voiceList, loadVoiceList } =
    useSettingsStore();

  useEffect(() => {
    loadVoiceList();
  }, [loadVoiceList]);

  const voicesByLang = useMemo(() => {
    const map: Record<string, typeof voiceList> = {};
    for (const voice of voiceList) {
      // voice.language is like "vi-VN", extract prefix "vi"
      const lang = voice.language.split("-")[0].toLowerCase();
      if (!map[lang]) map[lang] = [];
      map[lang].push(voice);
    }
    return map;
  }, [voiceList]);

  if (!settings) return null;

  const update = (patch: Partial<AppConfig>) => {
    const updated = { ...settings, ...patch };
    saveSettings(updated);
  };

  const handleTargetToggle = (langCode: string) => {
    const current = settings.translation.target_langs;
    const next = current.includes(langCode)
      ? current.filter((c) => c !== langCode)
      : [...current, langCode];

    // Auto-assign default voice for newly added language
    const newVoices = { ...settings.tts.voices };
    if (!current.includes(langCode) && next.includes(langCode)) {
      if (!newVoices[langCode]) {
        newVoices[langCode] = DEFAULT_VOICES[langCode] || "";
      }
    }

    update({
      translation: { ...settings.translation, target_langs: next },
      tts: { ...settings.tts, voices: newVoices },
    });
  };

  const handleVoiceChange = (lang: string, voiceId: string) => {
    const newVoices = { ...settings.tts.voices, [lang]: voiceId };
    update({ tts: { ...settings.tts, voices: newVoices } });
  };

  return (
    <div className="space-y-5 px-5 py-4">
      {/* Source Language */}
      <div>
        <Label className="text-xs text-muted-foreground mb-3">
          {t("config.sourceLang")}
        </Label>
        <Select
          value={settings.translation.source_lang}
          onValueChange={(v) =>
            update({
              translation: { ...settings.translation, source_lang: v },
            })
          }
        >
          <SelectTrigger className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent searchable>
            {SOURCE_LANGUAGES.map((l) => (
              <SelectItem key={l.code} value={l.code}>
                {l.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* Target Languages */}
      <div>
        <Label className="text-xs text-muted-foreground mb-3">
          {t("config.targetLangs")}
        </Label>
        <div className="flex flex-wrap gap-1.5">
          {TARGET_LANGUAGES.filter(
            (l) => l.code !== settings.translation.source_lang,
          ).map((lang) => {
            const selected = settings.translation.target_langs.includes(
              lang.code,
            );
            return (
              <button
                key={lang.code}
                onClick={() => handleTargetToggle(lang.code)}
                className={`px-2.5 py-1 text-xs rounded-full border transition-colors cursor-pointer ${
                  selected
                    ? "border-primary bg-primary/20 text-primary"
                    : "border-border text-muted-foreground hover:border-ring"
                }`}
              >
                {lang.name}
              </button>
            );
          })}
        </div>
      </div>

      {/* Per-language voice selection */}
      {settings.translation.target_langs.length > 0 && (
        <div>
          <Label className="text-xs text-muted-foreground mb-3">
            {t("config.voicePerLang")}
          </Label>
          <div className="space-y-2">
            {settings.translation.target_langs.map((lang) => {
              const langVoices = voicesByLang[lang] || [];
              const currentVoice =
                settings.tts.voices[lang] || DEFAULT_VOICES[lang] || "";
              const langName =
                TARGET_LANGUAGES.find((l) => l.code === lang)?.name || lang;

              return (
                <div key={lang} className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground w-20 shrink-0">
                    {langName}
                  </span>
                  <Select
                    value={currentVoice}
                    onValueChange={(v) => handleVoiceChange(lang, v)}
                  >
                    <SelectTrigger className="flex-1 h-8 text-xs">
                      <SelectValue placeholder={t("config.selectVoice")} />
                    </SelectTrigger>
                    <SelectContent searchable>
                      {langVoices.length > 0 ? (
                        langVoices.map((voice) => (
                          <SelectItem
                            key={voice.id}
                            value={voice.id}
                            className="text-xs"
                          >
                            {voice.name}
                            {voice.gender && (
                              <span className="text-muted-foreground ml-1">
                                ({voice.gender})
                              </span>
                            )}
                          </SelectItem>
                        ))
                      ) : (
                        <SelectItem
                          value={currentVoice || DEFAULT_VOICES[lang] || "auto"}
                        >
                          {currentVoice || t("config.defaultVoice")}
                        </SelectItem>
                      )}
                    </SelectContent>
                  </Select>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Speed + Format row */}
      <div className="grid grid-cols-2 gap-3">
        <div>
          <Label className="text-xs text-muted-foreground mb-3">
            {t("config.speed")} ({settings.tts.speed.toFixed(1)}x)
          </Label>
          <Slider
            min={0.5}
            max={2.0}
            step={0.1}
            value={[settings.tts.speed]}
            onValueChange={([v]) =>
              update({ tts: { ...settings.tts, speed: v } })
            }
            className="mt-2"
          />
        </div>
        <div>
          <Label className="text-xs text-muted-foreground mb-3">
            {t("config.format")}
          </Label>
          <Select
            value={settings.output.format}
            onValueChange={(v) =>
              update({ output: { ...settings.output, format: v } })
            }
          >
            <SelectTrigger className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {OUTPUT_FORMATS.map((f) => (
                <SelectItem key={f.id} value={f.id}>
                  {f.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  );
}
