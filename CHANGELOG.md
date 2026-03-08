# Changelog

## v0.4.1 - Bugfixes (2026-03-08)

### Fixed
- yt-dlp/ffmpeg not detected when installed via Homebrew — macOS .app doesn't inherit shell PATH, now checks `/opt/homebrew/bin`, `/usr/local/bin` as fallback
- Update notification text overflow in Dependencies modal — changelog text now properly constrained with `line-clamp` and `break-all`
- CI: Windows ARM64 build failing — `--no-default-features` must be passed as cargo arg via `-- --no-default-features`, not as Tauri CLI arg
- CI: macOS x86_64 build — `macos-13` runner retired, switch to `macos-latest` cross-compile

### Changed
- Status dot animation: replaced `animate-pulse` with ping ring effect (dot stays solid, ring expands and fades out)

---

## v0.4.0 - YouTube Auto-Translation & Provider Cleanup (2026-03-08)

### Added

**Backend**
- YouTube auto-translated subtitle download via yt-dlp with `--impersonate Chrome` (browser TLS fingerprinting via `curl_cffi`)
- `download_translated_subtitle()` in downloader — fetches YouTube server-side translations for any target language
- `map_yt_sub_lang()` — maps app language codes to YouTube codes (`zh` → `zh-Hans`, `he` → `iw`, etc.)
- `get_ytdlp_path()` now prioritizes venv yt-dlp (with curl_cffi) over standalone binary
- Managed Python venv for yt-dlp + curl_cffi: `setup_ytdlp_env()` / `delete_ytdlp_env()` commands
- `BinaryStatus` tracks `curl_cffi_available` and `ytdlp_env_exists` for UI state
- `check_curl_cffi()` — checks impersonation availability (venv then system)
- Smart retry in orchestrator: caches `original.{format}` subtitle, skips re-download on retry
- YouTube translation pre-fetch step in orchestrator before per-language processing
- `reqwest` `cookies` feature enabled for cookie jar support

**Frontend**
- YouTube Impersonation (curl_cffi) row in Dependencies modal with install/delete
- `useYtTranslation` checkbox on Source tab (experimental YouTube auto-translated subtitles)
- `useYtTranslation` state persisted in UI store
- `setupYtdlpEnv()` / `deleteYtdlpEnv()` Tauri API bindings

### Changed
- **Translation provider registry restructured**: removed duplicate entries (`Claude Haiku` + `Anthropic Messages` merged into `Anthropic (Claude)`, `OpenAI GPT-4o` + `OpenAI Compatible` split properly)
- Provider definitions now include `defaultModel`, `modelPlaceholder`, `baseUrlPlaceholder` metadata
- Settings panel uses provider-specific placeholders for model and base URL inputs
- Default provider changed from `"claude"` to `"anthropic"` (backend keeps `"claude"` alias for backward compat)
- Switching provider now resets model to `null` to prevent stale model values leaking
- NLLB native: tuned CTranslate2 config — `beam_size: 1`, `ComputeType::AUTO`, uses half CPU cores (2–8 threads)

### Fixed
- Edge TTS `chunk_text` panic on multi-byte UTF-8: finds char-safe boundary before slicing
- YouTube 429 rate limiting on auto-translated subtitles — solved via yt-dlp browser impersonation
- Delete YouTube Impersonation showing "Ready" after restart when system yt-dlp had curl_cffi — now checks `ytdlp_env_exists` separately
- CI: macOS x86_64 build — `macos-13` runner retired, switch to `macos-latest` cross-compile

### Meta
- Added MIT LICENSE
- Added README.md

---

## v0.3.1 - NLLB Multi-Model Support (2026-03-04)

### Added

**Backend**
- `NllbModelVariant` enum: switch between `NLLB 600M` (~2.5 GB) and `NLLB 1.3B` (~5.5 GB) CTranslate2 models
- Per-variant model directories: `~/.config/subflow/models/nllb-600M/` and `nllb-1.3B/`
- Singleton re-initialization: switching model variant automatically drops old provider and loads new one
- `download_nllb_model` / `delete_nllb_model` commands now accept `variant` parameter
- `BinaryStatus` tracks both models independently (`nllb_600m_available`, `nllb_1_3b_available`)

**Frontend**
- Two separate NLLB model rows in Dependencies modal (600M and 1.3B) with independent download/delete/progress
- Model dropdown in Settings when NLLB provider is selected (600M default, 1.3B for higher quality)
- `NLLB_MODELS` constant for model selection UI
- i18n keys for both model variants (EN + VI)

### Changed
- NLLB provider now reads `model` field from config to determine variant (defaults to 600M)
- Progress events include variant-specific model key (`nllb_600m` / `nllb_1_3b`)
- `ct2rs` is now optional behind `nllb-native` feature flag for cross-platform compatibility

### Fixed
- CI: Windows x86_64 build — add `RUSTFLAGS=-C target-feature=+crt-static` for ct2rs linking
- CI: macOS x86_64 build — use `macos-13` (Intel) runner instead of cross-compiling from ARM
- CI: Windows ARM64 — disable `nllb-native` (CTranslate2 upstream doesn't support this platform)

---

## v0.3.0 - Native NLLB Translation + UX Improvements (2026-03-03)

### Added

**Backend**
- Native NLLB-200 translation via CTranslate2 FFI (`ct2rs` crate) — local translation without Docker/server
- Model manager: streaming download of NLLB-600M model (~1.2GB) from HuggingFace with progress events
- Actor-pattern NLLB provider: dedicated OS thread for CTranslate2 (not Send/Sync), mpsc channel communication
- Lazy singleton initialization for NLLB translator (loads once, stays warm across tasks)
- `download_nllb_model` / `delete_nllb_model` Tauri commands
- NLLB model status in `BinaryStatus` (available flag + path)
- `nllb_api` provider ID for existing HTTP-based NLLB server (renamed from `nllb`)
- LibreTranslate provider support

**Frontend**
- NLLB model download/delete UI in Dependencies modal with progress bar
- Processing mode badge on TaskCard (Sub, Sub+Translate, Sub+Translate+TTS)
- Standalone Test button for providers without API key or base URL (NLLB local)
- `NLLB-200 (Local)` and `NLLB-200 (Server)` as separate provider options

### Fixed
- Orphaned `.tts_chunks_*` directories now cleaned up at start of each language processing
- Previously, failed/cancelled SubTranslateTts tasks left TTS chunk dirs behind

### Changed
- `nllb` provider ID now routes to native CTranslate2 (was HTTP)
- Old HTTP NLLB provider available as `nllb_api`
- Added `ct2rs` (with sentencepiece feature) and `futures-util` dependencies

---

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
