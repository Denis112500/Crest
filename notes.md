# Project notes

## Current state
- **Works:** **version 1 complete (milestones a–g)** plus fixes confirmed by the user: no white title bar/corners, layers fade in turn, a loading ring instead of Brave's logo on skip, only one Crest at a time. Pill at the top center shows YouTube Music; springs open on hover/click/new track; hides 30 s after pausing or ~4.5 s after the player closes; tray icon with Quit. CPU: 0% idle/paused/hidden, ~4.7% of one core while playing.
- **In progress:** Phase 1 → v0.2.0. Item 1 (hide during fullscreen apps/games): probe run done, `ABN_FULLSCREENAPP` confirmed for Valorant and YouTube Music; next is 1b (the real feature). Last release: v0.1.2 (https://github.com/Denis112500/Crest/releases/tag/v0.1.2).
- **Broken:** nothing known; button presses during a track change are now held and delivered (user confirmed). One unexplained observation in the (g) edge-case test didn't reproduce (see that entry).

---

## 2026-10-02 — Phase 1, item 1a: fullscreen probe
- **Decisions (user):** probe before building; when a fullscreen app comes to the front, the pill disappears **instantly** (no fade over the game); it comes back with the normal animation.
- **Done:** `dev-tools/fullscreen_event_watcher/` (Rust, same `windows` 0.62 as Crest, no new downloads): a hidden window registered as an appbar logs every `ABN_*` notification, plus an out-of-context `EVENT_SYSTEM_FOREGROUND` hook; each line has the front window's class/title, its monitor, `covers_monitor`, and `SHQueryUserNotificationState`. Ctrl+C or the time limit removes the appbar cleanly; re-registers on `TaskbarCreated`. Added to `dev-tools/README.md` and `.gitignore`. clippy clean.
- **Verified by testing (smoke run, 6 s):** with Valorant in front, the probe reported `quns=busy`, `covers_monitor=true` on `\\.\DISPLAY1(primary)` at 2560×1440, window class `VALORANTUnrealWindow`. So Valorant in its current mode counts as a normal (borderless/optimized) fullscreen window, not exclusive Direct3D (`d3d_exclusive_fullscreen`).
- **Learned:** a DPI-unaware program sees scaled coordinates on a scaled monitor, so the probe declares itself per-monitor DPI aware (Crest/Tauri already is); otherwise "covers the monitor" can be wrong.
- **Next:** user's test run; then 1b (build the real feature) based on the log.
- **Test run (verified by testing, 15 min of real play, log in `%TEMP%\crest_fullscreen_events.log`, local only: it contains window titles):**
  - Valorant (borderless/optimized, `quns=busy`): **every** entry into the game gave `ABN_FULLSCREENAPP open` (14×) and every alt-tab out gave `close`, 0–150 ms after the front window changed. Also when leaving to a minimized window or to the taskbar.
  - When Valorant becomes the front window it is 2560×**1439**; it reaches 2560×1440 ~10 ms later, when `open` arrives. **Option B alone would have missed it** → C confirmed.
  - YouTube Music (Brave app window) fullscreen on the primary monitor: `open` / `close` as expected.
  - Mode switch in Valorant's settings: `close` then `open` again 20 ms later → the real feature waits briefly before showing the pill again.
  - One phase reported `quns=d3d_exclusive_fullscreen`: there, a ~1 s alt-tab to another window sent **no** `close` (Windows still counted the game as fullscreen). Accepted: the pill then behaves like the taskbar.
  - Alt-tab puts short-lived helper windows in front (`ForegroundStaging` 0×0, `XamlExplorerHostIslandWindow` "Task Switching"); irrelevant for C. No false `open` in 15 min (clicking the taskbar `Shell_TrayWnd` didn't trigger one).
  - **Not tested:** fullscreen on the second monitor, desktop clicks (`Progman`/`WorkerW`), quitting the game.

---

## 2026-10-02 — Phase 1, item 1: how to detect a fullscreen app (research, no code yet)
- **Decision (user):** Phase 1 confirmed in this order: fullscreen hide → autostart → greyed-out buttons → WebView2 memory → v0.2.0 release on GitHub.
- **Options found:**
  - **A. `SHQueryUserNotificationState`** (verified from docs): returns `QUNS_BUSY` (fullscreen app or presentation mode) or `QUNS_RUNNING_D3D_FULL_SCREEN` (exclusive Direct3D). A question you have to ask, never a notification → only usable with polling. No monitor information.
  - **B. `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)`** + own check "does the front window cover its whole monitor?" (verified from docs): event-driven and knows the monitor, but only fires when the front window *changes*, so it misses a window that becomes fullscreen while already in front (F11, a game switching display mode).
  - **C. Register as an appbar (`SHAppBarMessage(ABM_NEW)`) and receive `ABN_FULLSCREENAPP`** (verified from docs): the shell's own fullscreen detection, the one that hides the taskbar; sent when the first fullscreen app opens (`lParam` TRUE) and the last one closes (FALSE). Event-driven, also catches F11. Weak spots (unverified, from forum/GitHub reports): no monitor information (this PC has 2 monitors: primary 2560×1440, second 1920×1080 on the right); clicking the desktop can look fullscreen (window classes `Progman`/`WorkerW`); the registration is lost when Explorer restarts (re-register on the `TaskbarCreated` message).
- **Proposed:** C as the trigger, plus our own check on each notification (front window covers the *pill's* monitor and isn't the desktop). Before building it, a small read-only probe in `dev-tools/` logs what Windows actually sends while the user plays Valorant (fullscreen and windowed fullscreen), uses F11 on each monitor, alt-tabs and clicks the desktop.
- **Learned:** the reference project (macOS) does the opposite: it deliberately stays visible over fullscreen apps, so nothing to reuse here. Anything that injects into another process (in-context hooks) is off-limits next to anti-cheat (Vanguard); A, B and C are all passive.
- **Open questions:** hide instantly (no fade over the game) or with the usual animation? Does exclusive fullscreen trigger C? (probe will show)

---

## 2026-10-02 — Roadmap (proposed, order not confirmed yet)
- **Phase 1: daily-driver polish → v0.2.0**
  1. Hide the pill (and stop the bars) while a fullscreen app or game is in front (the pill sits on Valorant's round timer).
  2. Start with Windows, toggled from the tray (`tauri-plugin-autostart` → ask before installing).
  3. Grey out buttons the player doesn't support (SMTC's control flags).
  4. Reduce WebView2 memory (~330 MB of 388 MB): measure, try options, keep what helps.
  5. ~~Save the debug scripts in `dev-tools/`.~~ Done 2026-10-02: `inspect_pill_window.ps1` (both style words + region, merged from two scripts), `evaluate_in_pill_page.ps1`, `record_pill_session.ps1` (capture area now follows the window), `list_media_sessions.ps1` (PowerShell 5.1), `smtc_thumbnail_watcher/` (Rust); `dev-tools/README.md` says when to use which. Verified by testing: all parse; the inspector, session lister and Rust watcher ran against the live app; the two page tools weren't re-run (need the debugging port).
- **Phase 2: groundwork → v0.3.0**
  6. Update CLAUDE.md for v2 (network only for enabled integrations, keys in Windows Credential Manager, free APIs only, no telemetry).
  7. Small settings window from the tray (autostart, allowed players, later integrations on/off).
  8. Several activities at once: design how they share the pill (switch / split / queue) before coding.
- **Phase 3: integrations, free only → v0.4.0+:** Claude Code sessions with Allow/Deny (local), weather (Open-Meteo, no key), calendar (private ICS link), GitHub (free token).
- **Phase 4: personality:** own visual identity (character, idle animations, sounds), never Coucou's Mochi.
- **Phase 5: Linux** (KDE Plasma, Wayland): MPRIS + layer-shell.
- **Decisions (user):** no GitHub issues or pull requests for now; the roadmap lives here and commits go straight to `main`.
- **Next:** user confirms or reorders; suggested first step is 1 (hide during fullscreen games).

---

## 2026-10-02 — "Toy project" → "hobby project"
- **Decision (user):** the README now calls Crest a personal hobby project instead of a toy project; it had outgrown "toy" (public releases, tests, plugin architecture). The disclaimer stays: works on the author's setup (Windows 11, YouTube Music in Brave), shared as-is, no support guaranteed.

---

## 2026-10-02 — Direction: integrations later, free APIs only
- **Decisions (user):** Crest will grow toward a Coucou-like app with integrations, but **only free APIs, never paid ones**. Not started yet; no integration chosen.
- **Open questions (decide before the first integration):**
  - CLAUDE.md currently says "no network calls, no API keys" and "not in v1: other activity sources". It must be updated first. Proposed: network only for integrations the user enables; keys in Windows Credential Manager, never in files or the repo; still no telemetry.
  - Candidates discussed: Claude Code sessions (local, no key), GitHub (free, personal token), weather via Open-Meteo (no key), calendar via private ICS link (no key).
- **Learned:** "free" ≠ "no key": some free APIs need nothing, others need a free personal token.

---

## 2026-10-01 — Release build 0.1.2
- **Done:** version 0.1.1 → 0.1.2 (`package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock`); `npm run tauri build` in 3 min 44 s → `Crest_0.1.2_x64-setup.exe` (1.35 MiB). Contains all of today's fixes (title bar, layer fades, loading ring, single instance, held button presses).
- **Verified by testing:** with the dev copy running, the release `crest.exe` exits by itself (exit code 0): dev and release share the identifier `dev.crest.pill`, so the single-instance guard covers both. A full release startup wasn't re-tested (it would have meant closing the user's dev copy).
- **Learned:** the single-instance lock is per app identifier, not per exe file: to try the installed version, quit the dev copy first (and the reverse).
- **Next:** user uploads the installer as GitHub Release v0.1.2.

---

## 2026-10-01 — Button presses during a track change are no longer lost
- **Problem:** "no media session to send PreviousTrack to" ×4 in the user's log. Brave drops the media session for about 0.5 s on every track change; a press in that gap had no session to go to and was thrown away, so fast repeated next/previous skipped fewer tracks than pressed.
- **Done:**
  - `media/pending_transport_commands.rs` (platform-independent, 2 tests): holds presses in order with their time; `take_still_relevant` hands back those younger than `PENDING_MEDIA_TRANSPORT_COMMAND_LIFETIME` (2 s) and empties the list.
  - `smtc_session_tracker.rs`: the buttons' target app (`button_target_source_app_identifier`) is no longer cleared when the session vanishes. A press with no session is held; when the session list changes and the target app's session is back, held presses are sent.
  - `smtc_transport_commands.rs` now reports send failures itself; `smtc_tracked_session.rs` (new) holds `TrackedSmtcSession` and `find_session_of_app`. Split because the tracker grew to 166 lines; it's 146 now.
  - clippy clean, 28/28 tests.
- **Learned:** **borrowing fields separately.** A method `&self -> &Session` borrows the whole struct, so you can't change another field while holding the result. A free function that takes only `&self.tracked_sessions` borrows just that field, and Rust allows changing `self.pending_transport_commands` at the same time.
- **Decisions:** stale presses (older than 2 s) are dropped instead of fired late; no timer is needed, since age is checked when the session comes back.

---

## 2026-10-01 — Loading ring instead of Brave's logo on skip
- **Problem (user):** after the cache fix, Brave's logo still flashed briefly on every skip. User's idea: show a loading wheel while it's the logo.
- **Measured (verified by testing, 6 skips, throwaway Rust probe reading SMTC every 60 ms):** on every skip the session disappears for ~0.5 s, comes back with the **same** 18 844-byte 256×256 PNG (Brave's logo) and the real cover (150×84 PNG) arrives **60–130 ms** later.
- **Done:**
  - `media/album_art_settle_gate.rs` (platform-independent, 3 tests): when the track identity (app, title, artist, album) changes, the art is held back for `ALBUM_ART_SETTLE_WINDOW` (300 ms, about 2× the slowest measured case) and the snapshot says `is_album_art_loading: true`. Doesn't recognize any specific logo, so it survives Brave changing its icon.
  - `smtc_worker_thread.rs`: waits with `recv_timeout` until the settle deadline, then publishes again (the player sends nothing new at that moment). Still no polling: it sleeps until a message or that one deadline.
  - `MediaSessionSnapshot` / `NowPlayingPayload`: new `isAlbumArtLoading`.
  - `albumArtImage.ts` + `albumArtImage.css`: a CSS ring spinner shown while loading (`display: none` otherwise, so its animation costs nothing then); new tokens `--album-art-spinner-*`.
  - clippy clean, 26/26 tests, tsc clean.
- **Learned:**
  - **Measure before guessing:** a 60-line probe showed the placeholder is byte-identical every time and how long it stays; that picked the design and the 300 ms value.
  - **PowerShell 5.1 can't hand WinRT streams to .NET** easily (cast errors); a tiny Rust program with the same `windows` crate was quicker.
  - **Timers without polling:** `recv_timeout(deadline - now)` lets a worker sleep until either a message or a known moment.
- **Decisions:** a time window rather than "skip the image if it's Brave's logo" (fragile) or "learn placeholders that get replaced" (shows the logo at least once per run).

---

## 2026-10-01 — Fixes from the user's screenshots: title bar, overlapping layers, Brave logo
- **Problems (user screenshots):** (1) a white classic title bar ("Crest" + ✕) across the top of the window, the real source of the "white corners"; (2) the small pill drawn on top of the big one ("songs overlap"); (3) the Brave logo as album art.
- **How they were found:** a recorder script polled the window style and the page state every ~120 ms with a screen capture per change while the user used the pill with the real mouse. Synthetic DOM events did *not* reproduce (1): it needs Windows' real click/focus handling.
- **Causes:**
  1. Verified by testing: the white bar appeared while the window style had **no** caption bits (`0x14000000`), so stripping styles wasn't enough. Tao makes the window transparent with `DwmEnableBlurBehindWindow`; anything classic GDI paints into such a window shows up white. With a custom window region, Windows repaints the classic frame on `WM_NCPAINT` / `WM_NCACTIVATE` (and the undocumented `WM_NCUAHDRAWCAPTION`/`WM_NCUAHDRAWFRAME`).
  2. Verified by testing: on collapse, the expanded layer faded out while the compact layer faded in at the same time; the recording shows `compact=0.47 expanded=0.53` mid-collapse.
  3. Verified from code, not reproduced live: the album art was cached per track (title+artist+album). Brave first publishes its own logo as the thumbnail, then the real cover under the same title, and the cache kept the logo for the whole song.
- **Done:**
  - New `pill_window/windows_native/classic_frame_painting_blocker.rs`: a Win32 subclass (`SetWindowSubclass`, needs the `Win32_UI_Shell` feature of the existing `windows` crate) that swallows the frame-painting messages and passes `WM_NCACTIVATE` on with `lParam = -1` ("don't repaint") so tao's focus tracking still works. Installed from `prepare_pill_window_as_overlay`.
  - `pillShell.css`: the layers take turns; the incoming one waits `--pill-content-fade-duration` until the outgoing one is gone. Verified by testing: sampled every 20 ms, the overlap (smaller of the two opacities) is 0 when expanding and collapsing.
  - `smtc_thumbnail_reader.rs`: per-track cache removed; the thumbnail is read on every snapshot (only on player events, a few times per song). `SmtcAlbumArtCache` and `AlbumArtTrackKey` are gone.
  - clippy clean, 23/23 tests, tsc clean.
- **Learned:**
  - **Subclassing** = putting your own function in front of a window's message handler; you handle some messages and pass the rest on with `DefSubclassProc`. It's how you change the behavior of a window someone else (here tao) created.
  - **"GDI on glass":** in a DWM blur-behind window, classic GDI drawing (which knows nothing about transparency) comes out white/see-through instead of opaque.
  - **Reproduce with the real input:** simulated events skip Windows' own activation and painting; bugs there only show with a real mouse.
  - **Caches need the right key:** "same song" isn't "same picture" when the source updates the picture later.

---

## 2026-10-01 — Only one Crest at a time (single-instance plugin)
- **Problem (reported by the user):** in a test right after the corner fix, songs overlapped, the pill lagged, buttons barely worked, play/pause didn't work, and the corners were still there. Most likely cause (unverified, the copies were gone before I could check): two older release copies from 21:47 were still running under the new dev copy, so three pills were stacked in one spot. Each animates on its own; a click goes to whichever window is on top at that point; the old copies don't have the corner fix.
- **Done:** added the official `tauri-plugin-single-instance` 2.5 (Rust only, no npm package; user approved the install). Registered first in `lib.rs`; the "another copy was started" callback does nothing, because the pill only appears with music, so there's nothing to bring forward. Verified by testing: with one copy running, a second `crest.exe` exits by itself (exit code 0) within 4 s; clippy clean, 23/23 tests.
- **Learned:** a Tauri plugin is added on the Rust side with `.plugin(…)` on the builder; some plugins also have an npm package for a JS API, this one doesn't. The plugin must be registered first so the second copy exits before creating windows or a tray icon. On Windows it uses a named mutex plus a message to the first copy.
- **Also found (from the user's log):** "no media session to send PreviousTrack to" ×4. During a track change YouTube Music's session disappears for about 0.4 s, and a button press in that gap is dropped. To fix next: hold the press until the session is back.

---

## 2026-10-01 — Fix: white "old app" corners after clicking the pill
- **Problem (reported by the user):** after the first click on the pill, white square corners appeared around it, like a classic-Windows app frame.
- **Cause (verified from the window's styles; the visual link is unverified until the user retests):** the pill window still had `WS_CAPTION` and `WS_SYSMENU` (style `0x4C80000`). Tauri/tao hides an undecorated window's title bar by giving the frame zero size, but keeps the caption styles. Our `SetWindowRgn` (interactive area) makes Windows stop drawing the modern DWM frame, so on a click or activation change it paints the classic frame instead.
- **Done:** `windows_pill_window_platform.rs` → `prepare_pill_window_as_overlay` also clears `WS_CAPTION | WS_SYSMENU` from `GWL_STYLE`, then calls `SetWindowPos(SWP_FRAMECHANGED …)` so Windows re-reads the frame. Verified by testing: the dev window's style is now `0x4000000` (no caption), position and size unchanged; clippy clean, 23/23 tests pass.
- **Learned:**
  - A Win32 window has two style words: `GWL_STYLE` (frame, caption, borders) and `GWL_EXSTYLE` (tool window, topmost, no-activate…). We had only fixed the extended one.
  - "Decorations off" in Tauri doesn't mean "no frame styles"; a custom window region turns off the modern frame drawing and exposes the old one.
  - Style changes affecting the frame need `SetWindowPos` with `SWP_FRAMECHANGED`, or Windows keeps using the cached frame.
- **Correction (same day):** not sufficient. The white bar still appeared with the caption bits removed; the real cause is classic frame painting into a blur-behind window, fixed with a subclass (see "Fixes from the user's screenshots"). The style change stays, as it leaves Windows less frame to paint.
- **Also noticed:** two copies of the release `crest.exe` were running at once (both started 21:47). Each shows its own pill. A single-instance guard would prevent this (needs the `tauri-plugin-single-instance` crate → ask first).

---

## 2026-10-01 — Publishing on GitHub
- **Done:** added `LICENSE` (MIT); README gained "Installing" (installer from Releases, SmartScreen note), build command, and a "License" section (toy project, not affiliated with Apple/Google/YouTube). Checked the tracked files for personal data before publishing: no emails, no personal paths, no build output.
- **Decisions:** public repo, MIT license. The user creates the empty repo on github.com; no GitHub CLI installed. The installer goes into a GitHub Release (uploaded by hand), not into git: build output never belongs in the repo.
- **Done (later):** pushed to https://github.com/Denis112500/Crest. Before the push, every commit's author email was changed from the personal Gmail to the GitHub private address (`196484913+Denis112500@users.noreply.github.com`, set as this repo's `user.email`); commit hashes changed, dates and messages didn't.
- **Learned:** every commit stores its author's name and email; on a public repo anyone can read them. GitHub's "noreply" address (Settings → Emails) still links commits to the account. Rewriting history is only safe before anyone else has it, i.e. before the first push.
- **Learned:** `git remote add origin <url>` links the local repo to GitHub; `git push -u origin main` uploads it and makes later `git push` calls go there. Git for Windows' Credential Manager handles the GitHub login in the browser on the first push.

---

## 2026-10-01 — First release build (version 0.1.1)
- **Done:**
  - Version raised from 0.1.0 to 0.1.1 in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`.
  - `npm run tauri build`: Rust release compile took 4 min 59 s; the result is `crest.exe` (4.4 MB) and the installer `Crest_0.1.1_x64-setup.exe` (1.34 MB).
  - Verified by testing: the release exe starts, its page loads from `http://tauri.localhost/` under the production CSP, and the pill shell is built (hidden, since nothing was playing).
- **Learned:**
  - **Dev vs release:** `tauri dev` loads the page from Vite (`localhost:1420`) with hot reload and `devCsp`; `tauri build` bakes `dist/` into the exe, which serves it as `tauri.localhost` with the strict `csp`. The release profile (LTO, `codegen-units=1`) is slow to compile but makes a small, fast exe.
  - **The exe is a snapshot:** to change the app, edit the code, raise the version, rebuild, quit the installed Crest from the tray, and run the new installer over the old one (same identifier `dev.crest.pill`, so it upgrades in place). Settings in `%APPDATA%\dev.crest.pill\` are untouched.
  - **The installer is NSIS** (`bundle.targets = ["nsis"]`); the installer is smaller than the exe because it's compressed. WebView2 is already part of Windows 11, so it isn't bundled.
  - **Unsigned app:** Windows SmartScreen warns on first run ("More info" → "Run anyway"). Code signing costs money; not worth it for a personal app.
- **Problems:**
  - On its first bundle, the Tauri CLI downloaded NSIS 3.11 and `nsis_tauri_utils.dll` from Tauri's GitHub releases into its own tools cache (`%LOCALAPPDATA%\tauri\`). It's build-time tooling only, nothing in the app itself, but it happened without asking first. Next time it's already there.
  - My CDP test script only looked for the dev URL; the release page is `tauri.localhost`.
- **Next:** install it and use it day to day; autostart with Windows is still an open idea.

---

## 2026-09-30 — Milestone (g): hide/show logic, tray icon, app icon, edge cases
- **Done:**
  - Rust core: `pill_visibility_policy.rs` (pure rules; 5 tests), `pill_visibility_controller.rs` (hide countdowns on sleeping threads with a generation counter; emits `pill-visibility-changed`), `pill_visibility_command.rs` (`get_current_pill_visibility`). The arbiter's listener in `lib.rs` now also feeds the visibility controller.
  - Rust window: `hide_pill_window` in the platform trait + Windows `ShowWindow(SW_HIDE)`; command `conceal_pill_window`.
  - `system_tray.rs`: tray icon (app icon, tooltip "Crest") with "Quit Crest"; Cargo feature `tray-icon`.
  - Constants: 30 s hide after pause, 3 s after the activity ends.
  - Frontend: `pill/pillVisibilityController.ts` (show: window first, then grow in; hide: fade/shrink first, then hide the window; never hides under the pointer), `pill/waitUntilNextFrameIsPainted.ts` (moved out of `main.ts`), `ipc/listenForRustStateChanges.ts` (generic listen-then-ask; replaces `listenForPillPresentation.ts`), `ipc/requestPillWindowConceal.ts`; `pillStateMachine.ts` gets `isPointerOverPill` and `handlePillConcealed()`; hide animation in `pillShell.css`; `main.ts` no longer reveals the window itself.
  - App icon: own design `src-tauri/icons/crest-icon-source.svg` (indigo rounded square, black capsule, warm "album" square, three bars); all sizes generated with `npx tauri icon`; the generated Android/iOS folders were deleted (desktop only).
  - Verified by testing: tsc, vite build, 23/23 Rust tests, clippy clean. User confirmed: tray icon + Quit, hides 30 s after pause, comes back on play, hides when YouTube Music closes. Edge cases via injected fake events: very long title/artist → "…" in both views; missing art → music-note placeholder; no duration → progress row hidden; unknown activity kind → "Nothing to show"; hide → native window really hidden and interactive area back to compact. CPU hidden: 0%. Memory of all Crest processes: about 388 MB (mostly WebView2).
- **Learned:**
  - **Pure decision + thin executor:** `decide_pill_visibility(previous, current)` has no timers or threads, so every rule has a unit test; the controller only runs the countdowns. Same split as the frontend state machine and morph controller.
  - **Generation counter for cancellable timers:** every schedule change bumps a number; a sleeping timer only acts if the number is still the one it started with. Nothing needs to be "cancelled" explicitly.
  - **Rust decides, the frontend animates:** Rust says "hide"; the page plays the hide animation and only then asks Rust to hide the native window (and the reverse for showing).
  - **Tray icons in Tauri 2:** `TrayIconBuilder` + a `Menu` of `MenuItem`s with IDs; `on_menu_event` matches the ID; `app.exit(0)` quits. Needs the `tray-icon` Cargo feature.
  - **App icons:** one square source (SVG/PNG) → `tauri icon` generates `.ico` (Windows), `.icns` (macOS) and PNG sizes; the icon is embedded at compile time, so it needs a rebuild.
  - **Hidden ≠ paused:** WebView2 keeps `document.visibilityState === "visible"` when the native window is hidden, so page timers keep running. Our only endless timer (the bars) runs only while music plays, which is exactly when the pill is visible.
  - **Testing with injected events:** the frontend can emit Tauri events to itself (`__TAURI_INTERNALS__.invoke('plugin:event|emit', …)`), which makes rare cases (huge titles, no art, unknown kinds) testable on demand.
- **Decisions:**
  - Visibility rules live in the Rust core (as planned), the animation in the frontend.
  - A pending hide waits while the pointer is over the pill.
  - A paused pill that's hidden reappears on a new track or on play; there's no "hover the top edge to reveal" strip (not in the v1 scope).
  - Tray menu has only "Quit Crest" (spec). Rejected for now: "Show pill", settings.
- **Problems:**
  - A getter named like its private field (`isPointerOverPill`) doesn't compile in TypeScript → field renamed to `isPointerCurrentlyOverPill`.
  - A Crest process from the user's `tauri dev` stayed alive after its Vite server was gone; closed with the user's permission.
  - **Unexplained, not reproduced:** in the first edge-case run, 1.5 s after a hide the shell still had `is-expanded` and the full interactive area (it went compact 4 s later). Two controlled replays of the same sequence (logged with a MutationObserver) behaved correctly. Most likely cause: my injected fake events mixed with real events from the live session, since Rust doesn't know about the fakes. Watch for a pill that stays expanded after reappearing.
- **Open questions / ideas for later:**
  - Hide (or stop the bars) while a fullscreen app or game is in front.
  - WebView2 memory (about 330 MB of the 388 MB): try browser arguments, or suspending the webview while hidden.
  - Build an installer (`npm run tauri build`, NSIS) and start with Windows (autostart); not in the v1 scope.
  - Use SMTC's "control enabled" flags to grey out buttons the player doesn't support.
  - Linux (KDE Plasma, Wayland): `media/linux_mpris/` (MPRIS over D-Bus) and `pill_window/linux_layer_shell/` (layer-shell for position, always-on-top and input region).
- **Next:** v1 is done. Pick from the ideas above, or start the Linux port.

## 2026-09-30 — Milestone (f): control buttons + CPU fix for animations
- **Done:**
  - Rust media: `media_source.rs` adds `MediaTransportCommand` and `send_media_transport_command` (and `MediaSource: Send`); `windows_smtc/smtc_worker_message.rs` (moved the message enum out of `smtc_event_subscriptions.rs`, adds `TransportCommandRequested`); `windows_smtc/smtc_transport_commands.rs` (`TryTogglePlayPauseAsync` / `TrySkipNextAsync` / `TrySkipPreviousAsync`); `smtc_media_source.rs` now creates the channel and keeps a sender; `smtc_worker_thread.rs` handles commands; `smtc_session_tracker.rs` remembers the shown session and sends commands to it.
  - Rust core: `activity_source.rs` adds `perform_activity_action` (and `ActivitySource: Send`); new `activity_source_registry.rs` (kind → running source; starts sources; 2 tests); new `activity_action_command.rs` (`perform_activity_action` command). `music_activity_source.rs` maps action names to commands (1 test). `ipc_channel_names.rs` adds the three action names. `lib.rs` uses the registry.
  - Frontend: `ipc/requestActivityAction.ts`; `ipcChannelNames.ts` adds the command + action names; `musicControlButtons.ts` sends actions on click.
  - CPU fix: `playbackBarsIndicator.ts` drives the bars with a 15 fps timer (cosine bounce per bar) instead of a CSS animation; `playbackProgressBar.ts` redraws 4×/s with a timer instead of `requestAnimationFrame`; constants in `frontendConstants.ts`; the bar keyframes and duration tokens are gone from the CSS.
  - Verified by testing: 18/18 Rust tests, tsc, clippy clean. The user confirmed play/pause, next and previous all control YouTube Music and the pill follows. CPU of Crest + WebView2 while playing, compact: **34.7% of one core before the fix → 4.7% after** (GPU process 2.8%, renderer 1.9%, Rust 0%); paused: about 0%.
- **Learned:**
  - **Routing by name:** the frontend sends `(activityKind, activityAction)`; the core looks up the source by kind and hands it the action string. The core never knows what "next-track" means, so new sources bring their own actions without core changes.
  - **Never block Tauri's thread:** the command only drops a message into the SMTC worker's channel; the worker does the blocking `.join()` on the WinRT call.
  - **One-way state flow:** a button doesn't change the pill directly. The player changes, SMTC reports it, and the pill updates. So the pill always shows the truth (if the player refuses, nothing lies).
  - **Refresh rate matters:** endless CSS animations and `requestAnimationFrame` run at the monitor's refresh rate (**239 Hz** here). In a transparent WebView2 window, each frame is composited by the GPU process. A slow fixed-rate timer means the page only produces a frame when something actually changes.
  - **Measure per process:** WebView2 splits into browser, renderer (JS, style, layout), GPU (drawing/compositing) and utility processes; the command line's `--type=` tells which is which.
  - **`tauri dev` watches files:** Rust edits rebuild and restart the app; frontend edits hot-reload. That's why the user's running copy already had the new buttons.
- **Decisions:**
  - Buttons don't update the UI optimistically; they wait for the player's confirmation (about 100 ms, feels immediate). Rejected: optimistic toggle, which could show the wrong state if the player declines.
  - Bars at 15 fps, progress at 4 redraws/s. Trade-off: slightly less fluid bars for 7× less CPU. 10 fps would be about 3%, if the user wants less.
  - The spring morph stays a CSS transition (600 ms, then idle).
  - Control availability flags from SMTC (e.g. "next" disabled) aren't used yet; YouTube Music enables all three.
- **Problems:**
  - My end-to-end test run failed with "Port 1420 is already in use". Cause: the user was running `tauri dev`, which had already rebuilt with the new code. Fix: tested in the user's running copy instead (the user pressed the buttons).
  - 35% CPU while playing (see Learned). Fixed as above.
- **Open questions:** should the bars stop when a fullscreen app or game is in front (the pill isn't visible there anyway)? Candidate for (g) or later.
- **Next:** milestone (g): visibility policy (hide after 30 s paused / 3 s without session, reappear on track change), tray icon with Quit, own app icon, edge-case pass.

## 2026-09-30 — Milestone (e): compact and expanded states with animation
- **Done:**
  - Rust: `pill_window_platform.rs` gets `set_pill_window_interactive_area`; Windows implementation with `CreateRectRgn` + `SetWindowRgn` (`Win32_Graphics_Gdi` feature); `pill_interactive_area.rs` (CSS px → physical px, rounded outward, 1 test); command `set_pill_interactive_area`.
  - Frontend pill: `pillStateMachine.ts` (hover 150 ms → expand, leave 350 ms → collapse, click → expand, attention → 4 s peek), `pillPointerInput.ts`, `pillMorphController.ts` (order of class change vs interactive area), `pillContentPresenter.ts` (mounts one view set per activity kind), `pillShellElements.ts` (renamed from `pillShellElement.ts`: shell + compact/expanded layers), `pillDimensionCssVariables.ts`.
  - Frontend activities: `activityViewSet.ts` (interface), `activityViewRegistry.ts` (kind → view-set factory), `nothingToShowViewSet.ts`, `createSvgIconElement.ts`; music: `musicViewSet.ts`, `compactMusicView.ts`, `expandedMusicView.ts`, `albumArtImage.ts`, `playbackBarsIndicator.ts`, `playbackProgressBar.ts`, `musicControlButtons.ts`.
  - Styles: `designTokens.css` (all visual numbers incl. the spring curve), `pillShell.css`, `albumArtImage.css`, `compactMusicView.css`, `expandedMusicView.css`, `playbackBarsIndicator.css`, `playbackProgressBar.css`, `musicControlButtons.css`. Removed `musicViews.css`.
  - `ipc/requestPillInteractiveArea.ts`; `frontendConstants.ts` has every size and delay; `main.ts` wires it all up.
  - Verified by testing: tsc, `vite build` (11 KB JS / 7 KB CSS), 15/15 Rust tests, clippy clean. Running app: window 396×184 centered, region 220×36 while compact; a simulated `mouseenter` (sent inside the page over the DevTools protocol) expanded the pill to 380×176 and the region to the full window; `mouseleave` kept the region until the animation ended, then went back to 220×36. Screenshots of compact and expanded match the design. CPU of Crest + 6 WebView2 processes, compact and paused: 0 ms in 10 s.
- **Learned:**
  - **Window regions (`SetWindowRgn`):** a window can be limited to a shape; outside it, the mouse reaches the windows below and nothing is drawn. The OS does the hit-testing, so no polling. The region's edges are hard pixels, so we use a rectangle and let CSS draw the rounded corners.
  - **Why not resize the window during animation:** Windows moves and resizes the window immediately, but the web content re-lays itself out a frame or two later, so the pill would jump sideways. A fixed window with a changing region avoids that.
  - **State machine:** all rules ("hover → wait 150 ms → expand") live in one small class with no DOM; the controller only carries out states.
  - **CSS transitions + `linear()`:** a transition animates between two values; `linear(0, 0.036, …, 1)` is a custom easing made of sampled points, here a damped spring (slight overshoot, then settle). A `transitionend` event tells us when it finished; a timer backs it up in case it never fires.
  - **GPU-friendly animation:** animating `transform` (bars, progress fill) doesn't trigger layout; width/height (the capsule) do, which is fine for a short animation of a tiny page.
  - **`requestAnimationFrame` loop only while visible:** the progress bar redraws every frame only while expanded and playing; otherwise zero work.
  - **Build once, update in place:** views keep their elements and only change text/classes, so running animations don't restart on every update.
  - **Chrome DevTools Protocol:** WebView2 started with `--remote-debugging-port` can be scripted (evaluate JS, read DOM). Useful for testing UI without moving the real mouse.
- **Decisions:**
  - **Changed (was: resize the native window between compact and expanded):** the window is always expanded-size plus 8 px spring-overshoot room; only the interactive region changes. Reason: avoids the relayout jump described above. Trade-off: while expanded, the 8 px margin around the pill also catches clicks.
  - Compact shows album art + **title** + bars (the spec mentions art and bars; the title uses the space a notch would take on a phone).
  - Hover expands after 150 ms (prevents opening when the mouse just passes by); leaving collapses after 350 ms.
  - Peek also happens at startup when a track is already playing (acts as a "hello").
  - Icons are my own simple shapes on a 24×24 grid (no copied icon set).
- **Problems:** none during implementation; everything compiled and worked on the first run. The user rejected a multi-question checklist, so verification used scripted DOM events + screenshots + Win32 inspection instead.
- **Open questions:** CPU while playing (the bars animate continuously): measure in (f). Feel of the spring (duration 600 ms, 2.8% overshoot) is a taste call for the user. The peek on a real track change isn't verified yet.
  - **Correction (milestone f):** CPU while playing was 34.7% of one core with the CSS-animated bars (239 Hz monitor). Fixed in (f) with timer-driven bars at 15 fps → 4.7%. The user confirmed (e) works ("Ok it works").
- **Next:** milestone (f): buttons send play/pause/next/previous to the player through Rust.

## 2026-09-30 — Milestone (d): live data reaches the frontend
- **Done:**
  - Core (`activity_core/`): `activity_update.rs` (what a source reports: priority, is-ongoing, attention key, JSON payload), `activity_source.rs` (plugin trait), `activity_publisher.rs` (a source's handle to the core), `activity_arbiter.rs` (keeps the latest update per source, picks the winner, notifies only on change; 3 tests), `pill_presentation_command.rs` (`get_current_pill_presentation`).
  - Music plugin (`activity_sources/music/`): `music_activity_source.rs` (snapshot → activity update; attention key = title + artist), `session_loss_grace_period.rs` (ignores a session gap shorter than 1.5 s; 2 tests).
  - `ipc_channel_names.rs`: event name + activity kind shared with TS. `media_session_snapshot.rs` now serializes to camelCase JSON. `lib.rs` creates the arbiter (emits `pill-presentation-changed` to the pill window), shares it with `manage()`, and starts every activity source in a loop. Removed `media_console_preview.rs`.
  - Frontend: `activities/pillPresentationTypes.ts`, `activities/activityViewRegistry.ts` (kind → view; "Nothing to show" fallback), `activities/music/nowPlayingTypes.ts`, `activities/music/compactMusicView.ts` (text with ellipsis), `ipc/listenForPillPresentation.ts`, `styles/musicViews.css`; `main.ts` renders presentations instead of fake text.
  - Verified by testing: tsc clean, 14/14 Rust tests, clippy clean. User confirmed: the pill shows the playing song; pause adds "(paused)"; play removes it; next track updates without flashing "Nothing to show".
- **Learned:**
  - **Tauri events** (Rust → frontend, many times: `emit_to(window, name, payload)` / `listen(name, callback)`) vs **commands** (frontend → Rust, request/response: `invoke`). Payloads cross as JSON; serde's `rename_all = "camelCase"` turns `track_title` into `trackTitle`.
  - **Events sent before anyone listens are lost.** The page starts after Rust, so it listens first, then asks for the current state with a command, and ignores that answer if an event already arrived (it could be older).
  - **Shared state across threads:** `Arc<Mutex<T>>`. `Arc` = several owners (the media thread and a Tauri command both use the arbiter); `Mutex` = one at a time. `app.manage(value)` + a `State<'_, T>` command parameter hands shared state to commands.
  - **Plugin design via a trait:** `lib.rs` holds `Vec<Box<dyn ActivitySource>>` and starts each with its own publisher. The core never knows it's music; only the frontend view does.
  - **`#[serde(flatten)]`** merges a nested struct's fields into the parent's JSON (PillPresentation = kind + ActivityUpdate fields).
  - **`textContent` vs `innerHTML`:** song titles come from the web; `textContent` can never inject markup.
  - **Vite dependency optimizing:** the first time the code imports a new package (`@tauri-apps/api/event`), Vite pre-bundles it and reloads the page once (dev only).
- **Decisions:**
  - The frontend picks its view by `activityKind`; the payload stays `unknown` in the core types and is cast by the activity's own view.
  - Arbiter tie-break: equal priority → most recent update wins.
  - Grace period with a generation counter under a mutex, so a returning session and a pending withdrawal can't race. Rejected: acting in the SMTC worker with timeouts (would put music-specific logic in the Windows code).
  - Idle text is generic ("Nothing to show"), since the core doesn't know which kind is missing. It'll mostly be hidden once (g) hides the pill.
- **Problems:**
  - Unused re-export warning for `PillPresentation` → removed.
  - First run logged "IPC custom protocol failed … Failed to fetch" and repeated "Couldn't find callback id …". Cause: Vite optimized `@tauri-apps/api/event` and reloaded the page; the first page's in-flight call was cut off, and its event listener stayed registered in Rust. Fix: none needed. Verified by testing that a second start has no reload and no warnings; production builds have no Vite server.
- **Open questions:** none new.
- **Next:** milestone (e): compact view with tiny album art + animated bars, expanded view on hover/click, 4 s peek on track change, spring animations, window resize between the two sizes.

## 2026-09-30 — Milestone (c): Rust reads the media session
- **Done:**
  - Platform-neutral: `media/media_session_snapshot.rs` (what the pill needs to know), `media/media_source.rs` (`MediaSource` trait + listener type), `media/media_session_selector.rs` (app-ID filter + choose playing / most recently active, 4 tests), `user_settings_file.rs` (optional `settings.json`).
  - Windows: `media/windows_smtc/` with `smtc_media_source.rs` (starts the worker thread), `smtc_worker_thread.rs` (MTA init + message loop with 50 ms coalescing), `smtc_event_subscriptions.rs` (event handlers → channel messages; unsubscribe on drop), `smtc_session_tracker.rs` (sessions, last activity, dedup), `smtc_snapshot_reader.rs` (WinRT → snapshot, 2 tests), `smtc_thumbnail_reader.rs` (album art → base64 data URL, cached per track, 1 test).
  - Temporary `media_console_preview.rs` prints each snapshot (removed in d). `lib.rs` loads settings and starts the media source. `Cargo.toml`: serde, serde_json, base64, more `windows` features.
  - Verified by testing: 9/9 tests, clippy and tsc clean. Live run with YouTube Music (Brave PWA): title/artist/album/app ID correct; album art PNG, 26 KB; pause, play and seeks all arrive, about 60 ms after the browser's timestamp; no errors. Rust process: **0 ms CPU in 10 s** idle, 35 MB, 11 threads.
- **Learned:**
  - **Threads and channels:** the SMTC work runs on its own thread; event handlers (called by Windows on its thread pool) only `send` a message into an `mpsc` channel (multi-producer, single-consumer). The worker blocks in `recv()` → zero CPU until something happens.
  - **Coalescing:** after the first message, the worker collects everything else arriving within 50 ms and handles it once, so a seek burst becomes one update.
  - **`Drop`:** Rust runs `drop()` automatically when a value goes away. `SmtcSessionEventSubscription` unsubscribes there, so replacing the session list cleans up closed sessions by itself (the RAII pattern).
  - **COM apartments:** confirmed in Rust. With `RoInitialize(RO_INIT_MULTITHREADED)` the values update live without a message loop.
  - **`.join()`** blocks the current thread until an `IAsyncOperation` finishes. That's fine on our worker thread, never on the UI thread.
  - **Data URLs:** `data:image/png;base64,....` embeds the image bytes in text, so the webview can show album art without file or network access (our CSP allows `img-src data:`).
  - **Magic numbers (file signatures):** a PNG starts with `\x89PNG`, a JPEG with `FF D8 FF`. We use them when a player doesn't say what format the art is.
  - **Idle cost of a web UI:** the Rust side is tiny; WebView2 (Chromium) is 6 processes, about 325 MB.
- **Decisions:**
  - On a session-list change, unsubscribe from all sessions and subscribe again (simple and always correct; list changes are rare). Rejected: diffing sessions, since WinRT session objects have no reliable identity to compare.
  - Snapshots are compared to the previous one and only sent when something changed.
  - Album art is cached per (title, artist, album) and re-read while missing, because browsers often publish the art after the title.
  - The OS implementation is picked with a `cfg`-gated alias `CurrentPlatformMediaSource` in `media/mod.rs`, like the window. So `platform_media_source_factory.rs` from the approved tree isn't needed.
  - Windows-only constants in `backend_constants.rs` are `#[cfg(target_os = "windows")]` so a Linux build has no unused-constant warnings.
- **Problems:**
  - The first snapshot said the position was "reported 4625018 ms ago". Cause: the track had been **paused** since an hour earlier (our PowerShell probe); a paused position doesn't move, so the old timestamp is correct. Rule for (e): extrapolate the position only while playing, and clamp it to the duration.
  - My own verification script picked "the newest log file", which was its own output, and fed itself into a 512 KB loop. Stopped and deleted; not project code.
  - At first I found no YouTube Music window by title (only "New Tab - Brave"): a PWA window can live in a Brave process whose main window is another one. SMTC doesn't care.
- **Open questions:** can WebView2's memory be reduced (for example with browser arguments or by suspending the webview when hidden)? Look at in (g). Behavior with several allowed sessions at once is only unit-tested.
- **Next:** milestone (d): core + music activity source, emit updates to the frontend as Tauri events, show plain text in the pill.

## 2026-09-30 — Milestone (b): pill window behavior with fake content
- **Done:**
  - Rust: `backend_constants.rs`; `pill_window/` with `pill_window_platform.rs` (trait), `pill_window_placement.rs` (top-center math + 2 unit tests), `pill_window_commands.rs` (`place_pill_window_at_top_center`, `reveal_pill_window`), `windows_native/windows_pill_window_platform.rs`; `lib.rs` prepares the window in `setup` and registers the commands.
  - Config: `tauri.conf.json` window is hidden at start, frameless, transparent, no shadow, always on top, skipTaskbar, not focusable, not resizable. `Cargo.toml` adds `windows` 0.62 for Windows only.
  - Frontend: `frontendConstants.ts` (pill size), `ipc/ipcChannelNames.ts`, `ipc/requestPillWindowPlacement.ts`, `ipc/requestPillWindowReveal.ts`, `pill/pillShellElement.ts`, `styles/designTokens.css`, `styles/pillShell.css`, `vite-env.d.ts`; `main.ts` builds the pill, asks Rust to size and place it, waits for a painted frame, then asks Rust to show it.
  - Repo: `.gitattributes` (LF everywhere).
  - Verified by testing: `tsc` clean, `cargo test` 2/2, `cargo clippy` no warnings. Win32 inspection of the running window: visible, `WS_EX_TOPMOST` + `WS_EX_TOOLWINDOW` + `WS_EX_NOACTIVATE`, no `WS_EX_APPWINDOW`, 220×36 at left 1170 on a 2560-wide screen (exactly centered), top 8. The foreground window stayed the user's game.
- **Learned:**
  - **Logical vs physical pixels:** CSS and Tauri configs use logical pixels; Windows places windows in physical pixels = logical × scale factor (1.5 at 150% scaling). We convert using the monitor's scale factor.
  - **Extended window styles** are bit flags on every Win32 window: `WS_EX_TOPMOST` (above normal windows), `WS_EX_TOOLWINDOW` (no taskbar button, not in Alt+Tab), `WS_EX_NOACTIVATE` (clicking never takes keyboard focus), `WS_EX_APPWINDOW` (forces a taskbar button). Read and written with `GetWindowLongPtrW` / `SetWindowLongPtrW`; checking one flag = `style & FLAG != 0`.
  - **Activation vs showing:** `ShowWindow(SW_SHOW)` shows *and activates* (takes focus); `SW_SHOWNOACTIVATE` only shows.
  - **Why the frontend waits for a painted frame before reveal:** a `requestAnimationFrame` callback runs just before a paint, so after two of them at least one frame with our content has been painted → no empty or white window.
  - **Tauri commands:** a `#[tauri::command]` Rust function becomes callable from TS with `invoke("function_name", { camelCaseArgs })`; Tauri converts camelCase keys to snake_case parameters. Parameters like `WebviewWindow` are filled in by Tauri (the calling window). Returning `Result<_, String>` turns an `Err` into a rejected promise in TS.
  - **`#[cfg(target_os = "windows")]`** compiles code only on that OS; `[target.'cfg(windows)'.dependencies]` does the same for dependencies. That's how the Linux version will slot in.
  - **`unsafe`** in Rust marks code the compiler can't check (here: raw Win32 calls). Keep it tiny and write a `SAFETY:` comment saying why it's fine.
- **Decisions:**
  - Show/hide the pill only through our platform trait (Win32 `ShowWindow`), never Tauri's `show()`/`hide()` or flag-changing setters. Why: see Problems. Rejected: a window subclass that re-adds `WS_EX_TOOLWINDOW` on every style change (more robust but more complex; revisit if styles get lost in testing).
  - The OS implementation is picked with a `cfg`-gated `pub use ... as CurrentPlatformPillWindow` in `pill_window/mod.rs`. No factory file or trait object is needed, since the choice is made at compile time.
  - Sizes live only in `frontendConstants.ts`; no width/height in `tauri.conf.json`.
  - Placement is split into a pure function (`calculate_top_center_bounds`, unit-tested) and the part that talks to the window.
  - Small deviations from the approved tree: `ipcEventNames.ts` → `ipcChannelNames.ts` (it holds command names; events come in (d)); `windows_pill_window_styles.rs` → `windows_pill_window_platform.rs` (it also shows the window).
- **Problems:**
  - `tsc`: "Cannot find module or type declarations for side-effect import of './styles/designTokens.css'". Cause: TypeScript 6 checks side-effect imports and didn't know `.css`. Fix: `src/vite-env.d.ts` with `/// <reference types="vite/client" />`.
  - `cargo`: "cannot find `__cmd__reveal_pill_window` in `pill_window`". Cause: `#[tauri::command]` generates hidden helpers next to the function, and `generate_handler!` looks for them at the path you write; a `pub use` re-export doesn't carry them. Fix: `pub mod pill_window_commands` and full paths in `generate_handler!`.
  - Port 1420 in use when the user ran `tauri dev`. Cause: the preview server started for them was still running. Fix: stop it; always stop our servers before handing over.
- **Open questions:** hover events in a `WS_EX_NOACTIVATE` window are untested until (e).
- **Verified by testing (user, visually):** clean black rounded capsule with no border or corner artifacts; no taskbar icon; not in Alt+Tab; clicking it keeps typing focus in Notepad; stays above other windows.
- **Next:** milestone (c), the `MediaSource` trait + SMTC implementation printing snapshots to the console.

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
- **Correction (milestone b, verified by reading tao 0.37.1 source, `platform_impl/windows/window_state.rs` and `window.rs`):**
  - `focusable: false` sets `WS_EX_NOACTIVATE`, and tao keeps it.
  - `focus: false` only affects creation: tao clears its "don't focus" marker right after creating the window, so every later `show()` uses `SW_SHOW` and activates the window.
  - On any flag change (show, hide, always-on-top, ignore-cursor, ...) tao rewrites the whole extended style from its own flags, which never include `WS_EX_TOOLWINDOW`.
  - Consequence: add `WS_EX_TOOLWINDOW` ourselves while hidden, and show with our own `ShowWindow(SW_SHOWNOACTIVATE)`.

## 2026-09-30 — Research: what YouTube Music exposes through SMTC (verified by testing)
Setup: YouTube Music installed as a PWA in **Brave**. Probe: PowerShell 5.1 calling the same WinRT API, script in the session scratchpad.
- **App ID:** `Brave._crx_cinhimbnkkghhklpknlkffjgod`. `cinhimbnkkghhklpknlkffjgod` is the YT Music web-app ID; the prefix is the browser. Whether the suffix is identical in Chrome/Edge is **unverified**.
- **Present:** Title; Artist (joined, e.g. "Azahriah and DESH"); AlbumTitle; PlaybackType = Music; PlaybackStatus (Playing/Paused); Timeline Start/End/Position/LastUpdated/MinSeek/MaxSeek (End = duration); Controls flags (pause, toggle, next, previous, seek enabled; play disabled while playing).
- **Missing/empty:** AlbumArtist, Subtitle, TrackNumber (0), AlbumTrackCount (0), Genres, Shuffle, Repeat.
- **Thumbnail:** reference present. Bytes, format and pixel size **unverified**: PowerShell 5.1 can't pass WinRT stream interfaces to .NET. Verify in Rust in milestone (c).
  - **Correction (milestone c, verified by testing in Rust):** the thumbnail is a PNG, about 26 KB, read with `OpenReadAsync` + `DataReader`. Pixel size not measured yet.
- **Update behavior:**
  - Play/pause shows up within about 100 ms (limited by the probe's 100 ms sampling).
  - Timeline Position is **not** pushed continuously during steady playback, only on play/pause/seek/track change → the UI must extrapolate: `position + (now − LastUpdatedTime) × rate` while playing.
  - Dragging the seek bar causes bursts of about 10 updates/s → coalesce updates before emitting.
  - On track change the session **disappears for about 0.4 s** and comes back → don't hide on "no session" immediately; use a grace period.
- **Learned (strongly indicated, confirm in Rust):** in the default single-threaded COM apartment (STA) with no message pump, session values went stale (1 change in 60 s); in MTA they updated live. → SMTC must run on an MTA thread.
  - **Correction (milestone c):** confirmed working in Rust on an MTA thread with events (verified by testing). The STA failure itself wasn't retested in Rust.
  - **Also learned (milestone c):** when a session is paused, `LastUpdatedTime` can be very old (an hour, in our test). That's correct for a paused position, but the UI must not extrapolate from it.
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

# Release build: exe + installer (first build about 5-10 min). Raise "version" in package.json,
# src-tauri/tauri.conf.json and src-tauri/Cargo.toml first; quit the installed Crest (tray) before installing.
npm run tauri build
#   standalone exe: src-tauri\target\release\crest.exe
#   installer:      src-tauri\target\release\bundle\nsis\Crest_<version>_x64-setup.exe

# Publish commits to GitHub (https://github.com/Denis112500/Crest)
git push

# Checks
npx tsc --noEmit                       # type-check the frontend
cd src-tauri; cargo build; cd ..       # compile the Rust side only
git status                             # what changed since the last commit

# Debugging helpers (window styles, page state, recorder, media sessions, album art):
# see dev-tools/README.md
pwsh dev-tools/inspect_pill_window.ps1

# Debug the pill's page from outside (DevTools protocol on a local-only port), then open
# http://127.0.0.1:9223/json to find it. Only for testing; don't leave it on.
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9223"; npm run tauri dev

# "Port 1420 is already in use": find out which process holds it
Get-NetTCPConnection -LocalPort 1420 -State Listen | ForEach-Object { Get-Process -Id $_.OwningProcess }
```
