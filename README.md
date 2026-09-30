# Crest

A small pill at the top center of the screen, in the spirit of the iPhone's Dynamic Island. It shows what's playing (YouTube Music first), grows when you hover or click it, and hides when there's nothing to show. Windows 11 first; Linux (KDE Plasma on Wayland) later.

Built with [Tauri 2](https://v2.tauri.app/): a Rust backend and a TypeScript + Vite frontend drawn with plain CSS/SVG. Everything stays on your machine: no network calls, no telemetry, no API keys.

> Status: milestone (a) — an empty app that opens a window. See `notes.md` for the running log.

## How it works, in plain language

Crest has two halves that talk to each other:

- **The backend (Rust, `src-tauri/`)** knows what's going on in the system: what music is playing, where the screen is, when to show or hide the pill.
- **The frontend (TypeScript, `src/`)** runs inside a small web view in the pill window and only draws what the backend tells it, and sends button presses back.

They talk through **Tauri events** (backend → frontend: "here's what to show now") and **Tauri commands** (frontend → backend: "the user pressed Next").

### Activities and sources (the plugin idea)

The pill always shows one **activity**: right now, "music is playing". Activities come from **sources**. Music is the first source; a timer or Claude Code events could be others later.

- Every source implements the same small Rust trait, `ActivitySource`. It reports updates ("this is what I'd show, this is how important it is, is it still ongoing?") and handles actions (like "next track").
- The **core** (`activity_core/`) keeps the latest update from each source and decides what the pill shows and whether it's visible. It never looks inside a source's content, so adding a new source doesn't change the core.
- On the frontend, a small registry maps each activity kind (like `"music"`) to the views that draw it.

### Media: one trait, one implementation per operating system

The music source doesn't talk to Windows directly. It uses a trait, `MediaSource`: "tell me when the current media session changes" and "play / pause / next / previous".

- **Windows** (`media/windows_smtc/`): uses the System Media Transport Controls (SMTC), the same system that feeds the media flyout next to the volume slider. Browsers publish what a web page plays there through the Media Session API, which is how we see YouTube Music without any unofficial API.
- **Linux, later** (`media/linux_mpris/`): a second struct implementing the same `MediaSource` trait over MPRIS, the D-Bus standard that Linux media players and browsers use. A small factory picks the implementation for the current OS at compile time (`#[cfg(target_os = ...)]`), so nothing else in the app changes.

Window behavior works the same way: a small trait for "don't take focus, don't show in Alt+Tab". The Windows implementation sets Win32 window styles; on Wayland a normal window can't place itself or stay on top, so KDE will need its own implementation (layer-shell).

## Current file tree

```
Dynamic Island/
├─ CLAUDE.md                    standing rules for AI-assisted sessions
├─ README.md                    this file
├─ notes.md                     running project log
├─ .gitignore                   ignores node_modules, dist, build output
├─ .vscode/extensions.json      recommends the Tauri and rust-analyzer VS Code extensions
├─ package.json                 npm scripts and frontend dependencies
├─ tsconfig.json                strict TypeScript settings
├─ vite.config.ts               Vite dev server settings Tauri expects (fixed port 1420)
├─ index.html                   the page loaded into the pill window
├─ src/
│  └─ main.ts                   frontend entry (placeholder text for now)
└─ src-tauri/
   ├─ Cargo.toml                Rust package and dependencies
   ├─ build.rs                  Tauri's build step (reads tauri.conf.json at compile time)
   ├─ tauri.conf.json           app name, window settings, content security policy, bundling
   ├─ capabilities/default.json what the frontend is allowed to call
   ├─ icons/                    app icons (Tauri defaults for now; our own icon comes later)
   └─ src/
      ├─ main.rs                program entry; calls run_crest_app
      └─ lib.rs                 builds and runs the Tauri app
```

## Running it

Prerequisites on Windows: Rust (`stable-msvc`), Microsoft C++ Build Tools with "Desktop development with C++", Node.js. WebView2 ships with Windows 11.

```powershell
npm install
npm run tauri dev
```
