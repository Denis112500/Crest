# Read-only: how long the pill takes to appear after music starts. Samples every
# -SamplingMilliseconds whether any media session reports Playing (what Crest hears through SMTC)
# and whether Crest's pill window is visible, prints each change with a timestamp, and the delay
# from "playing" to "visible". Start it, then press play while the pill is hidden; it stops after
# the first measured appearance (or after -WatchSeconds).
# Must run under Windows PowerShell 5.1 (powershell.exe, not pwsh), which can load WinRT types:
#   powershell.exe -ExecutionPolicy Bypass -File dev-tools\measure_pill_reveal_delay.ps1 -WatchSeconds 60
param([int]$WatchSeconds = 60, [int]$SamplingMilliseconds = 20)

Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class CrestPillRevealProbe {
    public delegate bool EnumWindowsCallback(IntPtr windowHandle, IntPtr unused);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr unused);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr windowHandle, out uint processId);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr windowHandle);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr windowHandle, StringBuilder text, int maxCount);
}
"@
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$asTaskMethod = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {
    $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
} | Select-Object -First 1

$crestProcess = Get-Process crest -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $crestProcess) { "Crest is not running"; exit 1 }
# The pill is the top-level window titled "Crest"; the settings window is "Crest Settings".
$pillWindowHandle = [IntPtr]::Zero
$findPillWindow = [CrestPillRevealProbe+EnumWindowsCallback]{
    param($windowHandle, $unused)
    $ownerProcessId = 0
    [void][CrestPillRevealProbe]::GetWindowThreadProcessId($windowHandle, [ref]$ownerProcessId)
    $windowTitle = New-Object System.Text.StringBuilder 64
    [void][CrestPillRevealProbe]::GetWindowTextW($windowHandle, $windowTitle, 64)
    if ($ownerProcessId -eq $crestProcess.Id -and $windowTitle.ToString() -eq 'Crest') { $script:pillWindowHandle = $windowHandle; return $false }
    return $true
}
[void][CrestPillRevealProbe]::EnumWindows($findPillWindow, [IntPtr]::Zero)
if ($pillWindowHandle -eq [IntPtr]::Zero) { "No top-level 'Crest' window found"; exit 1 }

$sessionManagerType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
$sessionManagerTask = $asTaskMethod.MakeGenericMethod($sessionManagerType).Invoke($null, @($sessionManagerType::RequestAsync()))
$sessionManager = $sessionManagerTask.Result

function Test-AnySessionPlaying {
    foreach ($mediaSession in $sessionManager.GetSessions()) {
        if ($mediaSession.GetPlaybackInfo().PlaybackStatus -eq 'Playing') { return $true }
    }
    return $false
}

"Crest pid $($crestProcess.Id) ($($crestProcess.Path)), started $($crestProcess.StartTime.ToString('HH:mm:ss'))"
"Watching for $WatchSeconds s, sampling every $SamplingMilliseconds ms. Press play while the pill is hidden."
$wasPlaying = Test-AnySessionPlaying
$wasPillVisible = [CrestPillRevealProbe]::IsWindowVisible($pillWindowHandle)
"{0}  start: playing={1} pill visible={2}" -f (Get-Date -Format 'HH:mm:ss.fff'), $wasPlaying, $wasPillVisible
$playingSince = $null
$watchStopwatch = [Diagnostics.Stopwatch]::StartNew()
while ($watchStopwatch.Elapsed.TotalSeconds -lt $WatchSeconds) {
    $isPlaying = Test-AnySessionPlaying
    $isPillVisible = [CrestPillRevealProbe]::IsWindowVisible($pillWindowHandle)
    $sampleTime = Get-Date
    if ($isPlaying -ne $wasPlaying) {
        "{0}  music {1}" -f $sampleTime.ToString('HH:mm:ss.fff'), $(if ($isPlaying) { 'started playing' } else { 'stopped playing' })
        $playingSince = if ($isPlaying -and -not $isPillVisible) { $sampleTime } else { $null }
    }
    if ($isPillVisible -ne $wasPillVisible) {
        "{0}  pill window {1}" -f $sampleTime.ToString('HH:mm:ss.fff'), $(if ($isPillVisible) { 'shown' } else { 'hidden' })
        if ($isPillVisible -and $playingSince) {
            "           -> appeared {0:N0} ms after the music started (+/- {1} ms sampling)" -f ($sampleTime - $playingSince).TotalMilliseconds, $SamplingMilliseconds
            break
        }
    }
    $wasPlaying = $isPlaying
    $wasPillVisible = $isPillVisible
    Start-Sleep -Milliseconds $SamplingMilliseconds
}
