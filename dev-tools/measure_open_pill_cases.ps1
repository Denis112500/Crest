# Measures the open pill's CPU in several look cases, back to back, in one running release build.
# Each case is a style sheet put into the live page through the debugging port (no rebuild), so
# every case runs the same exe minutes apart. Checks before and after each window that the pill
# is still open and the song still playing; a case where it wasn't is marked INVALID.
# Crest (a release build) must run with the debugging port (dev-tools/README.md). Keep the
# pointer resting on the open pill's title for the whole run (about 5 minutes with the default
# cases). The page is put back as built at the end.
#   pwsh dev-tools/measure_open_pill_cases.ps1
#   pwsh dev-tools/measure_open_pill_cases.ps1 -CaseOrder asBuilt,asBuilt   (one build, to compare with another)
param(
    [int]$WindowSeconds = 60,
    [int]$SettleSeconds = 5,
    [string[]]$CaseOrder = @('asBuilt', 'ringNotDrawn', 'ringWithoutGlow', 'asBuilt'),
    [int]$DebuggingPort = 9223,
    [string]$PillPageUrlPattern = 'http://tauri.localhost*'
)

$styleSheetByCase = @{
    asBuilt          = ''
    # The open ring isn't drawn, so its 4 redraws a second paint nothing: only the time's change
    # once a second makes a frame. Stands in for "redraw the ring once a second".
    ringNotDrawn     = '.expanded-music-view .cover-progress-ring { display: none !important; }'
    # The ring as now, without its drop-shadow glow (the first suspect).
    ringWithoutGlow  = '.cover-progress-ring { filter: none !important; }'
    # The ring on a compositor layer of its own.
    ringOwnLayer     = '.expanded-music-view .cover-progress-gauge svg { will-change: transform; }'
}

$evaluateScript = Join-Path $PSScriptRoot 'evaluate_in_pill_page.ps1'
$measureScript = Join-Path $PSScriptRoot 'measure_crest_cpu.ps1'
function Invoke-InPillPage([string]$javaScriptExpression) {
    & $evaluateScript -JavaScriptExpression $javaScriptExpression -DebuggingPort $DebuggingPort -PillPageUrlPattern $PillPageUrlPattern | Out-String
}

# The release page's CSP blocks <style> elements; a constructed style sheet is allowed.
function Set-CaseStyleSheet([string]$styleSheetText) {
    $escapedStyleSheetText = $styleSheetText.Replace('\', '\\').Replace('"', '\"')
    Invoke-InPillPage "(() => { const sheet = window.cpuCaseStyleSheet ??= new CSSStyleSheet(); document.adoptedStyleSheets = [sheet]; sheet.replaceSync(""$escapedStyleSheetText""); return 'set' })()" | Out-Null
}

function Get-OpenAndPlayingState {
    Invoke-InPillPage "(() => { const openRing = document.querySelector('.expanded-music-view .cover-progress-ring'); return { isOpen: !!document.querySelector('.pill-shell.is-expanded'), isPlaying: !!document.querySelector('.expanded-music-view .cover-progress-gauge:not(.is-paused)'), time: document.querySelector('.expanded-music-time')?.textContent, ringDisplay: openRing && getComputedStyle(openRing).display, ringFilter: openRing && getComputedStyle(openRing).filter } })()" | ConvertFrom-Json
}

$crestExe = Get-CimInstance Win32_Process -Filter "Name='crest.exe'" | Select-Object -First 1
if (-not $crestExe) { "Crest is not running"; exit 1 }
"Crest: $($crestExe.ExecutablePath) (PID $($crestExe.ProcessId)), exe written $((Get-Item $crestExe.ExecutablePath).LastWriteTime)"
"Cases in order: $($CaseOrder -join ', '); $WindowSeconds s each. Keep the pointer on the open pill's title."
""

foreach ($caseName in $CaseOrder) {
    Set-CaseStyleSheet $styleSheetByCase[$caseName]
    Start-Sleep -Seconds $SettleSeconds
    $stateBefore = Get-OpenAndPlayingState
    "=== $caseName  (start: open=$($stateBefore.isOpen) playing=$($stateBefore.isPlaying) time=$($stateBefore.time) ring display=$($stateBefore.ringDisplay) filter=$($stateBefore.ringFilter))"
    & $measureScript -WindowSeconds $WindowSeconds
    $stateAfter = Get-OpenAndPlayingState
    $isCaseValid = $stateBefore.isOpen -and $stateBefore.isPlaying -and $stateAfter.isOpen -and $stateAfter.isPlaying
    "    (end: open=$($stateAfter.isOpen) playing=$($stateAfter.isPlaying) time=$($stateAfter.time))  $(if ($isCaseValid) { 'valid' } else { 'INVALID: pill closed or song paused at one end' })"
    ""
}
Set-CaseStyleSheet ''
"Done; the page is back to as built."
