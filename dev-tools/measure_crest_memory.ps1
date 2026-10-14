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
$crestProcesses = @(& "$PSScriptRoot/find_crest_processes.ps1")
if ($crestProcesses.Count -eq 0) { "Crest is not running"; exit 1 }

$crestProcessIds = $crestProcesses.ProcessId
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

$processMemoryRows = foreach ($crestProcessEntry in $crestProcesses) {
    $processSamples = $samplesByProcessId[$crestProcessEntry.ProcessId]
    if (-not $processSamples) { continue }
    [pscustomobject]@{
        Role                   = $crestProcessEntry.Role
        ProcessId              = $crestProcessEntry.ProcessId
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
