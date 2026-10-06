# Dev tools

Debugging helpers used while building Crest. None of them are part of the app or the
installer; all are read-only (they look, they don't change anything), except the hook probe pair,
which writes hooks into one test folder and can answer Claude Code's permission requests there.

| Tool | What it answers | Run with |
|---|---|---|
| `inspect_pill_window.ps1` | What does Windows think the pill window is? Both style words (title bar bits, tool window, no-activate…), position, DPI, window region (interactive area), which app has focus. | `pwsh` (PowerShell 7) |
| `evaluate_in_pill_page.ps1` | Runs one JavaScript expression inside the pill's page and prints the result. Needs the debugging port (below). | `pwsh` |
| `record_pill_session.ps1` | Logs every change of the window style and the page state for a while, with a screen capture of the pill per change. For bugs that only show with the real mouse: start it, use the pill, read the log. Needs the debugging port. | `pwsh` |
| `list_media_sessions.ps1` | Which media sessions does Windows see (app ID, title, playback state, supported buttons, timeline)? `-WatchSeconds 30` prints every change afterwards. | **`powershell.exe`** (5.1 only: it loads WinRT types) |
| `smtc_thumbnail_watcher/` | What album art does the player send, and when? Prints every title/thumbnail change and saves each distinct image. Found Brave's placeholder logo. | `cargo run --release` |
| `fullscreen_event_watcher/` | What does Windows report when a fullscreen app or game comes to the front or leaves? Registers as an appbar (`ABN_FULLSCREENAPP`, the taskbar's own signal) and watches front-window changes; each event logs the front window's class/title, its monitor, whether it covers that monitor, and `SHQueryUserNotificationState`. | `cargo run --release` |
| `claude_code_hook_probe/` | Can Claude Code's hooks reach Crest locally, and can an Allow/Deny answer get back? `listen` stands in for Crest: prints every hook event arriving over a named pipe (only the current Windows user may open it) or `http://127.0.0.1:47615/hook`, and answers PermissionRequest as told (`--permission-answer none\|allow\|deny`, `--answer-delay-seconds`). `forward` is what a command hook runs. | `cargo run --release -- listen` |
| `write_hook_probe_session_settings.ps1` | Writes the hooks for the probe into one test folder's `.claude/settings.local.json` (never the global settings); `-Transport pipe` or `http`. Delete the folder to undo. | `pwsh` |
| `measure_crest_memory.ps1` | How much memory do Crest and its WebView2 processes use? Per process (role from the command line) and in total: private working set (Task Manager's "Memory"), private bytes, working set, averaged over samples. Measure release builds, and check which `crest.exe` is running. | `pwsh` |
| `measure_pill_reveal_delay.ps1` | How long does the pill take to appear after music starts? Samples every 20 ms whether a player reports Playing and whether the pill window is visible; prints each change and the delay, and stops after the first measured appearance. Press play while the pill is hidden. | **`powershell.exe`** (5.1 only: it loads WinRT types) |
| `record_readme_demo.py` | Records the README's demo: a GIF and a still of the open pill, capturing only a rectangle around the pill window (unchanged frames merged). Put a calm window behind the pill first; the still is the frame with the most visible text. | `python` (needs Pillow) |

## Debugging port

The page tools talk to WebView2 through the Chrome DevTools Protocol. Start Crest with the
port open (local only; don't leave it on in daily use):

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9223"; npm run tauri dev
```

The dev page is `http://localhost:1420`; for a release build pass
`-PillPageUrlPattern 'http://tauri.localhost*'`.

## Examples

```powershell
pwsh dev-tools/inspect_pill_window.ps1
pwsh dev-tools/measure_crest_memory.ps1 -SampleCount 10 -SampleIntervalSeconds 2
powershell.exe -ExecutionPolicy Bypass -File dev-tools/measure_pill_reveal_delay.ps1 -WatchSeconds 600
python dev-tools/record_readme_demo.py --seconds 16 --output-folder docs
pwsh dev-tools/evaluate_in_pill_page.ps1 -JavaScriptExpression "document.querySelector('.pill-shell').className"
pwsh dev-tools/record_pill_session.ps1 -RecordSeconds 120
powershell.exe -ExecutionPolicy Bypass -File dev-tools/list_media_sessions.ps1 -WatchSeconds 30
cd dev-tools/smtc_thumbnail_watcher; cargo run --release -- 300 $env:TEMP
pwsh dev-tools/write_hook_probe_session_settings.ps1 -SessionFolder ..\crest-hook-probe-session -Transport pipe
cd dev-tools/claude_code_hook_probe; cargo run --release -- listen --permission-answer allow --answer-delay-seconds 5
cd dev-tools/fullscreen_event_watcher; cargo run --release -- 600 $env:TEMP\crest_fullscreen_events.log
```
