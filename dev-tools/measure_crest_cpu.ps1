# Read-only: how much CPU Crest and its WebView2 processes used over a window of time, per process
# and in total, as a percentage of one CPU core (100 % = one core busy the whole time). Reads each
# process's total CPU time at the start and at the end, so short bursts (a sweep of light, a ring
# redraw) are counted, not missed between samples. Windows charges CPU time in clock ticks of
# about 15.6 ms, so very small work can still show as 0.
#   pwsh dev-tools/measure_crest_cpu.ps1 -WindowSeconds 60
param([int]$WindowSeconds = 60)

$crestProcesses = @(& "$PSScriptRoot/find_crest_processes.ps1")
if ($crestProcesses.Count -eq 0) { "Crest is not running"; exit 1 }

function Get-ProcessorSeconds([int]$processId) {
    $runningProcess = Get-Process -Id $processId -ErrorAction SilentlyContinue
    if ($runningProcess) { return $runningProcess.TotalProcessorTime.TotalSeconds }
    return $null
}

$startSecondsByProcessId = @{}
foreach ($crestProcessEntry in $crestProcesses) {
    $startSecondsByProcessId[$crestProcessEntry.ProcessId] = Get-ProcessorSeconds $crestProcessEntry.ProcessId
}
$windowStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
Start-Sleep -Seconds $WindowSeconds
$elapsedSeconds = $windowStopwatch.Elapsed.TotalSeconds

$processCpuRows = foreach ($crestProcessEntry in $crestProcesses) {
    $startSeconds = $startSecondsByProcessId[$crestProcessEntry.ProcessId]
    $endSeconds = Get-ProcessorSeconds $crestProcessEntry.ProcessId
    # A process that ended or started during the window can't be compared; it's left out.
    if ($null -eq $startSeconds -or $null -eq $endSeconds) { continue }
    [pscustomobject]@{
        Role             = $crestProcessEntry.Role
        ProcessId        = $crestProcessEntry.ProcessId
        CpuSeconds       = [math]::Round($endSeconds - $startSeconds, 3)
        PercentOfOneCore = [math]::Round(100 * ($endSeconds - $startSeconds) / $elapsedSeconds, 3)
    }
}
$processCpuRows | Sort-Object PercentOfOneCore -Descending | Format-Table -AutoSize
"Total: {0} % of one core ({1} CPU seconds in {2} s, {3} processes)" -f `
    [math]::Round(($processCpuRows | Measure-Object PercentOfOneCore -Sum).Sum, 3),
    [math]::Round(($processCpuRows | Measure-Object CpuSeconds -Sum).Sum, 3),
    [math]::Round($elapsedSeconds, 1),
    @($processCpuRows).Count
