# Project notes

## Current state
- **Works:** milestone (a). `npm run tauri dev` opens a normal window titled "Crest".
- **In progress:** milestone (b), the pill window. Milestone (a) is committed as the first commit on `main`.
- **Broken:** nothing known.

---

## 2026-09-30 — Milestone (a): empty Tauri app runs
- **Done:**
  - Scaffolded from `create-tauri-app` 4.7.4 (template `vanilla-ts`) in a scratch folder, then copied only what we need.
  - Removed the demo "greet" command, the `opener` plugin and the logo assets.
  - Files: `package.json`, `tsconfig.json`, `vite.config.ts`, `index.html`, `src/main.ts`, `.gitignore`, `.vscode/extensions.json`, `src-tauri/{Cargo.toml, build.rs, tauri.conf.json, capabilities/default.json, icons/, src/main.rs, src/lib.rs}`, `README.md`, `CLAUDE.md`. `git init` on branch `main`.
  - Verified by testing: `npx tsc --noEmit` passes; first `cargo build` 3 min 27 s with no warnings; rebuild 0.8 s; window "Crest" appears, about 28 MB for the main process.
- **Learned:**
  - `npm run tauri dev` does two things: it starts Vite (serves `index.html` + TS at `http://localhost:1420` with hot reload), then `cargo run`s the Rust app, whose window loads that URL. Frontend edits reload instantly; Rust edits trigger a recompile.
  - The first Rust build compiles every dependency (about 400 crates); after that, cargo only recompiles what changed. Build output lives in `src-tauri/target/` (large, git-ignored).
  - `main.rs` is tiny and calls `lib.rs`: Tauri keeps the app in a library so the same code could also build for mobile. `windows_subsystem = "windows"` stops release builds from opening a console window.
  - `tauri.conf.json` is read at **compile time** by `build.rs` / `generate_context!()`, so config changes need a rebuild (the dev command does that automatically).
  - Capabilities (`capabilities/default.json`) are Tauri 2's permission system: the frontend may only call what's listed. We grant only `core:default`.
  - CSP (Content Security Policy) restricts what the page may load. Production allows only our own files, `data:` images (album art later) and Tauri's IPC. The dev policy also allows inline styles and the Vite hot-reload websocket.
- **Decisions:**
  - Window label `pill` (referenced by capabilities).
  - `withGlobalTauri: false`: we import `@tauri-apps/api` as modules instead of a global `window.__TAURI__`.
  - Bundle target `nsis` only (the Windows installer), not MSI.
  - Default Tauri icons for now; our own icon comes with the tray in milestone (g).
  - Dropped `serde` until we need it (no unused dependencies).
- **Problems:**
  - The first session was cut off mid-way, so files were checked one by one before continuing.
  - A PowerShell `Copy-Item -Destination {scriptblock}` line errored; it was redundant.
  - "Port 1420 is already in use" when starting the app's preview server. Cause: the `npm run tauri dev` test was still running in the background and its Vite server held 1420. Also, `.claude/launch.json` expected Vite's default port 5173. Fix: stopped the old run and set launch.json to 1420. Only one Vite server can hold 1420 at a time, so stop the preview before `npm run tauri dev` (and vice versa).
- **Open questions:** does the page render under the production CSP inside the Tauri window? In the plain-browser preview, "Crest is running" renders with no console errors (verified by testing), but the browser doesn't apply Tauri's CSP.
- **Next:** first commit, then milestone (b): turn this window into the pill (transparent, frameless, on top, top-center, no focus, not in the taskbar or Alt+Tab).

## 2026-09-30 — Toolchain installed
- **Done:** installed Visual Studio Build Tools 2022 (workload "Desktop development with C++", includes Windows SDK 10.0.26100) and Rustup 1.29.1 via winget. `rustup default stable-msvc` → rustc/cargo 1.98.1. A hello-world compiled and ran (verified by testing), so the linker and SDK are found.
- **Learned:**
  - `rustup` is the installer/updater for Rust toolchains; `cargo` builds and runs Rust projects; `rustc` is the compiler that cargo calls.
  - The `-msvc` toolchain links with Microsoft's `link.exe`, which is why the Build Tools are needed. The other Windows toolchain (`-gnu`) isn't what Tauri supports on Windows.
  - Terminals opened before the install don't see `cargo` on PATH. Open a new terminal (or restart VS Code) after installing.
- **Problems:** the previous session was cut off mid-install, and the log confirmed both installs finished (exit 0).
- **Next:** finish milestone (a).

## 2026-09-30 — Plan approved
- **Decisions:**
  - File tree and milestones (a)–(g) approved as proposed (see the architecture entry below).
  - On track change the pill **expands for 4 s** ("peek"), then returns to compact.
  - Hide **30 s after pausing**, and **3 s after the session closes** (after the track-change grace period).
  - Working name **Crest**, identifier `dev.crest.pill`. The user may rename it later: name in `tauri.conf.json` (`productName`, `identifier`), `package.json`, `Cargo.toml`, README.
- **Next:** finish the toolchain install, then scaffold milestone (a).

## 2026-09-30 — Proposed architecture decisions (pending approval)
- **Done:** proposed file tree and milestones in chat. No code written.
- **Decisions (proposed):**
  - SMTC runs on its own worker thread in the multithreaded COM apartment (MTA). WinRT event handlers only send a "something changed" message over a channel; the worker re-reads the session. Idle cost = a thread blocked on `recv()` = zero CPU. Rejected: polling on a timer.
  - Button presses from the frontend go through the same channel to the worker. Rejected: calling blocking WinRT `.join()` inside a Tauri command, because sync commands run on the main (UI, STA) thread and could freeze the window.
  - Window sizing: the native window is resized between a "compact" and an "expanded" size, and hover is detected with normal DOM `mouseenter`/`mouseleave`. Rejected: Coucou's fixed big window + click-through + 60 Hz cursor poll, because our pill is visible whenever music plays, so it would poll almost all the time.
  - Animations: CSS transitions with a spring-shaped `linear()` easing (supported by WebView2 153). No JavaScript animation loop, except the progress bar, and only while expanded and playing.
  - Activities reach the core as a generic update (kind, priority, is-ongoing, attention key, JSON payload), so a new source never changes the core. Trade-off: the payload is untyped inside the core; each source and its frontend view share a typed struct.
  - Default media app filter: substring `_crx_cinhimbnkkghhklpknlkffjgod` (YouTube Music PWA ID), configurable in a local settings file.
  - Use `windows = "0.62"`, the same major version Tauri 2.12 uses, so only one copy compiles.
- **Open questions:** see "Open questions" in the entries below.
- **Next:** approval of the tree and plan, then toolchain install, then milestone (a).

## 2026-09-30 — Research: reference project Coucou (read-only clone in `../reference/coucou`, commit 5ae7bd9)
- **Learned (verified by reading its code):**
  - Window: one Tauri window labelled `island`, configured in `tauri.conf.json` with `transparent`, `decorations: false`, `shadow: false`, `alwaysOnTop`, `skipTaskbar`, `focus: false`. The black shape is drawn by CSS inside a large transparent window.
  - After creation it adds Win32 extended styles itself: `WS_EX_NOACTIVATE` (clicks don't steal focus) and `WS_EX_TOOLWINDOW` (not in Alt+Tab). Why: Tauri's options alone don't guarantee this on Windows (matches the open Tauri issues below).
  - Placement: reads the monitor's physical position, size and scale factor, computes top-center in physical pixels, and sets the size twice because moving across displays can rescale the window.
  - Click-through: the big transparent window would block clicks to apps below, so it calls `set_ignore_cursor_events` outside the island shape. A window ignoring the mouse gets no mouse events, so a Rust thread polls `GetCursorPos` at 60 Hz and emits a `cursor` event. When hidden, the window shrinks to a 240×6 "wake strip" and the thread parks on a `Condvar` (zero CPU).
  - IPC: Rust → TS with `app.emit` / `emit_to(label, …)`, TS listens with `listen()`. TS → Rust through `invoke` wrapped in one `bridge.ts` that does nothing outside Tauri, so the UI can be developed in a plain browser.
  - Island states live in a small DOM-free state machine class with timers (`fsm.ts`): easy to reason about and test.
  - Tray: `TrayIconBuilder` + `Menu` of `MenuItem`s with IDs; `on_menu_event` matches the ID; `"quit" => app.exit(0)`. Needs the Cargo feature `tray-icon`.
  - Capabilities file grants only `core:default`; CSP is strict, with `img-src 'self' data: blob:`.
- **Decisions:** adopt the extended-styles approach, DPI-aware placement, the no-op-outside-Tauri IPC wrapper idea, and a DOM-free state machine. Don't adopt the fixed big window + cursor polling (see architecture entry), and don't copy its file organisation.
- **Problems found in it:** `lib.rs` (389 lines) and `island.rs` mix many concerns; `island.ts` is 785 lines; short names (`w`, `h`, `cv`, `p`, `m`); inline magic numbers; it depends on `windows 0.61` while Tauri uses `0.62`, so two copies compile. Only window and event patterns apply; it has no media code.

## 2026-09-30 — Research: Tauri 2 window behavior on Windows 11
- **Verified from docs** (tauri-utils `config.rs`, Tauri 2.12.0 current):
  - Keys: `transparent`, `decorations`, `alwaysOnTop`, `skipTaskbar`, `focus`, `focusable`, `shadow`, `resizable`, `visible`, `x`, `y`, `width`, `height`, `center`.
  - `shadow: true` on an undecorated window adds a 1 px white border and rounded corners on Windows 11 → we need `shadow: false`.
  - `transparent` note: on Windows, "noRedirectionBitmap" can help avoid a white flash.
- **Verified from GitHub issue status (2026-09-30):**
  - `skipTaskbar` not working on Windows: tauri#10422 **open**. Also doesn't cover Alt+Tab → set `WS_EX_TOOLWINDOW` ourselves.
  - Non-focusable window activated on `set_visible(true)`: fix is tao PR #1358, **open/unmerged** → set `WS_EX_NOACTIVATE` ourselves.
  - White flash when showing a hidden transparent window: tauri#14515, closed 2025-12-20; how it was fixed is not visible. **Unverified.**
  - Ghost titlebar on transparent windows on focus change: tauri#14764, closed 2026-01-24.
- **Unverified (test in milestone b):** DOM `mouseenter`/`mouseleave` and clicks work in a `WS_EX_NOACTIVATE` window; resizing a transparent WebView2 window doesn't flicker; the pill over borderless-fullscreen games.
- **Learned:** on Linux Wayland a normal app cannot position itself or stay on top; KDE needs the layer-shell protocol. This is why window behavior sits behind a small trait.

## 2026-09-30 — Research: what YouTube Music exposes through SMTC (verified by testing)
Setup: YouTube Music installed as a PWA in **Brave**. Probe: PowerShell 5.1 calling the same WinRT API, script in the session scratchpad.
- **App ID:** `Brave._crx_cinhimbnkkghhklpknlkffjgod`. `cinhimbnkkghhklpknlkffjgod` is the YT Music web-app ID; the prefix is the browser. Whether the suffix is identical in Chrome/Edge is **unverified**.
- **Present:** Title; Artist (joined, e.g. "Azahriah and DESH"); AlbumTitle; PlaybackType = Music; PlaybackStatus (Playing/Paused); Timeline Start/End/Position/LastUpdated/MinSeek/MaxSeek (End = duration); Controls flags (pause, toggle, next, previous, seek enabled; play disabled while playing).
- **Missing/empty:** AlbumArtist, Subtitle, TrackNumber (0), AlbumTrackCount (0), Genres, Shuffle, Repeat.
- **Thumbnail:** reference present. Bytes, format and pixel size **unverified**: PowerShell 5.1 can't pass WinRT stream interfaces to .NET. Verify in Rust in milestone (c).
- **Update behavior:**
  - Play/pause shows up within about 100 ms (limited by the probe's 100 ms sampling).
  - Timeline Position is **not** pushed continuously during steady playback, only on play/pause/seek/track change → the UI must extrapolate: `position + (now − LastUpdatedTime) × rate` while playing.
  - Dragging the seek bar causes bursts of about 10 updates/s → coalesce updates before emitting.
  - On track change the session **disappears for about 0.4 s** and comes back → don't hide on "no session" immediately; use a grace period.
- **Learned (strongly indicated, confirm in Rust):** in the default single-threaded COM apartment (STA) with no message pump, session values went stale (1 change in 60 s); in MTA they updated live. → SMTC must run on an MTA thread.
- **Learned:** docs list the `globalMediaControl` capability, but an unpackaged desktop process (the probe) reads sessions without it.

## 2026-09-30 — Research: calling SMTC from Rust (verified from docs)
- `windows` crate **0.62.2** is the latest published (GitHub master says 0.100.0, but that's not on crates.io yet). Tauri 2.12.0 depends on `windows ^0.62`.
- Cargo features: `Media_Control`, `Foundation`, `Foundation_Collections`, `Storage_Streams`.
- Manager: `GlobalSystemMediaTransportControlsSessionManager::RequestAsync()` → async op; `GetSessions()`, `GetCurrentSession()` ("the session the system believes the user would most likely want to control").
- Events, all returning an `i64` token for `Remove…(token)`: manager `SessionsChanged`, `CurrentSessionChanged`; session `MediaPropertiesChanged`, `PlaybackInfoChanged`, `TimelinePropertiesChanged`.
- Session reads: `SourceAppUserModelId`, `TryGetMediaPropertiesAsync`, `GetPlaybackInfo`, `GetTimelineProperties`. Controls: `TryPlayAsync`, `TryPauseAsync`, `TryTogglePlayPauseAsync`, `TrySkipNextAsync`, `TrySkipPreviousAsync` (each `IAsyncOperation<bool>`).
- Album art: `MediaProperties.Thumbnail()` → `IRandomAccessStreamReference` → `OpenReadAsync()` → read bytes (the `DataReader` path is **unverified** until milestone c).
- Waiting on async ops (windows-future 0.3): `.join()` blocks until done; `.await` also works (`IntoFuture`); `.when(callback)` runs a callback on completion.
- **Open question:** exact Rust type and units of `TimeSpan`/`DateTime` fields (WinRT uses 100 ns ticks; confirm in c).

## 2026-09-30 — Environment check (Windows 11 Home 26200)
- **Done:** checked installed tools, installed nothing.
- **Found:** Node v24.15.0, npm 11.12.1, git 2.55.0, WebView2 153.0.4234.32, winget 1.29. **Missing:** Rust (rustup/cargo/rustc), MSVC C++ Build Tools + Windows SDK (the Visual Studio installer exists but has no products).
- **Learned:** Tauri on Windows needs (1) Rust with the MSVC toolchain and (2) Microsoft C++ Build Tools with "Desktop development with C++", because Rust uses Microsoft's linker (`link.exe`) and the Windows SDK libraries. WebView2 ships with Windows 11.
- **Problems:** the `setup-windows-tauri` skill doesn't exist; no CLAUDE.md existed, so there were no conflicting rules.
- **Next:** install the toolchain (commands below).

---

## Commands
```powershell
# Toolchain (one-time). Tauri docs: "Desktop development with C++" + Rust stable-msvc
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install --id Rustlang.Rustup
rustup default stable-msvc

# Check versions / active toolchain
rustc --version; cargo --version; node --version
rustup show

# Update Rust later
rustup update

# Run the app (hot reload). Close the window or press Ctrl+C to stop.
npm run tauri dev

# Checks
npx tsc --noEmit                       # type-check the frontend
cd src-tauri; cargo build; cd ..       # compile the Rust side only
git status                             # what changed since the last commit

# "Port 1420 is already in use": find out which process holds it
Get-NetTCPConnection -LocalPort 1420 -State Listen | ForEach-Object { Get-Process -Id $_.OwningProcess }
```
