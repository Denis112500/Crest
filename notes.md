# Project notes

## Current state
- **Works:** v0.3.1 published and installed (Phases 1 and 2 complete): notch glued to the top edge, hides instantly for fullscreen apps, greys out refused buttons; settings window (tray "Settings…" or launching Crest again) with Start with Windows (restored after updates), monitor choice, allowed players, About card. RAM while hidden 16–32 MB; CPU 0% idle/hidden (bars timer leak fixed), ~4.7% of one core playing; starts at login (verified).
- **In progress:** nothing. Phase 2 is done; v0.3.1 is published (https://github.com/Denis112500/Crest/releases/tag/v0.3.1): new installs show every player. Next: soft launch (Tauri Discord, r/tauri), then Phase 3 (integrations, free only) when the user starts it; big launch after the Claude Code integration. Rule: installs and autostart checks only outside the Claude app (MSIX container).
- **Broken:** nothing known; button presses during a track change are now held and delivered (user confirmed). One unexplained observation in the (g) edge-case test didn't reproduce (see that entry).

---

## 2026-10-06 - Comments next to every component
- **Request (user):** a comment next to each component, to understand the code. Option (b) chosen on my recommendation: a short header at the top of each file (what it's for, where it fits, why it's its own piece), the README file tree stays the overview. Fits CLAUDE.md's "comments explain WHY, not what" because the headers explain role and reason.
- **Done (branch `file-headers`):** 62 new headers (Rust `//!` module docs, TypeScript `//` at the top). 11 files already started with a role comment (e.g. `ipcChannelNames.ts`, `frontendConstants.ts`, `classic_frame_painting_blocker.rs`); 32 more had a role comment on their main type that the new header repeated almost word for word (found by word overlap ≥ 0.5), so their header was removed again: every file is explained once, either by its header or by the comment on its main type. Only comment lines added, no code changed; tsc, vite build, clippy, 55/55 tests, `cargo doc` clean.
- **Learned:** Rust's `//!` comments are module documentation: `cargo doc --no-deps --open` (in `src-tauri`) turns them into a clickable website of all modules (`target/doc/crest_lib/`).
- **Noticed:** all headings in this file changed from "—" to "-" outside my edits (editor or formatter, presumably); kept as they are.

## 2026-10-05 - 0.3.1: every player by default for new installs
- **Why (decided with the user):** before a soft launch (Tauri Discord, r/tauri; the big launch waits for Phase 3's Claude Code integration), a stranger with Spotify or a normal browser tab would install Crest, play music and see nothing, because only YouTube Music was allowed by default.
- **Done (branch `every-player-default`):** `CrestUserSettings::default()` (used only when there is no `settings.json`, i.e. a fresh install) now has `show_every_media_player: Some(true)`; the list still starts with YouTube Music, so switching "Show every player" off gives the old behavior. Existing files keep their choice (an older file with a list and no switch still means "only the list", test). 2 new tests; 55/55, clippy clean, tsc. README: "any player that appears in Windows' own media controls (built and tested with YouTube Music and Brave)", status 0.3.1, the Allowed players paragraph. Version 0.3.0 → 0.3.1 (6 places); installer `Crest_0.3.1_x64-setup.exe` built 18:27.
- **Also pushed before:** `118b607` Ignore local demo helper (`.gitignore`: `dev-tools/readme_demo_pointer.py`).
- **Not claimed:** that it works with Spotify or other players (only YouTube Music and Brave tabs were tested).
- **Verified by testing (user, real install, own Windows Terminal):** 0.3.0 → 0.3.1 with the installer's preselected "Uninstall before installing" (the first real upgrade outside the Claude container), then Crest started from the Start menu: About = 0.3.1, built 18:27; `Run\Crest` = `C:\Users\<user>\AppData\Local\Crest\crest.exe`. The check right after the installer (entry deleted) wasn't reported; the end result is the one that matters.
- **Published (2026-10-05):** commits `5c2ff15` Show every player on new installs, `5e77a85` Update README, `2317eb9` Update notes on `main`; GitHub Release v0.3.1 "Crest 0.3.1", tag on `2317eb9`, asset `Crest_0.3.1_x64-setup.exe` (1 457 435 bytes, the tested 18:27 build), marked Latest, text approved by the user. `dev-tools/record_readme_demo.py` stays uncommitted on purpose (its local version needs the unpublished mouse helper).
- **Next:** soft launch (Tauri Discord "showcase", r/tauri), then Phase 3; the big launch after the Claude Code integration.
- **Soft launch (2026-10-05):** posted in the Tauri Discord showcase (tag "app", GIF attached; the forum title is limited to 100 characters). Feedback from a commenter: "(no scraping, no network)" read as AI-generated; better to name the API: Crest uses `GlobalSystemMediaTransportControlsSessionManager` (via the `windows` crate, event-driven: `SessionsChanged`, `MediaPropertiesChanged`, `PlaybackInfoChanged`, `TimelinePropertiesChanged`). Post text rewritten in plain words ("around 20 MB of RAM" instead of "~16–32 MB … thanks to…").
- The commenter's verdict after the edit: "Nice thing though." Then posted on **r/tauri** (Images & Video post with the demo GIF; title "Made a little "now playing" notch for Windows 11 with Tauri"; short casual text: "like the iPhone's Dynamic Island", "Windows' media session API (SMTC)", "around 20 MB of RAM when hidden"); 33 views in the first minutes. Naming: call it a notch and mention the Dynamic Island only as a comparison (Apple's name).
- **Learned:** for developer audiences, name the exact API and keep the tone plain; a marketing-style checklist makes people suspicious. The README's "Private by default… no scraping, no unofficial APIs, no cookies" has the same tone (rewrite offered, not done).

---

## 2026-10-05 - Going public: README demo, anonymized notes
- **Decision (user):** make Crest more popular but "faceless" (pseudonymous posts, a demo GIF instead of a person). `notes.md` stays public but is anonymized from now on (option b): the Windows user name in paths becomes `<user>`, specific installed apps become "other startup apps". Older versions stay in the git history (rewriting it was option c, not chosen). GitHub URLs (`Denis112500/Crest`) stay; the user name is a separate decision.
- **Done:** `dev-tools/record_readme_demo.py` (new): records a 12 s GIF and a still of the open pill, capturing only a rectangle around the pill window (412×184 + margins), merging unchanged frames; uses Pillow 12.1.1 (already installed). A test frame showed the user's animated wallpaper behind the transparent corners → record in front of an empty maximized Notepad.
- **Song for the demo:** an NCS (NoCopyrightSounds) release, free to use in public content with credit, played from an NCS playlist so the "next track" in the GIF is NCS too.
- **Takes (verified by looking at extracted frames):** take 1 (12 s, dark Notepad) showed compact → open but no track change and no closing. Take 2 (16 s) changed to a **commercial** song ("Closer") because the queue wasn't NCS → unusable; its still was an empty capsule mid-animation, because "darkest frame" fails on a dark wallpaper → the still is now "frame with the most bright pixels" (text, icons). Take 3 (16 s, NCS "C U Again", starry wallpaper): open (peek) → close → compact → open on hover; the 7 s compact middle was cut to ~3 s → `docs/crest-demo.gif` 10.9 s, 1.5 MB; `docs/crest-expanded.png` 25 KB.
- **Correction (user):** the take-3 GIF looked like the song never changes. True, but not because of the cut: its first frame already showed the new song at 0:02 (the "next" click came ~2 s before recording), and the cut 4.5–8.5 s was compact "C U Again" throughout (checked frames).
- **Take 4 (automated):** `record_readme_demo.py --skip-to-next-track-at 2.5` sends "next track" itself through the WinRT media controls (a Windows PowerShell 5.1 snippet, no new Python package), so the pill's own 4 s new-song peek does the opening and closing: compact "C U Again" → "On & On" opens with its cover → closes → compact. 10.5 s, 1.9 MB, nothing cut; both songs NCS; still = open "On & On". Credit line now names both songs.
- **Correction (user):** in take 4 nobody could see the "next" press (it was sent invisibly). **Take 5:** a local helper (`readme_demo_pointer.py`, kept on the user's PC and **not published**, user's decision; the recorder's published version doesn't use it) draws the pointer into every frame (`GetCursorPos`; screen captures don't include it) with a ring right after a click (`GetAsyncKeyState`), and `--demonstrate-next-button` moves and clicks the real mouse in eased glides: onto the pill (opens) → to the next button (measured at 1345, 137) → click → away (closes). The invisible-skip option was removed. Result: 9.5 s, 1.07 MB, hover → open → click with ring → loading ring → "On & On" → close; still without pointer.
- **README:** GIF at the top, "What it looks like" with the still, NCS credit line, status "0.3.0 is published", `docs/` in the file tree. `dev-tools/README.md`: the recorder's row.
- **Learned:** timing a live demo by hand rarely works on the first try; record generously, look at extracted frames, then cut. Album covers in public images belong to the labels, so a free-to-use (NCS) track keeps the README clean.

---

## 2026-10-05 - Release 0.3.0 published
- **Done:** commits on `main` (pushed): `70ef6f6` Restore Start with Windows after updates, `963547d` Fix bugs from code review, `7d508ba` Add reveal delay probe, `14b1b7a` Update dev-tools README, `b7353c1` Update README, `6376f44` Update notes. The two files touched by both code commits (`settingsWindowMain.ts`, `backend_constants.rs`) were split so each commit holds only what its name says.
- **Published (2026-10-05):** GitHub Release v0.3.0 "Crest 0.3.0" with `gh release create`, tag `v0.3.0` on `6376f44`, asset `Crest_0.3.0_x64-setup.exe` (1 457 395 bytes, built 10:54 from that exact code; the copy the user has installed), marked Latest; text approved by the user. It says updating from 0.2.0 with the preselected option removes Start with Windows once (from Tauri's installer script + the same-version "Uninstall Crest" test on a real install; a real 0.2.0 → 0.3.0 update outside the container wasn't run).
- **Phase 2 is complete:** items 6, 6b, 7, 8 + the autostart repair and the code review.
- **Learned:** `gh release view --json` has no `isLatest` field; `gh release list` shows "Latest".
- **Next:** Phase 3, when the user starts it.

---

## 2026-10-05 - Full code review before releasing 0.3.0
- **Request (user):** analyze the whole code with much attention before the last push of 0.3.0, fix every bug, take notes.
- **Reviewed:** all ~5,900 lines: Rust core (arbiter, visibility policy/controller, publisher, registry, commands), music source + grace period, the SMTC worker/tracker/reader/thumbnails/transport commands, album-art gate, held presses, pill window (placement, display choice, region, Win32 styles, frame blocker, memory target), fullscreen appbar watcher, settings store, launch-at-login repair, settings window (opener, commands, player options, build description), `lib.rs` setup order, capabilities; frontend pill (state machine, morph, visibility, presenter, IPC), music views (bars, progress, buttons, album art), settings page. Also searched for panics/unchecked indexing/casts outside tests (none that can fire) and checked lock ordering (always grace period → arbiter → visibility; no cycle).
- **Bugs found and fixed (all verified from code):**
  1. **Orphaned bars timer.** `pillContentPresenter.ts` replaced an activity's views (e.g. music → "nothing to show" when the player is closed or filtered out *while playing*) after stopping only the expanded view; the old compact view's bars kept a 15-per-second timer running forever on detached elements (and a new one each time). Fix: the outgoing views are also told "off screen" before they're dropped.
  2. **Settings window taller than small screens.** It always opened 720 logical px tall and isn't resizable; on a 1920×1080 laptop at 150 % the whole screen is 720 logical px, so the bottom cards (Integrations, About) would be out of reach. Fix: `settings_window_opener.rs` fits the height to the main display's work area (`Monitor::work_area()`, verified in Tauri 2.12's source) minus a title-bar allowance (`SETTINGS_WINDOW_TITLE_BAR_ALLOWANCE_LOGICAL_PIXELS` = 40); the page scrolls. 2 tests.
  3. **One failing card emptied the settings page.** `settingsWindowMain.ts` awaited all cards with `Promise.all` and only then inserted them, so one failing Rust call left every card empty. Fix: each card fills on its own and logs its own failure.
- **Checked, not bugs:** lock poisoning handled everywhere; the interactive-area region is freed if Windows refuses it; SMTC handlers removed on drop; an SVG thumbnail can't run scripts in `<img>`; the visibility race guards (generation counters) hold; the page's `style.transform` writes are allowed by the CSP (CSSOM, not inline style attributes).
- **Known limits (not fixed, noted):**
  - The fullscreen appbar is registered on the main monitor; whether Windows notifies it about fullscreen apps on monitor 2 while the pill sits there is unverified (the user reported the step-2 test list working, not item by item). Moving the pill to another monitor doesn't re-check fullscreen state until the next notification.
  - Plugging/unplugging a monitor while Crest runs doesn't re-place the pill until a restart.
  - If Task Manager has Crest disabled **and** an update deletes the entry, the repair re-registers it as enabled (the plugin always writes "enabled").
  - Two very quick "Settings…" clicks can try to create the window twice; the second attempt fails with a logged error and no visible effect.
- **Checks after the fixes:** tsc, vite build, clippy clean, 53/53 Rust tests. Installer rebuilt (10:54).
- **Bug 1 verified by testing (installed builds, user's real install, CPU time of each Crest process over 60 s with the pill hidden):** procedure: music playing with bouncing bars → Settings → Remove YouTube Music while it plays → close Settings → pill hides → measure. Old build (01:05): pill renderer **78 ms**; fixed build (10:54): **0 ms**. A first try that closed the YouTube Music window instead showed 0 ms in the old build too (10 s sample; Brave probably reports "paused" before the session goes, so the bars stopped normally; Windows counts CPU time in ~15.6 ms steps, too coarse for 10 s). Crest's Rust process uses ~0.1 s per minute meanwhile: the removed player keeps playing and its updates are read and dropped (expected).
- **Learned:** **replacing a component must also shut it down.** Timers live in the page, not in the elements; dropping elements from the DOM doesn't stop the intervals that update them.

## 2026-10-05 - Install tests were inside the Claude app's container (results invalid)
- **Found (verified):** after the "Uninstall Crest" + reinstall test, the running Crest's path was `C:\Users\<user>\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Local\Crest\crest.exe`. The Claude desktop app is an **MSIX package** (`Get-AppxPackage`: `Claude_pzs8sxrjxfjjc`, signature "Developer"). Every process it starts (my tools, also with the sandbox disabled, and the app's terminal panel) runs in its container: Windows redirects writes to `AppData` (and HKCU registry writes) into the package's `LocalCache`, and reads inside the container see that copy merged with the real one.
- **Consequences:** every installer I launched installed into `LocalCache\Local\Crest` (the user found no Crest in the real `AppData\Local`); the uninstall entry, `Run` entries and test results I read from inside the container can't be trusted. Probably also behind the earlier "tray checked but no `Run` entry" and the vanished/reappearing install folder (unverified).
- **Still valid:** the installer behavior read from Tauri's NSIS script (an upgrade with the preselected choice deletes `Run\Crest`), the code of the repair (50 tests), the About line.
- **New: About line** in the settings window (user's request: several builds share "0.3.0"): `Crest 0.3.0 · release/dev build · built <exe file date>` plus the running exe's path; tray tooltip "Crest 0.3.0" / "(dev)". `settings_window/crest_build_description.rs`, `src/settings/crestBuildDescriptionLine.ts`, command `read_crest_build_description` (settings window only); window height 640 → 720.
- **First real check (user's own Windows Terminal, real install in `AppData\Local\Crest`, About path confirmed):** `Run\Crest` existed but pointed to `…\Packages\Claude_…\LocalCache\Local\Crest\crest.exe`, the container copy deleted minutes before. So the 00:56 repair had worked and reached the real registry (my in-container read didn't show it), and the new install's switch showed "on" for an entry that would start nothing at login.
- **Gap fixed:** the repair only asked "is there an entry?". Now `windows_run_key_registration.rs` reads the entry's command and checks that its program still exists (`RunEntryState`: Missing / StartsMissingProgram / StartsExistingProgram). Wanted + missing **or pointing at a removed Crest** → register the running Crest again; no saved choice + broken entry → register again too (someone wanted it); an entry starting another *existing* Crest (e.g. a test copy) is left alone. 51/51 Rust tests, clippy clean; installer rebuilt (01:05).
- **Verified by testing (user, real install, own Windows Terminal):**
  1. Installer (01:05 build) from Explorer, "Add/Reinstall", About = built 01:05, path `AppData\Local\Crest\crest.exe`; switch untouched → `Run\Crest` = `C:\Users\<user>\AppData\Local\Crest\crest.exe` (the broken entry was replaced on the first start).
  2. Installer again, **"Uninstall Crest"** (= what an upgrade's preselected choice does), data kept, not started → `reg query` → "unable to find" (entry deleted, pasted).
  3. Crest started from the Start menu → entry back (reported by the user; that output wasn't pasted).
- **Still open:** Crest really starting at login (restart).
  - **Answer (2026-10-05, verified by testing):** after a restart Crest started by itself from `AppData\Local\Crest\crest.exe`. Timing (process start times): boot 10:10:35, desktop (explorer) 10:10:59, two other startup apps 10:11:42 and 10:11:48, **Crest 10:11:50**, about 50 s after the desktop, in Windows' batch of `Run` programs (the user has many startup apps). The user noticed the pill didn't come exactly when the music started; open: whether the music started before Crest was running, or the first show after boot was slow (low-memory page being paged back in, unverified).
  - **Measured (verified by testing, new `dev-tools/measure_pill_reveal_delay.ps1`, samples SMTC "playing" and the pill window's visibility every 20 ms):** the user pressed play after the tray icon was there; only that first appearance after boot was slow. With Crest running and the pill hidden ~3 min in low-memory mode: **77 ms** (±20 ms) from "playing" to "pill window shown". A fresh-start run (Crest restarted, ~1 min hidden) missed the first appearance (music already playing when the probe started).
  - **Decision (user):** accept it as boot-only (Windows' busy startup minute: many `Run` programs at once); no change. A Crest-side cost of the very first show of a fresh process isn't ruled out (unverified).
  - The probe stops after the first measured appearance, so the user can press play whenever. Windows PowerShell 5.1 wrote "±" garbled into a redirected file → plain "+/-".
- **Rule from now on:** installers, the installed Crest, autostart and registry checks are done by the user outside the Claude app (Explorer, Start menu, a normal Windows Terminal), with exact commands from me. A Crest path under `Packages\Claude_…\LocalCache` = the containerized copy.
- **Learned:** **packaged-app containers (MSIX)**: a Store-style app's child processes inherit its virtual file system and registry. A test is only as real as the environment it runs in; check the path of what's actually running.

## 2026-10-05 - Release 0.3.0: build + install test over 0.2.0
- **Done:** version 0.2.0 → 0.3.0 (`package.json`, `package-lock.json` ×2, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock`), commit `c93511f` "Release 0.3.0" on `main` (pushed). Installer `Crest_0.3.0_x64-setup.exe` (1.39 MiB), built in 2 min 16 s. A "Create PR" press opened PR #2; the user wants "push to main as usual" instead, so `main` was pushed and GitHub marked #2 merged by itself; branches `release-0.3.0` deleted locally and on GitHub. Not published yet.
- **Before the test:** Start with Windows was **off** in the installed 0.2.0 (no `Run\Crest`). The user switched it on once and reported it checked, but `Run\Crest` was still missing (also checked outside the sandbox, same user account, Startup folder/RunOnce/scheduled tasks empty). Re-test with Crest 0.2.0 started by me with its error output logged: checkmark at start = off (matched the registry), one click → `Run\Crest` = `C:\Users\<user>\AppData\Local\Crest\crest.exe ` (note the trailing space from `auto-launch`'s "{path} {args}"), error log empty. **Unexplained:** the first "checked without a `Run` entry"; not reproduced.
- **Update test (verified by testing):** 0.3.0 installed over 0.2.0 with the preselected **"Uninstall before installing"** → **`Run\Crest` deleted**; Crest no longer starts at login. Uninstall entry says 0.3.0; Crest 0.3.0 runs.
  - **Cause (verified from Tauri's NSIS script):** on an upgrade the installer preselects "Uninstall before installing", which runs the old uninstaller without `/UPDATE`; the uninstaller deletes `HKCU\…\Run\Crest` whenever it isn't in update mode. "Do not uninstall" would keep it. So the open Phase 1 question ("does installing over an older version keep autostart?") is answered: **not with the default choice**.
- **Also lost: `settings.json`.** Cause: the user ticked "Delete the application data" in the old uninstaller, which deletes `%APPDATA%\dev.crest.pill` (by design). `%LOCALAPPDATA%\dev.crest.pill` survived (318 of 332 files older than the update); likely files still locked by WebView2 helpers of the 0.2.0 copy I had closed by force seconds before (unverified). With the box unticked (the default), the uninstaller keeps both folders (verified from source).
- **Proposed fix (not built):** Crest stores the user's "Start with Windows" choice in `settings.json`; at startup, if it should be on but `Run\Crest` is missing, it registers itself again. The first start after any update then repairs the installer's deletion. Decide before publishing 0.3.0.
  - **Decision (user): fix first** (option 1), on branch `autostart-repair`, before publishing.
  - **Done:** `launch_at_login.rs` → folder `launch_at_login/`: `launch_at_login_switch.rs` (the old file, unchanged), `launch_at_login_repair.rs` (new; pure `decide_launch_at_login_repair(saved choice, entry present)` with 3 tests + the startup step), `windows_run_key_registration.rs` (new; `RegGetValueW` on `HKCU`/`HKLM\…\Run`, feature `Win32_System_Registry` of the existing `windows` crate, nothing downloaded). `settings.json` gets `shouldLaunchAtLogin` (`None` until seen); the settings switch saves the choice; `lib.rs` runs the repair right after loading the settings.
  - **Rules:** wanted + entry missing → register again. No saved choice + entry present (coming from 0.2.0 with "Do not uninstall") → adopt it as wanted. It never removes an entry, and an entry Task Manager disabled still counts as present, so a Task Manager "off" isn't overridden (that's why it reads the `Run` key itself: the plugin's `is_enabled` also says "off" for a Task-Manager-disabled entry). Skipped in debug builds, which would register `target\debug\crest.exe`.
  - **Limit:** coming from 0.2.0 with the default choice, the entry is deleted and 0.2.0 never saved the choice, so the user switches it on once more. From 0.3.0 on it repairs itself.
  - **Verified from Tauri's NSIS script:** with the *same* version installed, the page offers "Add/Reinstall components" (preselected, no uninstall) or "Uninstall Crest", which runs the old uninstaller without `/UPDATE` (deletes `Run\Crest`) and then continues installing, so the update case can be reproduced without a new version number.
  - clippy clean, 50/50 Rust tests.
- **Not tested yet:** Crest actually starting at login (needs `Run\Crest` back and a restart).
- **Learned:** **read the installer before trusting "settings survive updates".** Two different things are deleted on an upgrade: the autostart entry (always, with the default choice) and the app data (only if the box is ticked). An app can't stop the installer, but it can notice at startup what got lost and repair it.

## 2026-10-04 - Phase 2, item 8: several activities at once (design only, no code)
- **Today (verified from code):** `activity_arbiter.rs` keeps one update per source and exactly one wins (highest `display_priority`, then most recent); the pill shows only that one.
- **Activities expected from Phase 3:** long-running (music, a running timer, Claude Code working), alerts that need the user now (timer done, Claude Code Allow/Deny, meeting starting), ambient (weather, next event).
- **Options considered:**
  - A. One at a time by priority (today + alerts): no work, but music disappears for as long as a timer runs.
  - **B. Split notch: main activity + companion segment** (iPhone-style), alerts take over.
  - C. Rotate every few seconds: everything visible in turn, but distracting, unpredictable to click, and a recurring timer against the event-driven rule.
- **Decision (user): B.**
  - Compact: one notch with two segments, `[ main activity | companion ]`; with 3+ ongoing activities the companion shows "+N" and the expanded view pages through all of them (dots).
  - Hover a segment → that activity opens in the big view. Click the companion → it becomes the main one (remembered until it ends).
  - Alerts take over the whole notch, open with their buttons, stay until answered or dismissed; several alerts queue (priority, then arrival).
  - Settings (with the first integration): "Activities" section with the priority order of kinds and each kind on/off.
- **What changes when it's built (Phase 3, with the second activity source):**
  - `ActivityUpdate.is_ongoing: bool` → three cases: lingering / ongoing / alert.
  - `activity_arbiter.rs`: "pick one winner" → a pure, unit-tested `arrange_pill_activities(...)` returning { alert or none, main, companion, count of the rest }; the user's swap is part of its input. New command `focus_activity(kind)`.
  - `pill_visibility_policy.rs` decides from the arrangement (visible while anything is ongoing or alerting).
  - Frontend: each activity view set gains a tiny companion view (icon + one short value); `pillContentPresenter.ts` mounts main + companion; the compact interactive area widens while a companion is shown; alerts use the activity's expanded view with buttons; answers go through the existing `perform_activity_action`.
  - Unchanged: one notch shape, one rectangular interactive area, window size (the expanded view is already wider than compact + companion), the music source apart from the new field.
- **Learned:** separate **what an activity is** (lingering / ongoing / alert) from **where it goes** (the arrangement): sources describe themselves, one pure function decides the layout, as the visibility policy already does for show/hide.

## 2026-10-04 - Phase 2, item 7: README brought up to date
- **Done:** `README.md` was still at v1 in places. Updated: notch description and status (0.2.0 published, 0.3.0 in progress); tray = Settings… + Quit, second launch opens Settings; privacy wording from the new CLAUDE.md rules; memory while hidden (16–32 MB, measured); fullscreen hide in the visibility rules; `MediaPlayerFilter`; the window now moves only when another monitor is chosen; new "The settings window" section (on demand, per-window permissions); the whole file tree (added `settings.html`, `src/settings/`, `capabilities/settings_window.json`, `fullscreen_detection/`, `settings_window/`, `launch_at_login.rs`, the new media and pill_window files; `user_settings_file.rs` → `user_settings_store.rs`); the Settings section rewritten for the settings window, the file's fields, the new matching rule and what changed for hand-written files from before 0.3.0.
- **Next:** user reviews the README; commit ("Update README", no co-author), then merge `settings-window` into `main` and push when the user says so. Then item 8 (design only).

## 2026-10-04 - Phase 2, item 7, step 3: allowed players
- **Done (branch `settings-window`):**
  - `media/media_player_filter_control.rs` (new, platform-neutral): `MediaPlayerFilterControl` lists the players open now and replaces the allowed-player filter while the media source runs. Created by `MediaSource::create_player_filter_control()` (new trait method) before the source goes into the music activity; managed by Tauri for the settings commands.
  - Windows side: `SmtcMediaSource` now creates its channel in `new()` (so the control exists before watching starts). New worker message `AllowedAppFilterReplaced`; the tracker swaps the filter and the worker publishes right after, so the pill follows **at once** (no restart, no polling). The tracker also writes every session's app ID into the shared "open players" list whenever Windows reports a session-list change. Listing sessions + subscribing moved from the tracker into `smtc_tracked_session.rs` (`subscribe_to_current_sessions`), because the tracker had grown to 162 lines; it's 153 now.
  - `media/media_app_identifier_label.rs` (new, 3 tests): "YouTube Music" for the known ID in any browser, "<Browser> web app" for other `._crx_` IDs, "Spotify" for `Spotify.exe`, the package name for Store apps, otherwise the raw ID.
  - `settings_window/allowed_player_options.rs` (new, 2 tests): saved list + open players not matched yet (all open players while the list is empty), sorted, duplicates removed. Commands `list_allowed_player_options`, `change_allowed_players` (save, hand to the media source, return the new options), permitted for the settings window only.
  - Settings page: `allowedPlayersSettingCard.ts` (rows with Remove, "Add a player" dropdown of open players, "Every player" note when the list is empty; redrawn from Rust's answer after each change and refreshed when the window comes back to the front). Window height 560 → 640.
  - Rule kept from v1: an empty list = every player; the window says so in words.
- **Verified by testing:** clippy (incl. tests) clean, 43/43 Rust tests, tsc, vite build. **Not tested yet:** the card in the real window, the live switch of the pill, persistence across a restart.
- **User test → two problems (same day):**
  1. **Removing YouTube Music didn't stop the pill.** Cause (verified from code): removing the only entry sent an **empty** list, and the v1 rule "empty list = every player" (`is_app_identifier_allowed`) kept YouTube Music allowed. The card even said "Every player". While the list was empty the dropdown offered YouTube Music, so re-adding it saved the exact ID `Brave._crx_…` instead of the browser-independent `_crx_…`.
  2. **"No way to add another player."** At that moment Windows reported only YouTube Music (`list_media_sessions.ps1`), already in the list, so "No other player is open" was correct.
- **Measured (verified by testing, `list_media_sessions.ps1 -WatchSeconds 120`, YouTube tab in Brave + YouTube Music app):** a normal Brave tab is its **own** session with app ID **`Brave`**, next to `Brave._crx_cinhimbnkkghhklpknlkffjgod`; both exist at once. This exposed a third problem: matching by "contains" would make an entry `Brave` also match every Brave web app, YouTube Music included.
- **Fixed:**
  - `media/media_player_filter.rs` (new, 3 tests): `MediaPlayerFilter { EveryPlayer, OnlyListedPlayers(list) }` replaces "empty list = everything"; an empty list now means **no** player. Used by the selector, the worker message (`MediaPlayerFilterReplaced`), the filter control and the settings.
  - Matching is now **exact** (ignoring case), except an entry starting with `_crx_` (a browser-independent web-app ID), which matches that app in any browser. Hand-written v1 fragments like "spotify" no longer match "Spotify.exe" (accepted: the settings window now writes exact IDs; the README will say so).
  - `settings.json` gets `showEveryMediaPlayer`. A file without it (written before 0.3.0) is read as "on" if its list is empty, so an old "[] = every player" setup keeps working (test). The field has its own `#[serde(default)]`, so a missing field is `None`, not `Default`'s `Some(false)`.
  - New command `change_show_every_player`; the card has a "Show every player" switch and says "No players chosen … the pill shows no music" for an empty list. Adding YouTube Music saves the `_crx_…` part again. "Brave" is labeled "Brave tabs".
  - clippy clean, 47/47 Rust tests, tsc, vite build.
- **Verified by testing (user):** tested the fixed dev build and reported "it works" (results not reported item by item).
- **Learned:** **make "all" its own case.** Using an empty list to mean "everything" was fine while the list was only edited by hand, but a Remove button makes "empty" a normal state, and then it silently meant the opposite of what the user did. An enum says what it means.

## 2026-10-04 - Phase 2, item 7, step 2: General + Display settings
- **Decisions (user):** "Start with Windows" leaves the tray (tray = "Settings…" + "Quit Crest"). New: choose the monitor the pill sits on, as a setting (option A); dragging the pill between monitors (B) maybe later, it would write the same setting.
- **Measured before designing (verified by testing):** DISPLAY1 2560×1440 at x 0 (main), DISPLAY2 1920×1080 at x 2560 (right), **both 100 % scaling**. So moving between them never changes the pill's pixel size here; the rescale path is written but can't be tested on this PC.
- **Done (branch `settings-window`):**
  - `user_settings_store.rs` (renamed from `user_settings_file.rs`): `CrestUserSettingsStore` holds the current settings behind a `Mutex` (managed by Tauri) and saves on every change: write `settings.json.tmp`, then rename over `settings.json` (a crash mid-write can't leave half a file). New field `pillDisplayName` (`None` = main display). 2 tests (round trip; an older file without the new field keeps its values).
  - `pill_window/pill_display_choice.rs` (new, 3 tests): chosen monitor while connected, else main; labels like "Display 2 · 1920 × 1080 · right of main"; "Main display" is its own entry. `connected_display_reader.rs` (new): Tauri monitors → that plain type. `pill_window_placement.rs`: places on the chosen monitor (+1 test for the second monitor).
  - `settings_window/settings_window_commands.rs` (new): `read_launch_at_login_setting`, `change_launch_at_login_setting`, `list_pill_display_options`, `choose_pill_display` (saves, then emits `pill-display-changed` to the pill). `launch_at_login.rs`: `toggle_` → `set_launch_at_login(should)`.
  - Pill page (`main.ts`, `pillMorphController.ts`): on `pill-display-changed` and on Windows rescaling the window, place again and resend the current interactive area (`applyCurrentInteractiveArea`).
  - Settings page: `src/settings/settingsWindowMain.ts`, `settingRowElement.ts`, `launchAtLoginSettingRow.ts` (switch shows Windows' answer, flips back on failure), `pillDisplaySettingRow.ts` (dropdown); switch/dropdown styles + tokens (Windows 11 accent blue, light/dark, `color-scheme: light dark` for the native dropdown list).
  - **Per-window permissions:** `build.rs` lists Crest's commands (`AppManifest::commands`), so Tauri makes an `allow-<command>` permission for each and a window may only call what its capability grants. `capabilities/default.json` (pill) gets the pill's 7 commands, new `capabilities/settings_window.json` gets the 4 settings commands. Before this, any page could call any of Crest's own commands. Generated files in `src-tauri/permissions/autogenerated/` are git-ignored (rebuilt every build, like `gen/schemas`).
- **Verified by testing:** clippy clean, 38/38 Rust tests, tsc, vite build. In the user's running dev copy (22:48): pill window 412 px wide at top 0, centered on DISPLAY1 (left 1074), interactive area 236×36 at x 88 → the pill's commands pass the new permissions.
- **Problem (mine):** I started the release exe for a test while the user's dev copy was running (I had listed it but not acted on it). The single-instance guard counted it as a second launch, so the user's Crest opened Settings and took focus. Rule: if any `crest.exe` is running, stop and ask before starting another.
- **Learned:** **Tauri app permissions:** without an app manifest, every window may call every `#[tauri::command]` of the app; listing them in `build.rs` turns each into a permission that capability files grant per window (least privilege for your own commands, not just plugins).
- **Not tested yet:** the settings rows in the real window, moving the pill to DISPLAY2, persistence across a restart.
  - **Update (same day):** the user tested the dev build against the test list (tray, switch, display choice, restart, fullscreen on DISPLAY2, light/dark) and reported "it works"; results weren't reported item by item. The different-scaling path stays untested (both monitors at 100 %).

## 2026-10-04 - Phase 2, item 7, step 1: empty settings window + memory
- **Decisions (user):** A second Tauri window, created on demand and destroyed on close (not kept alive hidden: no memory held for a rarely used window). "Start with Windows" moves from the tray into the window (step 2). Starting Crest again while it runs opens Settings. Later there will be more ways to open Settings (a settings wheel), so there's exactly one opener function.
- **Done (branch `settings-window`):**
  - `settings_window/settings_window_opener.rs` (new): `open_or_focus_settings_window` focuses an open window or builds a new one on its own thread (verified from Tauri docs: building a webview window inside a menu/event handler deadlocks on Windows). Window: "Crest Settings", 480×560, not resizable, centered.
  - `system_tray.rs`: "Settings…" item (the "Start with Windows" checkmark stays until step 2). `lib.rs`: the single-instance callback opens Settings.
  - `settings.html` + `styles/settingsWindow.css` (new, Windows 11 settings look, light/dark via `prefers-color-scheme`), settings tokens in `designTokens.css`; `vite.config.ts` builds two pages (`rollupOptions.input`), so settings code never loads into the pill.
- **Verified from source (Tauri NSIS script):** the uninstaller deletes `%APPDATA%\dev.crest.pill` (settings.json) only if "Delete the application data" is ticked (unticked by default) and never in update mode. So settings survive updates.
- **Problem found and fixed (old bug since item 4):** the pill sets the WebView2 memory target "Low" only when it hides, and it only hides after being shown. A Crest started with nothing playing (e.g. at login) stayed at "Normal": **78.3 MB**. Fix: `prepare_pill_window_as_overlay` also sets "Low" (the window starts hidden).
- **Measured (verified by testing, release build, nothing playing, pill hidden, private working set):**

  | State | Total | Notes |
  |---|---|---|
  | Never opened, before the fix | 78.3 MB | 60 s after start |
  | Never opened, with the fix | 31.8 / 32.8 MB | 60 s after start; again 50 s later |
  | Settings open | 66.0 MB | +1 renderer (15 MB); manager 10→23, GPU 3→9 |
  | 3 s after closing | 50.2 MB | the settings renderer is gone |
  | ~1 min after closing | **15.9 MB** | manager 4.9, pill renderer 0.2 |

  The settings page gets its **own** renderer process; the pill's renderer kept its "Low" level the whole time.
- **Verified by testing:** the second launch opened the window ("Crest Settings" title) and the extra copy exited. 32/32 Rust tests, clippy, tsc, vite build.
- **Unexplained:** never-opened settles at ~32 MB, after-close at ~16 MB. Maybe closing a webview makes WebView2 trim the shared processes (unverified). Both are far below the 78 MB before the fix.
- **Learned:** **multi-window apps in Tauri:** windows from `tauri.conf.json` exist from the start; windows built in Rust with `WebviewWindowBuilder` exist only while open. Closing a Tauri window destroys it (and here, its renderer process) unless the close is intercepted.

## 2026-10-04 - Phase 2, item 6: CLAUDE.md rules for v2
- **Decisions (user approved the exact wording):** in CLAUDE.md "Scope and platforms":
  - "Not in v1: …" → scope follows the roadmap here; not planned yet: Linux (Phase 5), notifications, volume/brightness popups; integrations one at a time, each approved first. (The old line also excluded a settings UI, which item 7 needs.)
  - "No network calls, no API keys" → network only for an integration the user switched on, only to that service, all off by default; free APIs only (no paid plans, no free tiers that need a payment method); keys/tokens only in Windows Credential Manager behind a trait, never in files, settings.json, logs, the repo or the page (the page may hand a token to Rust once, never reads it back); network calls only from Rust, the page's CSP stays closed; no telemetry, analytics or remote crash reports. SMTC-only music rule unchanged.
  - "No polling loops" gets one exception: an integration that can only fetch (weather, calendar) may use a coarse timer while it's switched on.
- **Why the two extra rules (mine, approved):** if the page could reach the network, a bug in it (e.g. markup in a song title) could too; keeping network and tokens in Rust keeps that surface small.
- **Open (Phase 3):** Credential Manager access either via the `keyring` crate (new install → ask) or the `Win32_Security_Credentials` feature of the `windows` crate we already have (nothing new downloaded).
- **Also fixed:** a broken path in "Release build 0.2.0" below (`target\release` had been saved with `\r` as a real line break).

## 2026-10-04 - Phase 2, item 6b (new): notch look, decided
- **Request (user):** the pill should be glued to the top edge (no 8 px gap) and look like a notch.
- **Decisions (user):** option B: flat top, rounded bottom corners, plus curved "shoulders" that flare into the screen edge; hiding slides up into the edge instead of shrinking and fading. Rejected: A (flat top without shoulders).
- **Expected trade-off (to watch in daily use):** pushing the mouse into the top center now lands on the pill and expands it after 150 ms; that's where a maximized window's title bar or a browser tab would be.
- **Next:** build it on branch `notch-look`.
- **Done (same day, branch `notch-look`):**
  - Rust: `PILL_WINDOW_TOP_MARGIN_LOGICAL_PIXELS` removed; `pill_window_placement.rs` puts the window at the monitor's top edge (tests now expect y = 0).
  - `pillShellElements.ts`: new outer element `.pill-notch` around the capsule. It draws the shoulders (`::before`/`::after`) and carries the hide slide. Needed because the capsule has `overflow: hidden` (it hides the expanded layer while small), which would clip shoulders drawn on the capsule itself.
  - `pillShell.css`: capsule corners `0 0 r r` (flat top); shoulders = an 8 px square with a quarter circle cut out by a `radial-gradient` (1 px soft edge against jaggies); hidden = `translateY(-100%)` on the notch instead of fade + shrink. Token `--pill-concealed-scale` removed.
  - `frontendConstants.ts`: `PILL_NOTCH_SHOULDER_LOGICAL_RADIUS = 8`; window width = expanded + 2 × (overshoot room + shoulder) = 412 (was 396). `pillMorphController.ts`: the compact interactive area includes the shoulders (236 × 36), since the window region also clips what's drawn.
  - `pillVisibilityController.ts`: takes `PillShellElements`; hide classes go on the notch; the end of a hide is the notch's `transform` transition (was the capsule's `opacity`). The "pointer left" listener stays on the capsule.
- **Verified by testing:** tsc clean, clippy clean, 32/32 Rust tests. Browser preview (page only, no Tauri, transitions turned off by hand to see end states): compact 220 × 36 at y = 0 with radius `0 0 18 18`, expanded 380 × 176 with `0 0 30 30`, shoulders visible on both sides of both sizes.
- **Not tested yet:** the real window (top edge, region with shoulders, slide-in/out animation, hover). Hovering a shoulder doesn't expand the pill (only the capsule listens); intended, as they're 8 px of decoration.
  - **Update (same day):** the user ran the dev build (`target\debug\crest.exe`, 20:37) and approved the commit. Which checks of the list were done wasn't reported item by item. Still open: whether the top-center hover triggers too often in daily use.
- **Learned:** **`overflow: hidden` clips pseudo-elements too**, because `::before`/`::after` are children of the element. Anything that must stick out of a clipping box goes on a wrapper.

---

## 2026-10-04 - Release build 0.2.0
- **Done:** version 0.1.2 → 0.2.0 (`package.json`, `package-lock.json` ×2, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock`), on branch `release-0.2.0`. `npm run tauri build` in 127 s → `Crest_0.2.0_x64-setup.exe` (1.36 MiB). Contains Phase 1 items 1–4.
- **Problem avoided:** the running Crest was `target\release\crest.exe`; a build can't overwrite a running exe on Windows, so Crest was stopped first.
- **Install test:** no Crest was installed any more (no `%LOCALAPPDATA%\Crest`, no uninstall entry), so this was a **fresh install**, not an update over 0.1.2. Installed 20:01 → `%LOCALAPPDATA%\Crest\crest.exe`, uninstall entry "Crest 0.2.0"; the user confirmed it works (pill, buttons, fullscreen hide).
- **Not tested (user's choice, no restart possible):** Crest actually starting at login, the unquoted path in the `Run` value with the installed exe, and the update path from 0.1.x ("uninstall first" question). The on/off switch itself was verified in dev (item 2).
- **Published (2026-10-04):** GitHub Release v0.2.0 with `gh release create`, tag `v0.2.0` on commit `898fdc6` ("Release 0.2.0"), asset `Crest_0.2.0_x64-setup.exe` (1 423 637 bytes), marked Latest; text approved by the user, without the untested claim that it updates 0.1.x in place.
- **Learned:** `gh release create <tag> <file> --target <commit> --title … --notes-file …` creates the tag on GitHub, uploads the installer and publishes in one step; `git fetch --tags` brings the new tag to the local copy.
- **Next:** Phase 2, item 6.

---

## 2026-10-04 - Phase 1, item 4: WebView2 memory (measured)
- **Correction to earlier notes:** "388 MB, of which ~330 MB WebView2" (milestone g) was the **sum of the processes' working sets**. That counts the WebView2/Edge program code, which Windows loads once and shares, once per process (7 processes). What Crest really occupies in RAM is the **private working set** (Task Manager's "Memory" column): ~81–86 MB.
- **Done:**
  - `dev-tools/measure_crest_memory.ps1` (new): finds Crest and every WebView2 process it started, names their role (manager, renderer, GPU, network/storage helpers, crash reporter) and averages private working set, private bytes and working set over N samples.
  - `pill_window/windows_native/webview_memory_usage_target.rs` (new): `MemoryUsageTargetLevel = Low` when the pill window is hidden, `Normal` before it's shown (called from `hide_pill_window` / `show_pill_window_without_activating`). Uses `webview2-com` 0.39 (user approved; the same crate and version Tauri already uses, so nothing new is compiled; +1 line in `Cargo.lock`).
- **Measured (verified by testing, release builds, 10 samples × 2 s, private working set; baseline exe kept as a copy so both builds were compared on the same day):**

  | State | Baseline | Memory target "Low" while hidden |
  |---|---|---|
  | Hidden (60 s after start, paused) | 80.9 / 86.2 / 83.2 MB (3 runs) | **19.6 MB** |
  | Playing, compact | 85.7 MB | **60.5 MB** (measured after a hidden → playing switch) |

  Private bytes stay the same (~157–183 MB): WebView2 lets Windows move the memory out of RAM instead of freeing it; it comes back when needed. Run-to-run spread ~5 MB. The user saw no difference when the pill reappeared (animation, art, bars).
- **Decisions:**
  - Kept: memory target "Low" while hidden (−64 MB hidden, −25 MB playing).
  - Not used: `TrySuspend` (verified from docs: pauses scripts too; the hidden page must keep listening for "show yourself"). Browser start-up switches (e.g. network service inside the manager process): at most ~6 MB playing / ~1.5 MB hidden left to win, and Tauri's own default switches would have to be repeated by hand.
- **Learned:**
  - **Which memory number:** private working set = RAM only this process uses; working set = also shared pages (don't sum over processes); private bytes = reserved, including what's paged out. Task Manager's default "Memory" column is the private working set.
  - **"Low memory target"** doesn't free memory, it allows Windows to page it out; good for an app that's hidden most of the time, as long as waking up stays fast.
  - **Measuring pitfalls hit today:** a running `tauri dev` restarts the debug build after every file change, and the single-instance guard then blocks the release build; check which `crest.exe` (path) is running before every measurement.
- **Next:** commit; then v0.2.0 (version bump, installer, install over 0.1.2 incl. the autostart checks, GitHub release).

---

## 2026-10-03 - Phase 1, item 3: grey out buttons the player doesn't accept
- **Measured (verified by testing, `list_media_sessions.ps1 -WatchSeconds`, YouTube Music in Brave, 6 quick skips, pause/play):**
  - `next`, `previous` and `toggle` are **always** enabled, also around track changes: between tracks the session disappears for 0.1–0.8 s and comes back with every flag on. Crest's 1.5 s grace period already bridges that gap, so the buttons can't flicker.
  - `play` / `pause` flip with the state (`play=False` while playing, `pause=False` while paused). Tying the play/pause button to them would grey it out all the time → it follows `IsPlayPauseToggleEnabled`, matching the toggle command it sends.
- **Done:**
  - `media_session_snapshot.rs`: `MediaControlAvailability { can_toggle_play_pause, can_skip_to_next_track, can_skip_to_previous_track }` in the snapshot (`availableControls` in JSON); 1 test pins the JSON names the frontend uses.
  - `smtc_snapshot_reader.rs`: reads `PlaybackInfo.Controls()` with every snapshot (same `PlaybackInfoChanged` event as before, no polling); if unreadable, all buttons stay usable (the player can still decline).
  - Frontend: `nowPlayingTypes.ts` (`NowPlayingControlAvailability`), `musicControlButtons.ts` (`showAvailableControls` sets `disabled`), `expandedMusicView.ts`, `musicControlButtons.css` (`:disabled` dimmed, hover circle only on `:enabled`), token `--control-button-disabled-opacity: 0.3`.
  - `dev-tools/list_media_sessions.ps1`: watch mode now also prints the button flags.
  - clippy clean, 32/32 tests, tsc, vite build.
- **Verified by testing (page level):** real YouTube Music data arrives as all-enabled; an injected update with previous/next refused gives `disabled=true` and computed opacity 0.3 on those two, play/pause unchanged.
- **Problem (mine):** to see the test I injected a fake "pill visible" event while the user had a fullscreen game in front, which overrode the fullscreen hide: the pill sat on the game for ~1 min until I re-sent Rust's real visibility. A screenshot taken then didn't match the page state (probably a stale frame over the game) and isn't used as evidence. Rule for later tests: never inject visibility; only inject content while the real pill is visible.
- **Learned:** **disabled buttons** in HTML don't fire `click`, can be styled with `:disabled`, and screen readers announce them as unavailable, so no extra click guard is needed.
- **Verified by testing (user):** with the injected update, previous/next look dimmed and don't react to clicks; the next real update from YouTube Music (pause/play) brings them back, because YouTube Music really allows them. The user first took that for a bug; the pill always shows the player's latest real state, so a button only stays dimmed while the player itself keeps refusing it.
- **Not tested with a real refusing player** (YouTube Music never refuses; a plain YouTube tab might, but Crest only shows YouTube Music by default).
- **Next:** item 4 (WebView2 memory).

---

## 2026-10-02 - Phase 1, item 2: "Start with Windows" in the tray
- **Done:**
  - Installed `tauri-plugin-autostart` 2.7.0 (user approved; Rust only, no npm package). 3.0.0-alpha.2 exists on crates.io but is a pre-release. New in `Cargo.lock`: `auto-launch` 0.6, `windows-registry` 0.6, `os_info`; macOS/Linux-only crates are listed but not compiled on Windows; `tauri-utils` 2.10.0 → 2.10.1 (patch, needed by the plugin).
  - `launch_at_login.rs` (new): `is_launch_at_login_enabled` / `toggle_launch_at_login`; always reads the real state back from Windows.
  - `system_tray.rs`: checkmark item "Start with Windows" + separator above "Quit Crest"; after a click the checkmark is set to what Windows reports.
  - `lib.rs`: `tauri_plugin_autostart::Builder::new().build()`. **No** permission for the page in `capabilities/` (the page can't switch autostart).
  - Off by default.
- **Verified by testing (user clicked, I read the registry):** on → `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value **`Crest`** = path of the running exe, plus `...\Explorer\StartupApproved\Run\Crest` = `02 00…` (enabled in Task Manager); nothing in HKLM. Off → the `Run` value is deleted. clippy clean, 31/31 tests.
- **Verified from source:**
  - The plugin registers `current_exe()`, so from `tauri dev` it registers `target\debug\crest.exe` (useless at login without Vite) → only test on/off in dev; the login test happens with the installed v0.2.0.
  - Tauri's NSIS uninstaller (CLI 2.12.0, `installer.nsi`) already deletes `HKCU\...\Run\${PRODUCTNAME}` and skips it during updates (`/UPDATE`), so no installer hook of our own was needed. The value name matches ("Crest").
  - Rust calls to the plugin (`app.autolaunch()`) don't go through permissions; `capabilities/` only limits what the page's JavaScript may invoke.
- **Problems / limits:**
  - "Off" and the uninstaller leave the `StartupApproved\Run\Crest` bytes behind; harmless (Windows only launches what's under `Run`), overwritten by the next "on".
  - The path is stored without quotes. Fine for the installed `%LOCALAPPDATA%\Crest\crest.exe` (no spaces); the dev path has a space ("Dynamic Island"). Re-check with v0.2.0.
  - Open: when upgrading by running a newer installer over the old one and choosing "uninstall first", the old uninstaller runs **without** `/UPDATE` and would remove the autostart entry. Check during the v0.2.0 install.
- **Learned:**
  - **`Run` key:** programs listed in `HKCU\...\CurrentVersion\Run` start at login; Task Manager's "Startup apps" shows them and stores enabled/disabled in `StartupApproved\Run`.
  - **Least privilege with plugins:** a plugin has a Rust API (no permission needed) and page commands (need permissions); grant the page nothing it doesn't use.
  - **Read the tool's own code first:** Tauri's installer script already did the cleanup we were about to write.
- **Next:** PR `autostart` → `main`; then item 3.

---

## 2026-10-02 - Phase 1, item 1b: the pill steps aside for fullscreen apps
- **Decision (user):** from now on, work goes through **pull requests**: one short-named branch per milestone, PR into `main` with `gh`. Replaces "no pull requests for now" (Roadmap entry).
- **Done:**
  - `fullscreen_detection/` (new): `fullscreen_app_watcher.rs` (trait `FullscreenAppWatcher`, like the other platform traits); Windows in `windows_shell_appbar/`: `appbar_fullscreen_app_watcher.rs` (own thread + hidden window + message loop), `appbar_watcher_window_procedure.rs` (reacts to `ABN_FULLSCREENAPP`, the settle timer and `TaskbarCreated`), `appbar_watcher_thread_state.rs` (what the window procedure needs, in a `thread_local`), `front_window_fullscreen_check.rs` (front window covers the **pill's** monitor and isn't the desktop `Progman`/`WorkerW`). Split in four because one file was ~190 lines mixing setup, message handling and state.
  - "Closed" waits `FULLSCREEN_APP_LEAVE_SETTLE_DELAY` (250 ms) with a Win32 timer and is cancelled by a new "opened": no flash during Valorant's 20 ms mode-switch flicker.
  - `pill_visibility_controller.rs`: visible = wanted by the activity rules **and** no fullscreen app; reports `PillVisibility { isPillVisible, isFullscreenAppInFront }` (was a bool); 3 new tests. Hide countdowns keep running meanwhile, so a pause during a game still ends with the pill hidden.
  - Frontend: `pillVisibilityTypes.ts` (new); `pillVisibilityController.ts` hides instantly (class `is-concealed-instantly` = no transition) and doesn't wait for the pointer to leave when a fullscreen app is the reason; tells the presenter when the pill is on/off screen → `ActivityViewSet.setPillOnScreen` → the bars stop while hidden.
  - Cargo feature `Win32_System_LibraryLoader` (for `GetModuleHandleW`; part of the existing `windows` crate, nothing downloaded).
  - Verified by testing: clippy clean, 31/31 Rust tests, tsc, vite build; dev app starts with no errors and the page receives the new visibility object.
- **Verified from code:** tao (Tauri's window layer) makes the process per-monitor DPI aware (`tao/src/platform_impl/windows/dpi.rs`), so window and monitor sizes are in real pixels, as in the probe.
- **Learned:**
  - **`thread_local!`:** a window procedure is a plain function Windows calls, with no `self`; a thread-local variable gives it the state of the thread that owns the window, without a `Mutex`.
  - **Raw handles and threads:** `HWND` is a raw pointer, which Rust won't send to another thread; passing the number (`isize`) and rebuilding the handle there is the usual way when the other thread only reads.
  - **Win32 timers:** `SetTimer` on a window posts `WM_TIMER` into its message loop, so a delay costs no thread and no polling; `KillTimer` cancels it.
- **Not handled (accepted):** in exclusive fullscreen, a short alt-tab may not send "closed" (seen in the probe), so the pill stays hidden like the taskbar. The appbar isn't removed when Crest quits; the shell drops registrations of windows that no longer exist (unverified).
- **Verified by testing (user + state recorder):** the pill vanishes instantly in a fullscreen game, the bars stop ~1 s later while the music keeps playing, and both come back after alt-tab. Not yet seen in practice: fullscreen on the second monitor, desktop clicks, a 30 s pause during a game.
- **Next:** pull request `fullscreen-hide` → `main`; then item 2.

---

## 2026-10-02 - GitHub CLI installed
- **Done:** `winget install --id GitHub.cli --exact --source winget` → `gh` 2.102.0 in `C:\Program Files\GitHub CLI\` (user asked for it). Not logged in yet: the user runs `gh auth login` once (browser login; the token is stored by `gh` in Windows Credential Manager, never in the repo).
- **Learned:** a newly installed program's folder is added to PATH only for terminals opened *after* the install; old windows still say "gh is not recognized".
- **Open questions:** should the app's "Create PR" button now open real pull requests (branch per milestone) instead of committing to `main`? Until the user decides, the old decision (no PRs, commits to `main`) stands.
  - **Answer (user, same day):** yes, real pull requests.

---

## 2026-10-02 - Phase 1, item 1a: fullscreen probe
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

## 2026-10-02 - Phase 1, item 1: how to detect a fullscreen app (research, no code yet)
- **Decision (user):** Phase 1 confirmed in this order: fullscreen hide → autostart → greyed-out buttons → WebView2 memory → v0.2.0 release on GitHub.
- **Options found:**
  - **A. `SHQueryUserNotificationState`** (verified from docs): returns `QUNS_BUSY` (fullscreen app or presentation mode) or `QUNS_RUNNING_D3D_FULL_SCREEN` (exclusive Direct3D). A question you have to ask, never a notification → only usable with polling. No monitor information.
  - **B. `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)`** + own check "does the front window cover its whole monitor?" (verified from docs): event-driven and knows the monitor, but only fires when the front window *changes*, so it misses a window that becomes fullscreen while already in front (F11, a game switching display mode).
  - **C. Register as an appbar (`SHAppBarMessage(ABM_NEW)`) and receive `ABN_FULLSCREENAPP`** (verified from docs): the shell's own fullscreen detection, the one that hides the taskbar; sent when the first fullscreen app opens (`lParam` TRUE) and the last one closes (FALSE). Event-driven, also catches F11. Weak spots (unverified, from forum/GitHub reports): no monitor information (this PC has 2 monitors: primary 2560×1440, second 1920×1080 on the right); clicking the desktop can look fullscreen (window classes `Progman`/`WorkerW`); the registration is lost when Explorer restarts (re-register on the `TaskbarCreated` message).
- **Proposed:** C as the trigger, plus our own check on each notification (front window covers the *pill's* monitor and isn't the desktop). Before building it, a small read-only probe in `dev-tools/` logs what Windows actually sends while the user plays Valorant (fullscreen and windowed fullscreen), uses F11 on each monitor, alt-tabs and clicks the desktop.
- **Learned:** the reference project (macOS) does the opposite: it deliberately stays visible over fullscreen apps, so nothing to reuse here. Anything that injects into another process (in-context hooks) is off-limits next to anti-cheat (Vanguard); A, B and C are all passive.
- **Open questions:** hide instantly (no fade over the game) or with the usual animation? Does exclusive fullscreen trigger C? (probe will show)

---

## 2026-10-02 - Roadmap (proposed, order not confirmed yet)
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
  - **Added (2026-10-04, user):** 6b. Notch look: glued to the top edge, notch shape with shoulders, slides up when hiding.
- **Phase 3: integrations, free only → v0.4.0+:** Claude Code sessions with Allow/Deny (local), weather (Open-Meteo, no key), calendar (private ICS link), GitHub (free token).
- **Phase 4: personality:** own visual identity (character, idle animations, sounds), never Coucou's Mochi.
- **Phase 5: Linux** (KDE Plasma, Wayland): MPRIS + layer-shell.
- **Decisions (user):** no GitHub issues or pull requests for now; the roadmap lives here and commits go straight to `main`.
  - **Correction (2026-10-02, later):** `gh` installed; milestones now go through pull requests (see item 1b).
- **Next:** user confirms or reorders; suggested first step is 1 (hide during fullscreen games).

---

## 2026-10-02 - "Toy project" → "hobby project"
- **Decision (user):** the README now calls Crest a personal hobby project instead of a toy project; it had outgrown "toy" (public releases, tests, plugin architecture). The disclaimer stays: works on the author's setup (Windows 11, YouTube Music in Brave), shared as-is, no support guaranteed.

---

## 2026-10-02 - Direction: integrations later, free APIs only
- **Decisions (user):** Crest will grow toward a Coucou-like app with integrations, but **only free APIs, never paid ones**. Not started yet; no integration chosen.
- **Open questions (decide before the first integration):**
  - CLAUDE.md currently says "no network calls, no API keys" and "not in v1: other activity sources". It must be updated first. Proposed: network only for integrations the user enables; keys in Windows Credential Manager, never in files or the repo; still no telemetry.
  - Candidates discussed: Claude Code sessions (local, no key), GitHub (free, personal token), weather via Open-Meteo (no key), calendar via private ICS link (no key).
- **Learned:** "free" ≠ "no key": some free APIs need nothing, others need a free personal token.

---

## 2026-10-01 - Release build 0.1.2
- **Done:** version 0.1.1 → 0.1.2 (`package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock`); `npm run tauri build` in 3 min 44 s → `Crest_0.1.2_x64-setup.exe` (1.35 MiB). Contains all of today's fixes (title bar, layer fades, loading ring, single instance, held button presses).
- **Verified by testing:** with the dev copy running, the release `crest.exe` exits by itself (exit code 0): dev and release share the identifier `dev.crest.pill`, so the single-instance guard covers both. A full release startup wasn't re-tested (it would have meant closing the user's dev copy).
- **Learned:** the single-instance lock is per app identifier, not per exe file: to try the installed version, quit the dev copy first (and the reverse).
- **Next:** user uploads the installer as GitHub Release v0.1.2.

---

## 2026-10-01 - Button presses during a track change are no longer lost
- **Problem:** "no media session to send PreviousTrack to" ×4 in the user's log. Brave drops the media session for about 0.5 s on every track change; a press in that gap had no session to go to and was thrown away, so fast repeated next/previous skipped fewer tracks than pressed.
- **Done:**
  - `media/pending_transport_commands.rs` (platform-independent, 2 tests): holds presses in order with their time; `take_still_relevant` hands back those younger than `PENDING_MEDIA_TRANSPORT_COMMAND_LIFETIME` (2 s) and empties the list.
  - `smtc_session_tracker.rs`: the buttons' target app (`button_target_source_app_identifier`) is no longer cleared when the session vanishes. A press with no session is held; when the session list changes and the target app's session is back, held presses are sent.
  - `smtc_transport_commands.rs` now reports send failures itself; `smtc_tracked_session.rs` (new) holds `TrackedSmtcSession` and `find_session_of_app`. Split because the tracker grew to 166 lines; it's 146 now.
  - clippy clean, 28/28 tests.
- **Learned:** **borrowing fields separately.** A method `&self -> &Session` borrows the whole struct, so you can't change another field while holding the result. A free function that takes only `&self.tracked_sessions` borrows just that field, and Rust allows changing `self.pending_transport_commands` at the same time.
- **Decisions:** stale presses (older than 2 s) are dropped instead of fired late; no timer is needed, since age is checked when the session comes back.

---

## 2026-10-01 - Loading ring instead of Brave's logo on skip
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

## 2026-10-01 - Fixes from the user's screenshots: title bar, overlapping layers, Brave logo
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

## 2026-10-01 - Only one Crest at a time (single-instance plugin)
- **Problem (reported by the user):** in a test right after the corner fix, songs overlapped, the pill lagged, buttons barely worked, play/pause didn't work, and the corners were still there. Most likely cause (unverified, the copies were gone before I could check): two older release copies from 21:47 were still running under the new dev copy, so three pills were stacked in one spot. Each animates on its own; a click goes to whichever window is on top at that point; the old copies don't have the corner fix.
- **Done:** added the official `tauri-plugin-single-instance` 2.5 (Rust only, no npm package; user approved the install). Registered first in `lib.rs`; the "another copy was started" callback does nothing, because the pill only appears with music, so there's nothing to bring forward. Verified by testing: with one copy running, a second `crest.exe` exits by itself (exit code 0) within 4 s; clippy clean, 23/23 tests.
- **Learned:** a Tauri plugin is added on the Rust side with `.plugin(…)` on the builder; some plugins also have an npm package for a JS API, this one doesn't. The plugin must be registered first so the second copy exits before creating windows or a tray icon. On Windows it uses a named mutex plus a message to the first copy.
- **Also found (from the user's log):** "no media session to send PreviousTrack to" ×4. During a track change YouTube Music's session disappears for about 0.4 s, and a button press in that gap is dropped. To fix next: hold the press until the session is back.

---

## 2026-10-01 - Fix: white "old app" corners after clicking the pill
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

## 2026-10-01 - Publishing on GitHub
- **Done:** added `LICENSE` (MIT); README gained "Installing" (installer from Releases, SmartScreen note), build command, and a "License" section (toy project, not affiliated with Apple/Google/YouTube). Checked the tracked files for personal data before publishing: no emails, no personal paths, no build output.
- **Decisions:** public repo, MIT license. The user creates the empty repo on github.com; no GitHub CLI installed. The installer goes into a GitHub Release (uploaded by hand), not into git: build output never belongs in the repo.
- **Done (later):** pushed to https://github.com/Denis112500/Crest. Before the push, every commit's author email was changed from the personal Gmail to the GitHub private address (`196484913+Denis112500@users.noreply.github.com`, set as this repo's `user.email`); commit hashes changed, dates and messages didn't.
- **Learned:** every commit stores its author's name and email; on a public repo anyone can read them. GitHub's "noreply" address (Settings → Emails) still links commits to the account. Rewriting history is only safe before anyone else has it, i.e. before the first push.
- **Learned:** `git remote add origin <url>` links the local repo to GitHub; `git push -u origin main` uploads it and makes later `git push` calls go there. Git for Windows' Credential Manager handles the GitHub login in the browser on the first push.

---

## 2026-10-01 - First release build (version 0.1.1)
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

## 2026-09-30 - Milestone (g): hide/show logic, tray icon, app icon, edge cases
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
    - **Correction (2026-10-04):** the 388 MB summed shared memory several times; real RAM use was ~83 MB, now ~20 MB while hidden (see item 4).
  - Build an installer (`npm run tauri build`, NSIS) and start with Windows (autostart); not in the v1 scope.
  - Use SMTC's "control enabled" flags to grey out buttons the player doesn't support.
  - Linux (KDE Plasma, Wayland): `media/linux_mpris/` (MPRIS over D-Bus) and `pill_window/linux_layer_shell/` (layer-shell for position, always-on-top and input region).
- **Next:** v1 is done. Pick from the ideas above, or start the Linux port.

## 2026-09-30 - Milestone (f): control buttons + CPU fix for animations
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

## 2026-09-30 - Milestone (e): compact and expanded states with animation
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

## 2026-09-30 - Milestone (d): live data reaches the frontend
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

## 2026-09-30 - Milestone (c): Rust reads the media session
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

## 2026-09-30 - Milestone (b): pill window behavior with fake content
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

## 2026-09-30 - Milestone (a): empty Tauri app runs
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

## 2026-09-30 - Toolchain installed
- **Done:** installed Visual Studio Build Tools 2022 (workload "Desktop development with C++", includes Windows SDK 10.0.26100) and Rustup 1.29.1 via winget. `rustup default stable-msvc` → rustc/cargo 1.98.1. A hello-world compiled and ran (verified by testing), so the linker and SDK are found.
- **Learned:**
  - `rustup` is the installer/updater for Rust toolchains; `cargo` builds and runs Rust projects; `rustc` is the compiler that cargo calls.
  - The `-msvc` toolchain links with Microsoft's `link.exe`, which is why the Build Tools are needed. The other Windows toolchain (`-gnu`) isn't what Tauri supports on Windows.
  - Terminals opened before the install don't see `cargo` on PATH. Open a new terminal (or restart VS Code) after installing.
- **Problems:** the previous session was cut off mid-install, and the log confirmed both installs finished (exit 0).
- **Next:** finish milestone (a).

## 2026-09-30 - Plan approved
- **Decisions:**
  - File tree and milestones (a)–(g) approved as proposed (see the architecture entry below).
  - On track change the pill **expands for 4 s** ("peek"), then returns to compact.
  - Hide **30 s after pausing**, and **3 s after the session closes** (after the track-change grace period).
  - Working name **Crest**, identifier `dev.crest.pill`. The user may rename it later: name in `tauri.conf.json` (`productName`, `identifier`), `package.json`, `Cargo.toml`, README.
- **Next:** finish the toolchain install, then scaffold milestone (a).

## 2026-09-30 - Proposed architecture decisions (pending approval)
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

## 2026-09-30 - Research: reference project Coucou (read-only clone in `../reference/coucou`, commit 5ae7bd9)
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

## 2026-09-30 - Research: Tauri 2 window behavior on Windows 11
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

## 2026-09-30 - Research: what YouTube Music exposes through SMTC (verified by testing)
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

## 2026-09-30 - Research: calling SMTC from Rust (verified from docs)
- `windows` crate **0.62.2** is the latest published (GitHub master says 0.100.0, but that's not on crates.io yet). Tauri 2.12.0 depends on `windows ^0.62`.
- Cargo features: `Media_Control`, `Foundation`, `Foundation_Collections`, `Storage_Streams`.
- Manager: `GlobalSystemMediaTransportControlsSessionManager::RequestAsync()` → async op; `GetSessions()`, `GetCurrentSession()` ("the session the system believes the user would most likely want to control").
- Events, all returning an `i64` token for `Remove…(token)`: manager `SessionsChanged`, `CurrentSessionChanged`; session `MediaPropertiesChanged`, `PlaybackInfoChanged`, `TimelinePropertiesChanged`.
- Session reads: `SourceAppUserModelId`, `TryGetMediaPropertiesAsync`, `GetPlaybackInfo`, `GetTimelineProperties`. Controls: `TryPlayAsync`, `TryPauseAsync`, `TryTogglePlayPauseAsync`, `TrySkipNextAsync`, `TrySkipPreviousAsync` (each `IAsyncOperation<bool>`).
- Album art: `MediaProperties.Thumbnail()` → `IRandomAccessStreamReference` → `OpenReadAsync()` → read bytes (the `DataReader` path is **unverified** until milestone c).
- Waiting on async ops (windows-future 0.3): `.join()` blocks until done; `.await` also works (`IntoFuture`); `.when(callback)` runs a callback on completion.
- **Open question:** exact Rust type and units of `TimeSpan`/`DateTime` fields (WinRT uses 100 ns ticks; confirm in c).

## 2026-09-30 - Environment check (Windows 11 Home 26200)
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

# GitHub CLI (once: log in through the browser)
gh auth login
gh auth status

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
