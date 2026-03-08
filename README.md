# SubFlow

A desktop subtitle studio — download, translate, and generate TTS audio from online videos or local subtitle files.

Built with [Tauri v2](https://tauri.app/) + React + Rust.

## Features

- **Subtitle download** — YouTube, and any platform supported by yt-dlp (1000+ sites)
- **Local subtitle files** — drag-and-drop SRT, VTT, TXT files
- **Multi-provider translation** — Anthropic (Claude), OpenAI, Google Gemini, GLM (Zhipu AI), DeepL, LibreTranslate, OpenAI-compatible APIs
- **NLLB-200 local translation** — offline translation via CTranslate2, no API key needed (600M and 1.3B models)
- **Text-to-Speech** — Edge TTS (free, 100+ languages) and OpenAI TTS
- **Batch processing** — queue multiple videos/files with parallel jobs
- **Multi-language output** — translate to multiple target languages in a single task
- **Processing modes** — Subtitle Only, Sub + Translation, Sub + Translation + TTS
- **YouTube auto-translated subtitles** — experimental support via yt-dlp browser impersonation
- **Desktop notifications** — get notified when tasks complete or fail
- **i18n** — English and Vietnamese

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri v2 |
| Frontend | React 19, TypeScript, Tailwind CSS v4, Zustand |
| Backend | Rust |
| UI | shadcn/ui, Radix UI, Lucide icons, Sonner toasts |
| TTS | msedge-tts, OpenAI Audio API |
| Translation | Anthropic, OpenAI, Gemini, DeepL, LibreTranslate, CTranslate2 (NLLB) |

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) (stable)
- Tauri v2 system dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Development

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Runtime Dependencies

SubFlow manages these automatically via the in-app Dependencies panel:

| Dependency | Purpose | Auto-install |
|-----------|---------|--------------|
| **yt-dlp** | Video/subtitle download (1000+ sites) | Yes |
| **ffmpeg** | Media processing | Yes |
| **curl_cffi** | Browser impersonation for YouTube auto-translated subs | Yes (requires Python 3) |
| **NLLB-200 600M** | Local translation model (~2.5 GB) | Manual download |
| **NLLB-200 1.3B** | Higher quality local translation (~5.5 GB) | Manual download |

## Project Structure

```
src/                    # React frontend
  components/           # UI components
  stores/               # Zustand state management
  hooks/                # Custom React hooks
  lib/                  # Utilities, types, Tauri API bindings
  i18n/                 # Internationalization (en, vi)
src-tauri/              # Rust backend
  src/
    subtitle/           # SRT/VTT/TXT parser & writer
    youtube/            # yt-dlp integration, Innertube API
    translate/          # Translation providers
    tts/                # TTS providers (Edge, OpenAI)
    queue/              # Task orchestrator
    commands/           # Tauri IPC commands
```

## License

[MIT](LICENSE)
