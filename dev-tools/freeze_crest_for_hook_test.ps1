# Not read-only: pauses the running Crest (only crest.exe itself, not its WebView2 processes) and
# lets it go again, to see what Claude Code's hooks do when Crest is running but never answers.
# The pipe stays open while paused, so a hook connects, sends its event and waits for a reply.
# Close the pill before freezing: a paused window doesn't redraw and Windows may mark it "Not
# responding". Always end with -Action Resume; ListHookCopies also says whether Crest is frozen.
#   pwsh dev-tools/freeze_crest_for_hook_test.ps1 -Action Freeze
#   pwsh dev-tools/freeze_crest_for_hook_test.ps1 -Action ListHookCopies
#   pwsh dev-tools/freeze_crest_for_hook_test.ps1 -Action Resume
param(
    [Parameter(Mandatory)]
    [ValidateSet('Freeze', 'Resume', 'ListHookCopies')]
    [string]$Action
)

# Claude Code starts `crest.exe --claude-code-hook` for every hook event; those copies are not Crest.
$hookCopyArgument = '--claude-code-hook'
$crestExecutableProcesses = @(Get-CimInstance Win32_Process -Filter "Name='crest.exe'")
$hookCopyProcesses = @($crestExecutableProcesses | Where-Object { "$($_.CommandLine)" -match [regex]::Escape($hookCopyArgument) })
$runningCrestProcesses = @($crestExecutableProcesses | Where-Object { "$($_.CommandLine)" -notmatch [regex]::Escape($hookCopyArgument) })

function Get-SuspendedThreadCount([int]$processId) {
    @((Get-Process -Id $processId).Threads | Where-Object { $_.ThreadState -eq 'Wait' -and $_.WaitReason -eq 'Suspended' }).Count
}

if ($Action -eq 'ListHookCopies') {
    $runningCrestProcesses | ForEach-Object {
        $crestState = if ((Get-SuspendedThreadCount $_.ProcessId) -gt 0) { 'frozen' } else { 'running' }
        "Crest, process $($_.ProcessId): $crestState"
    }
    "$($hookCopyProcesses.Count) hook copies of crest.exe running"
    $hookCopyProcesses | ForEach-Object {
        $runningSeconds = ((Get-Date) - $_.CreationDate).TotalSeconds
        [pscustomobject]@{
            ProcessId      = $_.ProcessId
            RunningSeconds = [math]::Round($runningSeconds, 1)
            ExecutablePath = $_.ExecutablePath
        }
    } | Sort-Object RunningSeconds -Descending | Format-Table -AutoSize
    exit 0
}

if ($runningCrestProcesses.Count -ne 1) { "Expected one running Crest, found $($runningCrestProcesses.Count)"; exit 1 }
$crestProcess = $runningCrestProcesses[0]

# NtSuspendProcess / NtResumeProcess pause and continue every thread of a process at once; they
# are what Task Manager's and Resource Monitor's "Suspend process" use. Not documented, but stable.
Add-Type -Namespace CrestHookTest -Name ProcessFreezing -MemberDefinition @'
[DllImport("ntdll.dll")] public static extern int NtSuspendProcess(IntPtr processHandle);
[DllImport("ntdll.dll")] public static extern int NtResumeProcess(IntPtr processHandle);
'@

$crestProcessHandle = (Get-Process -Id $crestProcess.ProcessId).Handle
# Windows counts suspensions: a second Freeze needs a second Resume, which is easy to miss, so
# Freeze refuses a frozen Crest and Resume repeats until no thread is left suspended.
$maximumResumeCalls = 20
if ($Action -eq 'Freeze') {
    if ((Get-SuspendedThreadCount $crestProcess.ProcessId) -gt 0) { "Crest is already frozen; run -Action Resume"; exit 1 }
    $ntStatus = [CrestHookTest.ProcessFreezing]::NtSuspendProcess($crestProcessHandle)
    if ($ntStatus -ne 0) { "Freeze failed: NTSTATUS 0x{0:X8}" -f $ntStatus; exit 1 }
} else {
    for ($resumeCall = 0; $resumeCall -lt $maximumResumeCalls -and (Get-SuspendedThreadCount $crestProcess.ProcessId) -gt 0; $resumeCall++) {
        $ntStatus = [CrestHookTest.ProcessFreezing]::NtResumeProcess($crestProcessHandle)
        if ($ntStatus -ne 0) { "Resume failed: NTSTATUS 0x{0:X8}" -f $ntStatus; exit 1 }
    }
    if ((Get-SuspendedThreadCount $crestProcess.ProcessId) -gt 0) { "Crest is still frozen after $maximumResumeCalls resumes"; exit 1 }
}
"{0}: Crest, process {1} ({2})" -f $(if ($Action -eq 'Freeze') { 'Frozen' } else { 'Running again' }), $crestProcess.ProcessId, $crestProcess.ExecutablePath
