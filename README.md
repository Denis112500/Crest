# Crest

A small pill at the top center of the screen, in the spirit of the iPhone's Dynamic Island. It shows what's playing (YouTube Music first), grows when you hover or click it, and hides when there's nothing to show. Windows 11 first; Linux (KDE Plasma on Wayland) later.

Built with [Tauri 2](https://v2.tauri.app/): a Rust backend and a TypeScript + Vite frontend drawn with plain CSS/SVG. Everything stays on your machine: no network calls, no telemetry, no API keys.

> Status: milestone (c) — the pill window behaves right and shows fake content; the Rust side reads YouTube Music through Windows SMTC and prints every change in the terminal. See `notes.md` for the running log.

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

- **Windows** (`media/windows_smtc/`): uses the System Media Transport Controls (SMTC), the same system that feeds the media flyout next to the volume slider. Browsers publish what a web page plays there through the Media Session API, which is how we see YouTube Music without any unofficial API. A dedicated background thread waits for SMTC events (no polling), so it uses no CPU while nothing changes.
- **Linux, later** (`media/linux_mpris/`): a second struct implementing the same `MediaSource` trait over MPRIS, the D-Bus standard that Linux media players and browsers use. A small factory picks the implementation for the current OS at compile time (`#[cfg(target_os = ...)]`), so nothing else in the app changes.

Window behavior works the same way: a small trait, `PillWindowPlatform`, for "stay out of the taskbar and Alt+Tab" and "show without taking focus". `pill_window/mod.rs` picks the implementation for the current OS with `#[cfg(target_os = ...)]`. The Windows implementation sets Win32 window styles itself and shows the window with `SW_SHOWNOACTIVATE`, because Tauri's own `show()` would take focus. On Wayland a normal window can't place itself or stay on top, so KDE will get its own implementation (layer-shell) in `pill_window/linux_layer_shell/`.

## Current file tree

```
Dynamic Island/
├─ CLAUDE.md                    standing rules for AI-assisted sessions
├─ README.md                    this file
├─ notes.md                     running project log
├─ .gitignore                   ignores node_modules, dist, build output
├─ .gitattributes               LF line endings everywhere (Windows and Linux)
├─ .claude/launch.json          dev-server config for Claude's preview pane (port 1420)
├─ .vscode/extensions.json      recommends the Tauri and rust-analyzer VS Code extensions
├─ package.json                 npm scripts and frontend dependencies
├─ tsconfig.json                strict TypeScript settings
├─ vite.config.ts               Vite dev server settings Tauri expects (fixed port 1420)
├─ index.html                   the page loaded into the pill window
├─ src/
│  ├─ main.ts                   frontend entry: builds the pill, sizes the window, then reveals it
│  ├─ frontendConstants.ts      the pill's size (single source of truth, Rust sizes the window from it)
│  ├─ vite-env.d.ts             lets TypeScript understand Vite imports such as CSS files
│  ├─ ipc/
│  │  ├─ ipcChannelNames.ts     names of the Rust commands the frontend calls
│  │  ├─ requestPillWindowPlacement.ts  asks Rust to size the window and center it at the top
│  │  └─ requestPillWindowReveal.ts     asks Rust to show the window without taking focus
│  ├─ pill/
│  │  └─ pillShellElement.ts    creates the black capsule element
│  └─ styles/
│     ├─ designTokens.css       colors, font, radius
│     └─ pillShell.css          transparent page + capsule shape
└─ src-tauri/
   ├─ Cargo.toml                Rust package and dependencies (`windows` crate only on Windows)
   ├─ build.rs                  Tauri's build step (reads tauri.conf.json at compile time)
   ├─ tauri.conf.json           app name, pill window flags, content security policy, bundling
   ├─ capabilities/default.json what the frontend is allowed to call
   ├─ icons/                    app icons (Tauri defaults for now; our own icon comes later)
   └─ src/
      ├─ main.rs                program entry; calls run_crest_app
      ├─ lib.rs                 wires the app: pill window, settings, media source, commands
      ├─ backend_constants.rs   every Rust constant (window, settings file, app filter, SMTC timing)
      ├─ user_settings_file.rs  reads the optional settings.json
      ├─ media_console_preview.rs  milestone (c) only: prints media snapshots in the terminal
      ├─ media/
      │  ├─ mod.rs              declares the module; picks this OS's implementation
      │  ├─ media_source.rs     trait: "watch media sessions and tell me what changed"
      │  ├─ media_session_snapshot.rs  title, artist, album, art, play state, timeline
      │  ├─ media_session_selector.rs  app-ID filter + which session to show (unit-tested)
      │  └─ windows_smtc/
      │     ├─ mod.rs
      │     ├─ smtc_media_source.rs        Windows MediaSource: starts the worker thread
      │     ├─ smtc_worker_thread.rs       WinRT setup + the event loop
      │     ├─ smtc_event_subscriptions.rs SMTC events → messages to the worker
      │     ├─ smtc_session_tracker.rs     known sessions, last activity, what was sent
      │     ├─ smtc_snapshot_reader.rs     WinRT properties → snapshot
      │     └─ smtc_thumbnail_reader.rs    album art → data URL, cached per track
      └─ pill_window/
         ├─ mod.rs              declares the module; picks this OS's implementation
         ├─ pill_window_platform.rs    trait: the OS-specific overlay behavior
         ├─ pill_window_placement.rs   top-center math in physical pixels (unit-tested)
         ├─ pill_window_commands.rs    the commands the frontend calls
         └─ windows_native/
            ├─ mod.rs
            └─ windows_pill_window_platform.rs  Win32: tool-window style, show without focus
```

## Running it

Prerequisites on Windows: Rust (`stable-msvc`), Microsoft C++ Build Tools with "Desktop development with C++", Node.js. WebView2 ships with Windows 11.

```powershell
npm install
npm run tauri dev
```

## Settings

Crest works without any settings. To change which apps it shows, create `%APPDATA%\dev.crest.pill\settings.json`:

```json
{
  "allowedMediaAppIdentifierFragments": ["_crx_cinhimbnkkghhklpknlkffjgod"]
}
```

A media session is shown only if its app ID contains one of the fragments (ignoring case). The default matches the YouTube Music web app in Chromium browsers. An empty list (`[]`) shows every app.
