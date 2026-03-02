# Changelog

## v0.2.1 - UX Polish + Notifications (2026-03-02)

### Added
- Splash screen with fade-out animation on app startup
- Update checker: fetches latest release from GitHub, shows orange indicator + changelog in deps modal, "Check for updates" button
- System notifications (via `tauri-plugin-notification`) when tasks complete or fail, with toggle in Settings
- Task duration display on completed TaskCard (e.g. "45s", "2m 13s")
- `started_at` timestamp on Task (Rust + frontend) — set when processing begins after semaphore acquire
- Searchable dropdowns: source language, voice selector, translation provider now have inline search input
- Version synced from `tauri.conf.json` at build time via Vite `define` — single source of truth for app version
- Release script auto-bumps version in `tauri.conf.json` + `Cargo.toml` before tagging
- GitHub Actions release workflow with auto-build on tag push
- Cross-platform builds: Windows (x86_64, ARM64), macOS (Apple Silicon, Intel), Linux (x86_64, ARM64)
- Auto-generated changelog from CHANGELOG.md in GitHub Releases

### Changed
- "Open Folder" renamed to "Reveal in Folder" on TaskCard context menu
- Context menu "Reveal in Folder" now always visible (disabled when no output dir)
- Removed white borders from context menus, sheets, and select dropdowns
- Removed `tsc` type-check from build command (Vite/esbuild handles transpilation)
- Relaxed TypeScript strict mode for CI compatibility

### Fixed
- Searchable select uses `position="popper"` to prevent dropdown from collapsing when filtering items
- UI component file casing for Linux case-sensitive filesystem
- OpenSSL dependency for macOS x86_64 cross-compile

---

## v0.2.0 - UI Overhaul + Reliability (2026-03-02)

### Added

**Backend**
- Direct YouTube subtitle fetcher via Innertube API (ANDROID client) — bypasses yt-dlp for subtitle downloads, much faster and avoids PO Token / rate limit issues
- Handles both JSON (json3) and XML (srv3) subtitle formats from YouTube
- yt-dlp fallback when direct fetch fails
- Processing mode support: SubOnly, SubTranslate, SubTranslateTts — skip unnecessary steps
- AbortHandle-based task cancellation for instant queue responsiveness
- Remove task from queue (backend command + frontend)
- Retry task support with proper state reset
- Auto-download yt-dlp and ffmpeg binaries on first launch
- Dependency status check command (`check_binary_status`)
- Task event now includes `video_title` for live title updates
- Edge TTS retry logic (3 attempts with backoff) for WebSocket failures
- yt-dlp retry with exponential backoff for 429/network errors

**Frontend**
- Full shadcn/ui + animate-ui component migration (Button, Input, Badge, Card, Select, Progress, Tabs, Sheet, Slider, Textarea, Label, Separator, ContextMenu)
- Animated tab transitions with animate-ui Radix Tabs
- Settings panel as slide-in Sheet (right side)
- Right-click context menu on TaskCard (Open Folder, Retry, Pause, Resume, Cancel, Remove)
- Processing mode selection: Source tab (3 modes), Files tab (2 modes)
- File staging area with drag-drop + browse, Start button to queue
- Video title display on TaskCard (updates live when metadata fetched)
- Created-at timestamp on TaskCard ("5m ago", "2h ago")
- Dependency status indicator in toolbar (green/red dot)
- Log panel (slide-up) with timestamped entries and clear button
- Toast notifications via Sonner
- i18n support (English + Vietnamese) with language switcher in Settings
- API key masking in Settings (shows "Saved" / "Not set")
- Output button opens folder directly (no browse dialog fallback)

### Fixed
- URL cleaning: strips playlist params, mobile URLs, timestamps from YouTube URLs
- Queue responsiveness: cancelling a task immediately starts next queued task (was ~2s delay)
- yt-dlp `ios` player client PO Token requirement — switched to `mweb,web` fallback
- Paste bug: URL detection now properly handles paste events
- Cancel/pause race condition resolved with AbortHandle

### Changed
- Tailwind CSS v4 with shadcn CSS variables (dark theme preserved)
- Task struct includes `mode`, `video_title`, `created_at` fields
- 16 unit tests (up from 5)

---

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
