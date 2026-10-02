# Read-only: asks Windows about the running Crest pill window: both style words, position,
# DPI, its window region (the interactive area) and which app currently has focus.
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class CrestPillWindowInspector {
    public delegate bool EnumWindowsCallback(IntPtr windowHandle, IntPtr unused);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr unused);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr windowHandle, out uint processId);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr windowHandle);
    [DllImport("user32.dll")] public static extern IntPtr GetWindowLongPtrW(IntPtr windowHandle, int index);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr windowHandle, StringBuilder text, int maxCount);
    [StructLayout(LayoutKind.Sequential)] public struct NativeRect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr windowHandle, out NativeRect rect);
    [DllImport("user32.dll")] public static extern int GetWindowRgnBox(IntPtr windowHandle, out NativeRect rect);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr windowHandle);
}
"@
$crestProcess = Get-Process crest -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $crestProcess) { "Crest is not running"; exit }

# The pill is the top-level window titled "Crest"; tao also creates hidden helper windows.
$pillWindowHandle = [IntPtr]::Zero
$findPillWindow = [CrestPillWindowInspector+EnumWindowsCallback]{
    param($windowHandle, $unused)
    $ownerProcessId = 0
    [void][CrestPillWindowInspector]::GetWindowThreadProcessId($windowHandle, [ref]$ownerProcessId)
    $windowTitle = New-Object System.Text.StringBuilder 64
    [void][CrestPillWindowInspector]::GetWindowTextW($windowHandle, $windowTitle, 64)
    if ($ownerProcessId -eq $crestProcess.Id -and $windowTitle.ToString() -eq 'Crest') { $script:pillWindowHandle = $windowHandle; return $false }
    return $true
}
[void][CrestPillWindowInspector]::EnumWindows($findPillWindow, [IntPtr]::Zero)
if ($pillWindowHandle -eq [IntPtr]::Zero) { "No top-level 'Crest' window found"; exit }

$windowStyle = [CrestPillWindowInspector]::GetWindowLongPtrW($pillWindowHandle, -16).ToInt64()
$extendedWindowStyle = [CrestPillWindowInspector]::GetWindowLongPtrW($pillWindowHandle, -20).ToInt64()
$windowStyleFlags = [ordered]@{ WS_CAPTION = 0xC00000; WS_SYSMENU = 0x80000; WS_THICKFRAME = 0x40000; WS_POPUP = 0x80000000 }
$extendedWindowStyleFlags = [ordered]@{ WS_EX_TOPMOST = 0x8; WS_EX_TOOLWINDOW = 0x80; WS_EX_APPWINDOW = 0x40000; WS_EX_NOACTIVATE = 0x8000000; WS_EX_LAYERED = 0x80000; WS_EX_TRANSPARENT = 0x20 }

$windowRect = New-Object CrestPillWindowInspector+NativeRect
[void][CrestPillWindowInspector]::GetWindowRect($pillWindowHandle, [ref]$windowRect)
$regionBox = New-Object CrestPillWindowInspector+NativeRect
$regionKind = [CrestPillWindowInspector]::GetWindowRgnBox($pillWindowHandle, [ref]$regionBox)
$regionKindName = @{ 0 = 'none'; 1 = 'empty'; 2 = 'rectangle'; 3 = 'complex' }[$regionKind]
$windowDpi = [CrestPillWindowInspector]::GetDpiForWindow($pillWindowHandle)
$foregroundWindow = [CrestPillWindowInspector]::GetForegroundWindow()
$foregroundOwnerProcessId = 0
[void][CrestPillWindowInspector]::GetWindowThreadProcessId($foregroundWindow, [ref]$foregroundOwnerProcessId)

"Visible          : $([CrestPillWindowInspector]::IsWindowVisible($pillWindowHandle))"
"Window style     : 0x{0:X}" -f $windowStyle
foreach ($flagName in $windowStyleFlags.Keys) { "  {0,-18}: {1}" -f $flagName, (($windowStyle -band $windowStyleFlags[$flagName]) -ne 0) }
"Extended style   : 0x{0:X}" -f $extendedWindowStyle
foreach ($flagName in $extendedWindowStyleFlags.Keys) { "  {0,-18}: {1}" -f $flagName, (($extendedWindowStyle -band $extendedWindowStyleFlags[$flagName]) -ne 0) }
"Window rect (px) : left=$($windowRect.Left) top=$($windowRect.Top) width=$($windowRect.Right - $windowRect.Left) height=$($windowRect.Bottom - $windowRect.Top)"
"Window region    : $regionKindName  left=$($regionBox.Left) top=$($regionBox.Top) width=$($regionBox.Right - $regionBox.Left) height=$($regionBox.Bottom - $regionBox.Top) (relative to the window)"
"Window DPI       : $windowDpi (scale $([math]::Round($windowDpi / 96, 2)))"
"Foreground app   : $(if ($foregroundOwnerProcessId -eq $crestProcess.Id) { 'CREST (focus was stolen!)' } else { (Get-Process -Id $foregroundOwnerProcessId -ErrorAction SilentlyContinue).ProcessName })"
