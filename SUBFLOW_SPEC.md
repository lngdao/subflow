# SubFlow - Subtitle Translation Studio

## Overview

Desktop app (Tauri) để translate subtitle YouTube sang nhiều ngôn ngữ và generate TTS audio. Target: automation workflow cho YouTube reup - 32 video/ngày, ~60 phút/video.

**Tên dự án:** SubFlow - "Subtitle workflow made simple"

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React + TypeScript + Tailwind CSS |
| Backend | Rust (Tauri) |
| Subtitle Extract | yt-dlp |
| Translation | Claude Haiku (via custom provider) hoặc GLM-5 (z.ai) |
| TTS | Edge TTS (free, nhiều ngôn ngữ) |

---

## AI Provider Configuration

### Translation Providers

| Provider | Type | API Key Required | Models | Cost | Notes |
|----------|------|------------------|--------|------|-------|
| **Claude Haiku** | LLM | Yes | claude-haiku-4-5-20251001 | $0.25/1M tokens | Via custom provider |
| **GLM-5** | LLM | Yes | glm-5, glm-4.7 | $80/3 months | z.ai package |
| **OpenAI** | LLM | Yes | gpt-4o-mini, gpt-4o | $0.15-2.50/1M tokens | Standard OpenAI API |
| **Gemini** | LLM | Yes | gemini-2.0-flash | Free tier / $0.075/1M | Google AI Studio |
| **DeepL** | Translation | Yes | N/A | $25/1M chars | Best quality for EU langs |

### TTS Providers

| Provider | API Key Required | Voices | Cost | Notes |
|----------|------------------|--------|------|-------|
| **Edge TTS** | No | 100+ langs | FREE | Microsoft Edge, best free option |
| **OpenAI TTS** | Yes | 6 voices | $15/1M chars | alloy, echo, fable, onyx, nova, shimmer |
| **ElevenLabs** | Yes | 1000+ | $5-22/month | Best quality, cloning support |
| **Azure TTS** | Yes | 400+ | $1/1M chars | Enterprise, neural voices |

### Provider Config Schema

```typescript
interface ProviderConfig {
  translation: {
    provider: 'claude' | 'glm' | 'openai' | 'gemini' | 'deepl';
    apiKey: string;
    baseUrl?: string;  // For custom providers
    model?: string;    // Provider-specific model
  };
  tts: {
    provider: 'edge' | 'openai' | 'elevenlabs' | 'azure';
    apiKey?: string;
    voice?: string;    // Provider-specific voice ID
    baseUrl?: string;
  };
}
```

### UI Settings Panel

```
┌─ AI Provider Settings ────────────────────────────────────┐
│                                                           │
│ TRANSLATION                                               │
│ Provider: [Claude Haiku (custom) ▼]                       │
│ API Key:  [••••••••••••••••••••••] [Test Connection]      │
│ Base URL: [https://anyrouter.top       ] (optional)       │
│ Model:    [claude-haiku-4-5-20251001  ▼]                  │
│                                                           │
│ ─────────────────────────────────────────────────────────│
│                                                           │
│ TTS                                                       │
│ Provider: [Edge TTS (Free) ▼]                             │
│ API Key:  (not required for Edge TTS)                     │
│ Voice:    [en-US-AriaNeural ▼]                            │
│ Speed:    [1.0 ▼] (0.5 - 2.0)                             │
│                                                           │
│ ─────────────────────────────────────────────────────────│
│                                                           │
│ [Save Settings]  [Reset to Defaults]                      │
└───────────────────────────────────────────────────────────┘
```

### Provider-specific Config

**Claude (via custom provider):**
```typescript
{
  provider: 'claude',
  apiKey: 'sk-xxx',
  baseUrl: 'https://anyrouter.top',
  model: 'claude-haiku-4-5-20251001'
}
```

**GLM (z.ai):**
```typescript
{
  provider: 'glm',
  apiKey: 'zai-xxx',
  model: 'glm-5'  // or 'glm-4.7'
}
```

**OpenAI:**
```typescript
{
  provider: 'openai',
  apiKey: 'sk-xxx',
  model: 'gpt-4o-mini'  // or 'gpt-4o'
}
```

**Gemini:**
```typescript
{
  provider: 'gemini',
  apiKey: 'AIza-xxx',
  model: 'gemini-2.0-flash'
}
```

**DeepL:**
```typescript
{
  provider: 'deepl',
  apiKey: 'xxx-xxx-xxx'
  // No model selection - DeepL handles automatically
}
```

### Settings Persistence

- Config stored in `~/.subflow/config.json`
- Encrypted API keys using OS keychain (via `keyring` crate)
- Per-profile support (work/personal presets)

---

## Features

### Core Features (MVP)

1. **Input**
   - Paste 1 hoặc nhiều YouTube URLs (batch mode)
   - Hoặc upload file subtitle trực tiếp (SRT, VTT, TXT)
   - Drag & drop support

2. **Subtitle Extraction**
   - Dùng yt-dlp để download subtitle
   - **Support formats:** SRT, VTT, TXT (input & output)
   - Auto-detect format từ file extension
   - Convert giữa các formats
   - Support cả manual subs và auto-generated
   - Fallback: auto-generated nếu không có manual

3. **Translation**
   - Select target languages (multi-select)
   - Language pairs: EN → VI, EN → JP, EN → KR, EN → CN, etc.
   - Preserve SRT timestamps
   - Chunking cho long subtitles (tránh token limit)

4. **TTS Generation**
   - Generate audio cho mỗi translated SRT
   - Match audio duration với SRT timing (adjust speed nếu cần)
   - Multiple voice options per language

5. **Output**
   - Folder structure:
     ```
     output/
     └── {video_id}/
         ├── original.srt (hoặc .vtt / .txt)
         ├── vi.srt / vi.vtt / vi.txt
         ├── vi.mp3
         ├── jp.srt / jp.vtt / jp.txt
         ├── jp.mp3
         └── ...
     ```
   - **Output format:** User chọn SRT, VTT, hoặc TXT

### Advanced Features (v2)

- Queue management (pause, resume, cancel)
- Progress tracking với estimated time
- Retry failed items
- Custom glossary/dictionary cho terms đặc biệt
- Speaker detection → different voices cho different speakers
- Export as ZIP

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Frontend (React)                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────────────────┐  │
│  │ URL     │  │ Config  │  │ Progress Dashboard  │  │
│  │ Input   │  │ Panel   │  │ (queue, status)     │  │
│  └─────────┘  └─────────┘  └─────────────────────┘  │
└───────────────────────┬─────────────────────────────┘
                        │ Tauri IPC
┌───────────────────────┴─────────────────────────────┐
│                   Backend (Rust)                     │
│  ┌─────────────────────────────────────────────┐    │
│  │              Task Orchestrator               │    │
│  │  - Queue management                          │    │
│  │  - Parallel processing (configurable)        │    │
│  │  - Progress tracking                         │    │
│  └───────┬─────────┬─────────┬─────────────────┘    │
│          │         │         │                       │
│  ┌───────▼───┐ ┌───▼───┐ ┌───▼───────┐              │
│  │ yt-dlp    │ │ LLM   │ │ Edge TTS  │              │
│  │ module    │ │ API   │ │ module    │              │
│  └───────────┘ └───────┘ └───────────┘              │
└─────────────────────────────────────────────────────┘
```

---

## Data Flow

```
1. User inputs YouTube URLs
2. For each URL:
   a. yt-dlp --write-subs --sub-format srt --sub-langs en --skip-download
   b. Parse SRT → extract text segments
   c. Chunk segments (max 4000 tokens/chunk)
   d. Send to LLM for translation (preserve context between chunks)
   e. Reconstruct SRT với translated text
   f. For each segment: Edge TTS generate audio
   g. Optionally: adjust audio speed to match SRT timing
3. Save all outputs to folder
```

---

## Subtitle Formats

### SRT (SubRip)
```srt
1
00:00:01,000 --> 00:00:04,000
Hello and welcome to this video.

2
00:00:04,500 --> 00:00:08,000
Today we're going to learn about...
```

### VTT (WebVTT)
```vtt
WEBVTT

00:00:01.000 --> 00:00:04.000
Hello and welcome to this video.

00:00:04.500 --> 00:00:08.000
Today we're going to learn about...
```

### TXT (Plain Text)
```
Hello and welcome to this video.
Today we're going to learn about...
```
Note: TXT format loses timing info - chỉ dùng cho transcript, không dùng cho video sync.

---

## SRT Format

```srt
1
00:00:01,000 --> 00:00:04,000
Hello and welcome to this video.

2
00:00:04,500 --> 00:00:08,000
Today we're going to learn about...
```

**Translation prompt template:**
```
You are a professional subtitle translator. Translate the following subtitles from {source_lang} to {target_lang}. 

IMPORTANT RULES:
1. Keep the same number of lines as input
2. Each line corresponds to one subtitle segment
3. Maintain natural speech patterns in {target_lang}
4. Keep timing-appropriate length (don't make subtitles too long)
5. Preserve any formatting markers like [Music], [Applause], etc.

Input subtitles:
{chunks}

Output ONLY the translated lines, one per line, numbered to match input.
```

---

## Cost Estimation (per video, ~60 min)

| Component | Cost |
|-----------|------|
| Translation (Claude Haiku) | ~$0.02-0.03 |
| TTS (Edge TTS) | $0 |
| **Total per video** | **~$0.02-0.03** |
| **Per day (32 videos)** | **~$0.64-0.96** |
| **Per month** | **~$19-29** |

With GLM-5 (z.ai $80/3 months package):
- Practically free for this use case
- ~75M chars/month included

---

---

## UI Design System

### Design Philosophy

**Vibe:** Minimalist, modern, creative tool aesthetic — inspired by Figma, Cursor, Adobe Creative Suite.

**Keywords:** Clean, Dark-first, Glassmorphism, Subtle animations, Icon-driven, Whitespace-rich

### Design References (Moodboard)

| App | What to learn |
|-----|---------------|
| **Cursor** | Dark theme, subtle gradients, minimal chrome |
| **Figma** | Floating panels, clean typography, tool-centric |
| **Linear** | Glassmorphism, smooth transitions, keyboard-first |
| **Arc Browser** | Playful but professional, rounded corners |
| **Raycast** | Spotlight-style interactions, command palette |

### Color Palette

```css
/* Dark Theme (Default) */
--bg-primary: #0D0D0F;      /* Deep charcoal */
--bg-secondary: #18181B;    /* Elevated surfaces */
--bg-tertiary: #27272A;     /* Cards, panels */
--border-subtle: #3F3F46;   /* Subtle dividers */
--border-focus: #71717A;    /* Focus rings */

/* Accent Colors */
--accent-primary: #8B5CF6;  /* Purple - main actions */
--accent-success: #10B981;  /* Green - completed */
--accent-warning: #F59E0B;  /* Amber - in progress */
--accent-error: #EF4444;    /* Red - errors */

/* Text */
--text-primary: #FAFAFA;    /* Headings */
--text-secondary: #A1A1AA;  /* Body text */
--text-tertiary: #71717A;   /* Muted, placeholders */

/* Glass Effect */
--glass-bg: rgba(39, 39, 42, 0.6);
--glass-border: rgba(255, 255, 255, 0.1);
--glass-blur: 12px;
```

### Typography

```css
--font-display: "Inter", system-ui;  /* Headings */
--font-body: "Inter", system-ui;     /* Body text */
--font-mono: "JetBrains Mono", monospace;  /* URLs, code */

--text-xs: 0.75rem;    /* 12px - labels, captions */
--text-sm: 0.875rem;   /* 14px - body small */
--text-base: 1rem;     /* 16px - body */
--text-lg: 1.125rem;   /* 18px - section headers */
--text-xl: 1.25rem;    /* 20px - page titles */
```

### Spacing & Radius

```css
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-6: 24px;
--space-8: 32px;
--space-12: 48px;

--radius-sm: 6px;
--radius-md: 10px;
--radius-lg: 16px;
--radius-xl: 24px;
```

---

## UI Layout

### Window Structure

```
┌─────────────────────────────────────────────────────────────────┐
│ ○ ○ ○                                                    ─ □ × │  ← Native title bar
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                                                         │   │
│   │                    MAIN CONTENT                         │   │
│   │                                                         │   │
│   │   • Drop zone (centered, large)                        │   │
│   │   • Task list (expandable)                             │   │
│   │   • Progress indicator (subtle, bottom)                │   │
│   │                                                         │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  ⚙ Settings    📁 Output    ⌨ Shortcuts    ◐ Theme           │  ← Bottom toolbar
└─────────────────────────────────────────────────────────────────┘
```

### Main View (Empty State)

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                                                                 │
│                         subflow                                 │
│                    ✦ subtitle studio ✦                         │
│                                                                 │
│         ┌───────────────────────────────────────────┐          │
│         │                                           │          │
│         │      📥 Drop files here                   │          │
│         │                                         │          │
│         │      or paste YouTube URLs              │          │
│         │                                         │          │
│         │      ───────────────────────────────    │          │
│         │      supported: .srt  .vtt  .txt        │          │
│         │                                           │          │
│         └───────────────────────────────────────────┘          │
│                                                                 │
│                         [ Browse Files ]                        │
│                                                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Main View (Processing)

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ┌─ Queue ───────────────────────────────────────────────────┐  │
│  │                                                           │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │ 🎬 How to Build an AI Agent - Full Tutorial        │  │  │
│  │  │ youtube.com/watch?v=abc123                          │  │  │
│  │  │                                                     │  │  │
│  │  │ EN → VI, JP, KR                                    │  │  │
│  │  │                                                     │  │  │
│  │  │ ████████████████░░░░░░░░░░  62%                    │  │  │
│  │  │ Translating to Japanese...  2m remaining           │  │  │
│  │  │                                                     │  │  │
│  │  │                              [⏸ Pause]  [✕ Cancel] │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                                                           │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │ ✅ Python Automation Tutorial                       │  │  │
│  │  │ EN → VI  •  Completed in 3m 24s                     │  │  │
│  │  │                                    [📁 Open Folder] │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                                                           │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │ ⏳ API Design Best Practices                        │  │  │
│  │  │ EN → VI, JP  •  Queued                              │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  [＋ Add More]                                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Settings Panel (Slide-over / Modal)

```
┌─────────────────────────────────────────────────────────────────┐
│  ← Settings                                              Done   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TRANSLATION                                                    │
│  ────────────                                                   │
│                                                                 │
│  Provider                                                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  ○ Claude Haiku                                         │   │
│  │  ○ GLM-5 (z.ai)                                        │   │
│  │  ○ OpenAI GPT-4o                                       │   │
│  │  ○ Gemini Flash                                        │   │
│  │  ○ DeepL                                               │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  API Key                                                        │
│  ┌─────────────────────────────────────────────────────┐       │
│  │ ••••••••••••••••••••••••••••••••                    │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                 │
│  Base URL (optional)                                            │
│  ┌─────────────────────────────────────────────────────┐       │
│  │ https://api.example.com                             │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  TTS                                                            │
│  ───                                                            │
│                                                                 │
│  Provider                                                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  ○ Edge TTS (Free)                                     │   │
│  │  ○ OpenAI TTS                                          │   │
│  │  ○ ElevenLabs                                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Voice                                                          │
│  ┌─────────────────────────────────────────────────────┐       │
│  │ en-US-AriaNeural (Female, Natural)            ▼     │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  OUTPUT                                                         │
│  ──────                                                         │
│                                                                 │
│  Format                    Folder                               │
│  ┌──────────────────┐     ┌────────────────────────────┐       │
│  │ SRT         ▼    │     │ ~/Downloads/subflow   📁   │       │
│  └──────────────────┘     └────────────────────────────┘       │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  [ Reset to Defaults ]                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Design Notes

**Buttons:**
- Primary: Filled with accent color, subtle hover glow
- Secondary: Ghost style, border only
- Tertiary: Text only, underline on hover

**Cards:**
- Subtle gradient background (top to bottom, 5% opacity diff)
- 1px border with glass effect
- Soft shadow: `0 4px 24px rgba(0,0,0,0.4)`

**Progress:**
- Gradient fill (accent to accent-lighter)
- Pulse animation on active items
- Smooth transitions (200ms ease)

**Icons:**
- Lucide icons or Phosphor icons
- 20px size, stroke-width 1.5
- Muted color by default, accent on hover

**Animations:**
- Page transitions: 300ms fade + slide
- Card hover: subtle lift + glow
- Progress: shimmer effect on loading

---

## Old UI Mockup (Reference)

```
┌────────────────────────────────────────────────────────────┐
│  SubFlow - Subtitle Translation Studio            [- □ ×]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌─ YouTube URLs ───────────────────────────────────────┐  │
│  │ https://youtube.com/watch?v=xxxxx                    │  │
│  │ https://youtube.com/watch?v=yyyyy                    │  │
│  │                                                      │  │
│  │                                    [+ Add] [Clear]   │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌─ Settings ───────────────────────────────────────────┐  │
│  │ Source: [English ▼]                                  │  │
│  │ Target: [☑ Vietnamese ☑ Japanese ☐ Korean ☐ Chinese]│  │
│  │                                                      │  │
│  │ LLM: (•) Claude Haiku  ( ) GLM-5                     │  │
│  │ TTS: (•) Edge TTS (Free)                            │  │
│  │ Parallel jobs: [4 ▼]                                 │  │
│  │ Output folder: [~/Downloads/srt-output] [Browse]     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  [▶ Start Processing]  [⏸ Pause]  [⏹ Stop]               │
│                                                            │
│  ┌─ Progress ───────────────────────────────────────────┐  │
│  │ ████████████░░░░░░░░░░░░░░░░░░░░░░░░ 35% (11/32)     │  │
│  │                                                      │  │
│  │ ✅ video_001 - Done (vi, jp)                         │  │
│  │ ✅ video_002 - Done (vi, jp)                         │  │
│  │ ⏳ video_003 - Translating to Vietnamese...          │  │
│  │ ⏸  video_004 - Queued                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## Rust Modules

### 1. `src-tauri/src/subtitle/mod.rs`
- SRT parsing và serialization
- Chunking logic
- Timestamp preservation

### 2. `src-tauri/src/youtube/mod.rs`
- yt-dlp wrapper
- Subtitle download
- Video metadata extraction

### 3. `src-tauri/src/translate/mod.rs`
- LLM API client (Claude Haiku / GLM-5)
- Chunk translation với context
- Error handling + retry

### 4. `src-tauri/src/tts/mod.rs`
- Edge TTS integration
- Audio file generation
- Speed adjustment (optional)

### 5. `src-tauri/src/queue/mod.rs`
- Task queue management
- Parallel execution
- Progress tracking

---

## Dependencies (Rust)

```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
anyhow = "1"
thiserror = "1"
chrono = "0.4"
regex = "1"
```

---

## File Structure

```
srt-translator/
├── src/                      # React frontend
│   ├── components/
│   │   ├── UrlInput.tsx
│   │   ├── Settings.tsx
│   │   ├── ProgressBar.tsx
│   │   └── TaskList.tsx
│   ├── hooks/
│   │   └── useTauri.ts
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── subtitle/
│   │   ├── youtube/
│   │   ├── translate/
│   │   ├── tts/
│   │   └── queue/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md
```

---

## MVP Scope (v1.0)

**Must have:**
- [ ] Single URL input
- [ ] Download SRT via yt-dlp
- [ ] Translate to 1 language (EN → VI)
- [ ] Generate TTS audio
- [ ] Save output to folder

**Nice to have:**
- [ ] Batch URLs
- [ ] Multi-language output
- [ ] Progress UI
- [ ] Settings persistence

---

## Success Criteria

1. **Accuracy**: Translation giữ nghĩa, natural sounding
2. **Speed**: < 5 phút/video (single language)
3. **Cost**: < $0.05/video với Claude Haiku
4. **Reliability**: 95%+ success rate, auto-retry on failure
5. **UX**: Simple, one-click operation cho daily workflow

---

## Next Steps

1. Initialize Tauri project: `npm create tauri-app@latest`
2. Implement yt-dlp wrapper in Rust
3. Implement SRT parser
4. Integrate Claude Haiku API (custom provider)
5. Integrate Edge TTS
6. Build minimal React UI
7. Test với real YouTube videos
8. Add batch processing + progress UI
