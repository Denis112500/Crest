# Read-only: the running crest.exe and every WebView2 process it started (browser, renderer, GPU,
# helpers), each with its role read from its command line. Used by the measuring scripts, so they
# all count the same processes. Returns nothing if Crest isn't running.
#   $crestProcesses = & "$PSScriptRoot/find_crest_processes.ps1"

$crestProcess = Get-CimInstance Win32_Process -Filter "Name='crest.exe'" | Select-Object -First 1
if (-not $crestProcess) { return }

# Crest starts the WebView2 browser process, which starts the renderer, GPU and helper processes.
$allProcesses = Get-CimInstance Win32_Process
$crestProcessTree = [System.Collections.Generic.List[object]]::new()
$processesToVisit = [System.Collections.Generic.Queue[object]]::new()
$processesToVisit.Enqueue($crestProcess)
while ($processesToVisit.Count -gt 0) {
    $visitedProcess = $processesToVisit.Dequeue()
    $crestProcessTree.Add($visitedProcess)
    $allProcesses | Where-Object { $_.ParentProcessId -eq $visitedProcess.ProcessId } | ForEach-Object { $processesToVisit.Enqueue($_) }
}

function Get-CrestProcessRole($processDescription) {
    $commandLine = "$($processDescription.CommandLine)"
    if ($processDescription.Name -eq 'crest.exe') { return 'Crest (Rust)' }
    if ($commandLine -match '--type=renderer') { return 'WebView2 renderer (page, JS)' }
    if ($commandLine -match '--type=gpu-process') { return 'WebView2 GPU (drawing)' }
    if ($commandLine -match '--type=crashpad-handler') { return 'WebView2 crash reporter' }
    if ($commandLine -match '--utility-sub-type=([\w.]+)') { return "WebView2 utility ($($Matches[1] -replace '^.*\.', ''))" }
    if ($commandLine -match '--type=(\S+)') { return "WebView2 $($Matches[1])" }
    return 'WebView2 browser (manager)'
}

foreach ($processDescription in $crestProcessTree) {
    [pscustomobject]@{
        ProcessId      = $processDescription.ProcessId
        Role           = Get-CrestProcessRole $processDescription
        ExecutablePath = $processDescription.ExecutablePath
    }
}
