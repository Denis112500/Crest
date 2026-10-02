# Dev tools

Debugging helpers used while building Crest. None of them are part of the app or the
installer; all are read-only (they look, they don't change anything).

| Tool | What it answers | Run with |
|---|---|---|
| `inspect_pill_window.ps1` | What does Windows think the pill window is? Both style words (title bar bits, tool window, no-activate…), position, DPI, window region (interactive area), which app has focus. | `pwsh` (PowerShell 7) |
| `evaluate_in_pill_page.ps1` | Runs one JavaScript expression inside the pill's page and prints the result. Needs the debugging port (below). | `pwsh` |
| `record_pill_session.ps1` | Logs every change of the window style and the page state for a while, with a screen capture of the pill per change. For bugs that only show with the real mouse: start it, use the pill, read the log. Needs the debugging port. | `pwsh` |
| `list_media_sessions.ps1` | Which media sessions does Windows see (app ID, title, playback state, supported buttons, timeline)? `-WatchSeconds 30` prints every change afterwards. | **`powershell.exe`** (5.1 only: it loads WinRT types) |
| `smtc_thumbnail_watcher/` | What album art does the player send, and when? Prints every title/thumbnail change and saves each distinct image. Found Brave's placeholder logo. | `cargo run --release` |

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
pwsh dev-tools/evaluate_in_pill_page.ps1 -JavaScriptExpression "document.querySelector('.pill-shell').className"
pwsh dev-tools/record_pill_session.ps1 -RecordSeconds 120
powershell.exe -ExecutionPolicy Bypass -File dev-tools/list_media_sessions.ps1 -WatchSeconds 30
cd dev-tools/smtc_thumbnail_watcher; cargo run --release -- 300 $env:TEMP
```
