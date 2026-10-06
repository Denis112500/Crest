# Writes the Claude Code hooks that send a test session's events to claude_code_hook_probe/.
# Only touches <SessionFolder>\.claude\settings.local.json, never the global
# ~/.claude/settings.json, so only sessions started in that folder run the hooks. Delete
# the folder to undo. Status events run in the background (they can't slow Claude Code
# down); PermissionRequest waits for the probe's answer, up to -PermissionRequestTimeoutSeconds.
#   pwsh dev-tools/write_hook_probe_session_settings.ps1 -SessionFolder <folder> -Transport pipe
param(
    [Parameter(Mandatory)] [string]$SessionFolder,
    [ValidateSet('pipe', 'http')] [string]$Transport = 'pipe',
    [int]$HttpPort = 47615,
    [int]$PermissionRequestTimeoutSeconds = 60
)

# Forward slashes: Claude Code runs command hooks through a shell, where backslashes can be escapes.
$probeExecutablePath = (Join-Path $PSScriptRoot 'claude_code_hook_probe/target/release/claude_code_hook_probe.exe') -replace '\\', '/'
if (-not (Test-Path $probeExecutablePath)) {
    "Build the probe first: cd dev-tools/claude_code_hook_probe; cargo build --release"
    exit 1
}

function New-ProbeHook([bool]$waitsForAnswer) {
    if ($Transport -eq 'pipe') {
        $probeHook = [ordered]@{ type = 'command'; command = "`"$probeExecutablePath`" forward" }
        if ($waitsForAnswer) { $probeHook.timeout = $PermissionRequestTimeoutSeconds } else { $probeHook.async = $true }
    } else {
        $probeHook = [ordered]@{ type = 'http'; url = "http://127.0.0.1:$HttpPort/hook" }
        $probeHook.timeout = if ($waitsForAnswer) { $PermissionRequestTimeoutSeconds } else { 5 }
    }
    $probeHook
}

$toolEventNames = 'PreToolUse', 'PostToolUse'
$otherStatusEventNames = 'SessionStart', 'UserPromptSubmit', 'Stop', 'Notification', 'SessionEnd'
$hooksByEventName = [ordered]@{}
foreach ($eventName in $otherStatusEventNames) {
    $hooksByEventName[$eventName] = @([ordered]@{ hooks = @(New-ProbeHook $false) })
}
foreach ($eventName in $toolEventNames) {
    $hooksByEventName[$eventName] = @([ordered]@{ matcher = '*'; hooks = @(New-ProbeHook $false) })
}
$hooksByEventName['PermissionRequest'] = @([ordered]@{ matcher = '*'; hooks = @(New-ProbeHook $true) })

$settingsFolder = Join-Path $SessionFolder '.claude'
New-Item -ItemType Directory -Force -Path $settingsFolder | Out-Null
$settingsPath = Join-Path $settingsFolder 'settings.local.json'
[ordered]@{ hooks = $hooksByEventName } | ConvertTo-Json -Depth 10 | Set-Content -Path $settingsPath -Encoding utf8NoBOM
"Wrote $settingsPath ($Transport). Start a new Claude Code session in $SessionFolder."
