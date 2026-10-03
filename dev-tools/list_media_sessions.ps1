# Read-only: lists every media session Windows knows about (what Crest sees through SMTC):
# app ID, title, artist, playback state, supported buttons and timeline. With -WatchSeconds
# it then prints every change of the current session (including its supported buttons).
# Must run under Windows PowerShell 5.1 (powershell.exe, not pwsh), which can load WinRT types:
#   powershell.exe -ExecutionPolicy Bypass -File dev-tools\list_media_sessions.ps1 -WatchSeconds 30
param([int]$WatchSeconds = 0, [int]$WatchSamplingMilliseconds = 100)

Add-Type -AssemblyName System.Runtime.WindowsRuntime
$asTaskMethod = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {
    $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
} | Select-Object -First 1
function Format-SupportedButtons($playbackControls) {
    "play=$($playbackControls.IsPlayEnabled) pause=$($playbackControls.IsPauseEnabled) toggle=$($playbackControls.IsPlayPauseToggleEnabled) next=$($playbackControls.IsNextEnabled) previous=$($playbackControls.IsPreviousEnabled)"
}
function Wait-WinRtOperation($winRtOperation, [Type]$resultType) {
    $operationTask = $asTaskMethod.MakeGenericMethod($resultType).Invoke($null, @($winRtOperation))
    $operationTask.Wait(-1) | Out-Null
    $operationTask.Result
}

$sessionManagerType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
$mediaPropertiesType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties, Windows.Media.Control, ContentType = WindowsRuntime]
$sessionManager = Wait-WinRtOperation ($sessionManagerType::RequestAsync()) $sessionManagerType
$mediaSessions = @($sessionManager.GetSessions())
$currentSession = $sessionManager.GetCurrentSession()
"Sessions found      : $($mediaSessions.Count)"
"Current session app : $(if ($currentSession) { $currentSession.SourceAppUserModelId } else { '<none>' })"

$sessionNumber = 0
foreach ($mediaSession in $mediaSessions) {
    $sessionNumber++
    "`n===== Session $sessionNumber : $($mediaSession.SourceAppUserModelId) ====="
    $mediaProperties = Wait-WinRtOperation ($mediaSession.TryGetMediaPropertiesAsync()) $mediaPropertiesType
    "Title         : '$($mediaProperties.Title)'"
    "Artist        : '$($mediaProperties.Artist)'"
    "Album         : '$($mediaProperties.AlbumTitle)'"
    "Thumbnail     : $(if ($mediaProperties.Thumbnail) { 'present' } else { 'missing' })"
    $playbackInfo = $mediaSession.GetPlaybackInfo()
    "Playback      : $($playbackInfo.PlaybackStatus)"
    $playbackControls = $playbackInfo.Controls
    "Buttons       : $(Format-SupportedButtons $playbackControls)"
    $timeline = $mediaSession.GetTimelineProperties()
    "Timeline      : start=$($timeline.StartTime) end=$($timeline.EndTime) position=$($timeline.Position) updated=$($timeline.LastUpdatedTime.ToLocalTime().ToString('HH:mm:ss.fff'))"
}

if ($WatchSeconds -gt 0) {
    "`n===== Watching the current session for $WatchSeconds s (only changes are printed) ====="
    $previousLine = ''
    $watchDeadline = (Get-Date).AddSeconds($WatchSeconds)
    while ((Get-Date) -lt $watchDeadline) {
        $currentSession = $sessionManager.GetCurrentSession()
        if ($currentSession) {
            $mediaProperties = Wait-WinRtOperation ($currentSession.TryGetMediaPropertiesAsync()) $mediaPropertiesType
            $timeline = $currentSession.GetTimelineProperties()
            $currentPlaybackInfo = $currentSession.GetPlaybackInfo()
            $currentLine = "$($currentSession.SourceAppUserModelId) | $($currentPlaybackInfo.PlaybackStatus) | $(Format-SupportedButtons $currentPlaybackInfo.Controls) | '$($mediaProperties.Title)' | position=$($timeline.Position)"
        } else {
            $currentLine = '<no current session>'
        }
        if ($currentLine -ne $previousLine) { "$((Get-Date).ToString('HH:mm:ss.fff'))  $currentLine"; $previousLine = $currentLine }
        Start-Sleep -Milliseconds $WatchSamplingMilliseconds
    }
}
