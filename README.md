# Crest

A small pill at the top center of the screen, in the spirit of the iPhone's Dynamic Island. It shows what's playing (YouTube Music first), grows when you hover or click it, and hides when there's nothing to show. Windows 11 first; Linux (KDE Plasma on Wayland) later.

Built with [Tauri 2](https://v2.tauri.app/): a Rust backend and a TypeScript + Vite frontend drawn with plain CSS/SVG. Everything stays on your machine: no network calls, no telemetry, no API keys.

> Status: version 1 complete. A compact pill (album art, title, playing bars) that springs open into a large view (art, title, artist, progress, previous/play-pause/next) on hover, click, or a new track; hides when the music has been paused for 30 s or the player closes; tray icon with Quit. See `notes.md` for the running log.

To quit Crest, use the tray icon (notification area, possibly behind the ^ arrow) → **Quit Crest**.

## How it works, in plain language

Crest has two halves that talk to each other:

- **The backend (Rust, `src-tauri/`)** knows what's going on in the system: what music is playing, where the screen is, when to show or hide the pill.
- **The frontend (TypeScript, `src/`)** runs inside a small web view in the pill window and only draws what the backend tells it, and sends button presses back.

They talk through **Tauri events** (backend → frontend: "here's what to show now") and **Tauri commands** (frontend → backend: "the user pressed Next").

### Activities and sources (the plugin idea)

The pill always shows one **activity**: right now, "music is playing". Activities come from **sources**. Music is the first source; a timer or Claude Code events could be others later.

- Every source implements the same small Rust trait, `ActivitySource`. It reports updates ("this is what I'd show, this is how important it is, is it still ongoing?") and handles actions (like "next-track"). A button press travels as `(activity kind, action name)`; the `ActivitySourceRegistry` hands it to the source of that kind, and the result comes back as an ordinary update.
- The **core** (`activity_core/`) keeps the latest update from each source and decides what the pill shows and whether it's visible. It never looks inside a source's content, so adding a new source doesn't change the core.
- On the frontend, a small registry maps each activity kind (like `"music"`) to the views that draw it.

### When the pill is on screen

The core also decides visibility, with rules in one small pure function (`pill_visibility_policy.rs`, unit-tested):

- something **ongoing** (music playing) → visible, and it stays visible;
- something **lingering** (music paused) → hidden after 30 seconds; a new track shows it again first;
- **nothing** (the player closed) → hidden after 3 seconds.

`pill_visibility_controller.rs` runs those countdowns and tells the frontend, which plays a short fade/shrink animation and then asks Rust to hide the native window (or shows the window first, then animates the pill in). A hide never happens while the mouse is over the pill.

### Media: one trait, one implementation per operating system

The music source doesn't talk to Windows directly. It uses a trait, `MediaSource`: "tell me when the current media session changes" and "play / pause / next / previous".

- **Windows** (`media/windows_smtc/`): uses the System Media Transport Controls (SMTC), the same system that feeds the media flyout next to the volume slider. Browsers publish what a web page plays there through the Media Session API, which is how we see YouTube Music without any unofficial API. A dedicated background thread waits for SMTC events (no polling), so it uses no CPU while nothing changes.
- **Linux, later** (`media/linux_mpris/`): a second struct implementing the same `MediaSource` trait over MPRIS, the D-Bus standard that Linux media players and browsers use. A small factory picks the implementation for the current OS at compile time (`#[cfg(target_os = ...)]`), so nothing else in the app changes.

### The window and the animation

The native window is always as big as the expanded pill (plus a little room for the spring's overshoot) and never moves. What changes is its **interactive area**: only the rectangle where the pill currently is takes the mouse; everywhere else, clicks go to the app below. Growing: the area widens first, then CSS animates the capsule. Shrinking: the capsule animates first, then the area shrinks. Resizing the real window during the animation would make the pill jump for a frame, because the web content re-lays itself out slightly after Windows resizes the window.

On the frontend, `pillStateMachine.ts` decides *when* the pill is compact or expanded (hover, leave, click, a new track), and `pillMorphController.ts` carries it out. Endless animations (the playing bars, the progress bar) run on slow timers (15 and 4 updates per second) rather than at the monitor's refresh rate; on a 240 Hz screen that is the difference between about 35% and 5% of a CPU core while music plays. Each activity kind provides a *view set* (a compact and an expanded view) through `activityViewRegistry.ts`, the frontend's plugin point.

### Platform traits

Window behavior works the same way as media: a small trait, `PillWindowPlatform`, for "stay out of the taskbar and Alt+Tab", "show without taking focus" and "only this rectangle takes the mouse". `pill_window/mod.rs` picks the implementation for the current OS with `#[cfg(target_os = ...)]`. The Windows implementation sets Win32 window styles itself and shows the window with `SW_SHOWNOACTIVATE`, because Tauri's own `show()` would take focus. On Wayland a normal window can't place itself or stay on top, so KDE will get its own implementation (layer-shell) in `pill_window/linux_layer_shell/`.

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
│  ├─ main.ts                   frontend entry: wires the pill together, then reveals the window
│  ├─ frontendConstants.ts      every size and delay (single source of truth; sent to Rust and CSS)
│  ├─ vite-env.d.ts             lets TypeScript understand Vite imports such as CSS files
│  ├─ ipc/
│  │  ├─ ipcChannelNames.ts     command, event and activity-kind names shared with Rust
│  │  ├─ listenForRustStateChanges.ts   receives Rust-owned state (listens, then asks once)
│  │  ├─ requestPillWindowPlacement.ts  asks Rust to size the window and center it at the top
│  │  ├─ requestPillInteractiveArea.ts  asks Rust which rectangle takes the mouse
│  │  ├─ requestPillWindowReveal.ts     asks Rust to show the window without taking focus
│  │  ├─ requestPillWindowConceal.ts    asks Rust to hide the window
│  │  └─ requestActivityAction.ts       sends a button press to the activity's Rust source
│  ├─ pill/
│  │  ├─ pillShellElements.ts          the black capsule and its compact/expanded layers
│  │  ├─ pillDimensionCssVariables.ts  hands the sizes from frontendConstants.ts to CSS
│  │  ├─ pillStateMachine.ts           when to be compact or expanded (hover, click, peek)
│  │  ├─ pillPointerInput.ts           mouse events → state machine
│  │  ├─ pillMorphController.ts        animates a state change and keeps the interactive area in step
│  │  ├─ pillVisibilityController.ts   animates showing/hiding and shows/hides the native window
│  │  ├─ waitUntilNextFrameIsPainted.ts  resolves once the current content is on screen
│  │  └─ pillContentPresenter.ts       puts the current activity's views into the layers
│  ├─ activities/
│  │  ├─ pillPresentationTypes.ts    the shape of what Rust sends
│  │  ├─ activityViewSet.ts          what an activity's views must provide
│  │  ├─ activityViewRegistry.ts     activity kind → its view set (plugin point)
│  │  ├─ nothingToShowViewSet.ts     shown when no activity has anything
│  │  ├─ createSvgIconElement.ts     builds an inline SVG icon
│  │  └─ music/
│  │     ├─ nowPlayingTypes.ts       the music payload's shape
│  │     ├─ musicViewSet.ts          the music activity's compact + expanded views
│  │     ├─ compactMusicView.ts      tiny art, title, bars
│  │     ├─ expandedMusicView.ts     large art, title, artist, progress, buttons
│  │     ├─ albumArtImage.ts         album art with a placeholder when missing
│  │     ├─ playbackBarsIndicator.ts bouncing bars while playing
│  │     ├─ playbackProgressBar.ts   position computed between updates; animates only when visible
│  │     └─ musicControlButtons.ts   previous / play-pause / next
│  └─ styles/
│     ├─ designTokens.css       every color, spacing, duration and the spring curve
│     ├─ pillShell.css          transparent page, capsule shape, the morph animation
│     └─ albumArtImage.css, compactMusicView.css, expandedMusicView.css,
│        playbackBarsIndicator.css, playbackProgressBar.css, musicControlButtons.css
└─ src-tauri/
   ├─ Cargo.toml                Rust package and dependencies (`windows` crate only on Windows)
   ├─ build.rs                  Tauri's build step (reads tauri.conf.json at compile time)
   ├─ tauri.conf.json           app name, pill window flags, content security policy, bundling
   ├─ capabilities/default.json what the frontend is allowed to call
   ├─ icons/                    app icons generated from crest-icon-source.svg (`npx tauri icon ...`)
   └─ src/
      ├─ main.rs                program entry; calls run_crest_app
      ├─ lib.rs                 wires the app: pill window, core, activity sources, commands
      ├─ backend_constants.rs   every Rust constant (window, settings, priorities, timings)
      ├─ ipc_channel_names.rs   event, activity-kind and action names shared with the frontend
      ├─ user_settings_file.rs  reads the optional settings.json
      ├─ system_tray.rs         tray icon with "Quit Crest"
      ├─ activity_core/
      │  ├─ mod.rs
      │  ├─ activity_source.rs         trait every activity plugin implements
      │  ├─ activity_update.rs         what a source reports (priority, ongoing, attention key, payload)
      │  ├─ activity_publisher.rs      a source's handle for reporting to the core
      │  ├─ activity_arbiter.rs        picks what the pill shows and notifies the frontend (unit-tested)
      │  ├─ activity_source_registry.rs   running sources by kind; routes actions (unit-tested)
      │  ├─ activity_action_command.rs    command: a button press for some activity kind
      │  ├─ pill_presentation_command.rs  lets the frontend ask what's showing right now
      │  ├─ pill_visibility_policy.rs     the show/hide rules as a pure function (unit-tested)
      │  ├─ pill_visibility_controller.rs runs the hide countdowns, reports visibility changes
      │  └─ pill_visibility_command.rs    lets the frontend ask whether the pill is visible
      ├─ activity_sources/
      │  ├─ mod.rs
      │  └─ music/
      │     ├─ mod.rs
      │     ├─ music_activity_source.rs     media snapshots → music activity
      │     └─ session_loss_grace_period.rs ignores the short session gap on track change (unit-tested)
      ├─ media/
      │  ├─ mod.rs              declares the module; picks this OS's implementation
      │  ├─ media_source.rs     trait: watch media sessions + send play/pause/next/previous
      │  ├─ media_session_snapshot.rs  title, artist, album, art, play state, timeline
      │  ├─ media_session_selector.rs  app-ID filter + which session to show (unit-tested)
      │  └─ windows_smtc/
      │     ├─ mod.rs
      │     ├─ smtc_media_source.rs        Windows MediaSource: starts the worker thread
      │     ├─ smtc_worker_thread.rs       WinRT setup + the event loop
      │     ├─ smtc_worker_message.rs      what can wake the worker (events, button presses)
      │     ├─ smtc_event_subscriptions.rs SMTC events → messages to the worker
      │     ├─ smtc_transport_commands.rs  play/pause, next, previous on a session
      │     ├─ smtc_session_tracker.rs     known sessions, last activity, what was sent
      │     ├─ smtc_snapshot_reader.rs     WinRT properties → snapshot
      │     └─ smtc_thumbnail_reader.rs    album art → data URL, cached per track
      └─ pill_window/
         ├─ mod.rs              declares the module; picks this OS's implementation
         ├─ pill_window_platform.rs    trait: the OS-specific overlay behavior
         ├─ pill_window_placement.rs   top-center math in physical pixels (unit-tested)
         ├─ pill_interactive_area.rs   CSS pixels → physical pixels for the interactive area (unit-tested)
         ├─ pill_window_commands.rs    the commands the frontend calls
         └─ windows_native/
            ├─ mod.rs
            └─ windows_pill_window_platform.rs  Win32: tool-window style, show without focus, hide, window region
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
