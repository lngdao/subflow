# SubFlow

A desktop subtitle studio for downloading, translating, and generating TTS audio from YouTube videos and local subtitle files.

Built with [Tauri v2](https://tauri.app/) + React + Rust.

## Features

- **YouTube subtitle download** — auto-subs, manual subs, with browser impersonation to bypass rate limits
- **Multi-provider translation** — Anthropic (Claude), OpenAI, Google Gemini, GLM (Zhipu AI), DeepL, LibreTranslate, NLLB-200
- **NLLB-200 local translation** — run translation locally via CTranslate2, no API key needed (600M and 1.3B models)
- **Text-to-Speech** — Edge TTS (free, 100+ languages) and OpenAI TTS
- **Batch processing** — queue multiple videos/files, parallel jobs, parallel language processing
- **Multi-language output** — translate to multiple target languages in a single task
- **Local subtitle files** — drag-and-drop SRT, VTT, TXT files
- **Processing modes** — Subtitle Only, Sub + Translation, Sub + Translation + TTS
- **YouTube auto-translated subtitles** — experimental support via yt-dlp impersonation
- **Desktop notifications** — get notified when tasks complete or fail

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
| **yt-dlp** | YouTube subtitle download | Yes |
| **ffmpeg** | Media processing | Yes |
| **curl_cffi** | YouTube browser impersonation (for auto-translated subs) | Yes (requires Python 3) |
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
