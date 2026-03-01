# Changelog

## v0.1.0 - Initial Release (2026-03-01)

### Added

**Backend (Rust/Tauri v2)**
- Subtitle parser supporting SRT, VTT, TXT formats with auto-detection and round-trip fidelity
- YouTube subtitle download via yt-dlp (`--write-subs --write-auto-subs`)
- Video metadata extraction via `yt-dlp --dump-json`
- Translation module with 5 providers:
  - Claude (Anthropic Messages API, supports custom base_url)
  - OpenAI (Chat Completions API)
  - Gemini (Google generateContent API)
  - GLM-5 (OpenAI-compatible, z.ai)
  - DeepL (direct translation API)
- Translation chunker: splits subtitles into ~50 entry chunks, numbered prompt template, response validation
- Retry logic: exponential backoff (3 attempts, 1s/2s/4s) for all translation providers
- TTS module with 2 providers:
  - Edge TTS (free, 100+ languages, via msedge-tts crate)
  - OpenAI TTS (6 voices, /v1/audio/speech)
- Task queue orchestrator with semaphore-based concurrency control
- Real-time progress events via Tauri event system (`task-event`)
- Task lifecycle: Queued → Downloading → Translating → GeneratingTTS → Completed/Failed
- Pause, cancel, resume support for tasks
- Settings persistence at `~/.config/subflow/config.json`
- File-based API key storage at `~/.config/subflow/keys/`
- 15 Tauri IPC commands for full frontend-backend communication
- Output folder structure: `{output_dir}/{video_id}/{lang}.srt + {lang}.mp3`

**Frontend (React + TypeScript + Tailwind CSS v4)**
- Dark theme UI with glassmorphism design (bg #0D0D0F, accent #8B5CF6)
- Empty state: branded drop zone with drag-drop, paste URL detection, file browse
- Queue view: real-time task cards with progress bars, status icons, action buttons
- Settings panel: slide-over with Translation, TTS, Output sections
  - Provider selection, API key input with test connection
  - Target language multi-select chips
  - Voice, speed, output format, parallel jobs configuration
- Zustand state management (3 stores: task, settings, UI)
- Tauri event listener hook for real-time task updates

**Infrastructure**
- Tauri v2 project with React + Vite + TypeScript
- Tailwind CSS v4 with custom color palette
- 5 passing unit tests (SRT parse, VTT parse, TXT parse, format detection, SRT roundtrip)
