# Read-only: memory of Crest and every WebView2 process it started, per process and in total,
# averaged over a few samples (memory moves a little from second to second).
# Three numbers per process:
#   private working set = Task Manager's "Memory" column: RAM that only this process uses;
#   working set         = all RAM the process touches, including pages shared with other programs
#                         (the WebView2/Edge libraries are loaded once and shared), so summing it
#                         over processes counts shared pages several times;
#   private bytes       = everything the process reserved for itself, including paged-out parts.
#   pwsh dev-tools/measure_crest_memory.ps1 -SampleCount 5 -SampleIntervalSeconds 2
param([int]$SampleCount = 5, [int]$SampleIntervalSeconds = 2)

$bytesPerMegabyte = 1MB
$crestProcess = Get-CimInstance Win32_Process -Filter "Name='crest.exe'" | Select-Object -First 1
if (-not $crestProcess) { "Crest is not running"; exit 1 }

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

function Get-ProcessRole($processDescription) {
    $commandLine = "$($processDescription.CommandLine)"
    if ($processDescription.Name -eq 'crest.exe') { return 'Crest (Rust)' }
    if ($commandLine -match '--type=renderer') { return 'WebView2 renderer (page, JS)' }
    if ($commandLine -match '--type=gpu-process') { return 'WebView2 GPU (drawing)' }
    if ($commandLine -match '--type=crashpad-handler') { return 'WebView2 crash reporter' }
    if ($commandLine -match '--utility-sub-type=([\w.]+)') { return "WebView2 utility ($($Matches[1] -replace '^.*\.', ''))" }
    if ($commandLine -match '--type=(\S+)') { return "WebView2 $($Matches[1])" }
    return 'WebView2 browser (manager)'
}

$crestProcessIds = $crestProcessTree.ProcessId
$samplesByProcessId = @{}
for ($sampleIndex = 0; $sampleIndex -lt $SampleCount; $sampleIndex++) {
    Get-CimInstance Win32_PerfFormattedData_PerfProc_Process |
        Where-Object { $crestProcessIds -contains $_.IDProcess } |
        ForEach-Object {
            if (-not $samplesByProcessId.ContainsKey($_.IDProcess)) { $samplesByProcessId[$_.IDProcess] = @() }
            $samplesByProcessId[$_.IDProcess] += , @($_.WorkingSetPrivate, $_.PrivateBytes, $_.WorkingSet)
        }
    if ($sampleIndex -lt $SampleCount - 1) { Start-Sleep -Seconds $SampleIntervalSeconds }
}

$processMemoryRows = foreach ($processDescription in $crestProcessTree) {
    $processSamples = $samplesByProcessId[$processDescription.ProcessId]
    if (-not $processSamples) { continue }
    [pscustomobject]@{
        Role                   = Get-ProcessRole $processDescription
        ProcessId              = $processDescription.ProcessId
        PrivateWorkingSetMB    = [math]::Round(($processSamples | ForEach-Object { $_[0] } | Measure-Object -Average).Average / $bytesPerMegabyte, 1)
        PrivateBytesMB         = [math]::Round(($processSamples | ForEach-Object { $_[1] } | Measure-Object -Average).Average / $bytesPerMegabyte, 1)
        WorkingSetMB           = [math]::Round(($processSamples | ForEach-Object { $_[2] } | Measure-Object -Average).Average / $bytesPerMegabyte, 1)
    }
}
$processMemoryRows | Sort-Object PrivateWorkingSetMB -Descending | Format-Table -AutoSize
"Total private working set: {0} MB   Total private bytes: {1} MB   Sum of working sets: {2} MB   ({3} processes, {4} samples)" -f `
    [math]::Round(($processMemoryRows | Measure-Object PrivateWorkingSetMB -Sum).Sum, 1),
    [math]::Round(($processMemoryRows | Measure-Object PrivateBytesMB -Sum).Sum, 1),
    [math]::Round(($processMemoryRows | Measure-Object WorkingSetMB -Sum).Sum, 1),
    @($processMemoryRows).Count, $SampleCount
