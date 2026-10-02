# Records, for a fixed time, every change of the pill window's native style and of the page
# state (shell classes, layer opacities, album art, title), saving a screen capture of the
# pill per change. Use it to catch bugs that only show with the real mouse: run it, use the
# pill, then read the log and look at the captures. Needs the debugging port (see
# evaluate_in_pill_page.ps1).
param(
    [int]$RecordSeconds = 120,
    [string]$CaptureDirectory = "$env:TEMP\crest-pill-recording",
    [int]$SamplingPauseMilliseconds = 120,
    [string]$PillPageUrlPattern = 'http://localhost:1420*'
)
Remove-Item $CaptureDirectory -Recurse -ErrorAction SilentlyContinue
New-Item -ItemType Directory $CaptureDirectory | Out-Null
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class CrestPillRecorderNative {
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string className, string title);
    [DllImport("user32.dll")] public static extern IntPtr GetWindowLongPtrW(IntPtr windowHandle, int index);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr windowHandle);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [StructLayout(LayoutKind.Sequential)] public struct NativeRect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr windowHandle, out NativeRect rect);
}
"@

# Captured around the pill window with a margin, so anything drawn next to it shows too.
$captureMarginPixels = 50
function Save-PillCapture([IntPtr]$pillWindowHandle, [string]$captureName) {
    $windowRect = New-Object CrestPillRecorderNative+NativeRect
    [void][CrestPillRecorderNative]::GetWindowRect($pillWindowHandle, [ref]$windowRect)
    $captureLeft = [Math]::Max(0, $windowRect.Left - $captureMarginPixels)
    $captureTop = [Math]::Max(0, $windowRect.Top - $captureMarginPixels)
    $captureWidth = $windowRect.Right - $windowRect.Left + 2 * $captureMarginPixels
    $captureHeight = $windowRect.Bottom - $windowRect.Top + 2 * $captureMarginPixels
    $captureBitmap = New-Object System.Drawing.Bitmap $captureWidth, $captureHeight
    $captureGraphics = [System.Drawing.Graphics]::FromImage($captureBitmap)
    $captureGraphics.CopyFromScreen($captureLeft, $captureTop, 0, 0, $captureBitmap.Size)
    $captureBitmap.Save("$CaptureDirectory\$captureName.png")
    $captureGraphics.Dispose(); $captureBitmap.Dispose()
}

$pageStateExpression = "(() => { const shell = document.querySelector('.pill-shell'); const images = [...document.querySelectorAll('img')]; return shell.className + ' | compact=' + getComputedStyle(document.querySelector('.pill-compact-layer')).opacity + ' expanded=' + getComputedStyle(document.querySelector('.pill-expanded-layer')).opacity + ' | art=' + images.map((image) => image.naturalWidth + 'x' + image.naturalHeight + '/' + image.src.length).join(',') + ' | title=' + (shell.innerText.split('\n')[0] || '') })()"
$previousNativeState = ''; $previousPageState = ''; $captureCount = 0
$recordingDeadline = (Get-Date).AddSeconds($RecordSeconds)
"Recording for $RecordSeconds s into $CaptureDirectory"
while ((Get-Date) -lt $recordingDeadline) {
    $pillWindowHandle = [CrestPillRecorderNative]::FindWindowW([NullString]::Value, 'Crest')
    $windowStyle = [CrestPillRecorderNative]::GetWindowLongPtrW($pillWindowHandle, -16).ToInt64()
    $extendedWindowStyle = [CrestPillRecorderNative]::GetWindowLongPtrW($pillWindowHandle, -20).ToInt64()
    $nativeState = "style=0x{0:X} ex=0x{1:X} visible={2} pillHasFocus={3}" -f $windowStyle, $extendedWindowStyle, [CrestPillRecorderNative]::IsWindowVisible($pillWindowHandle), ([CrestPillRecorderNative]::GetForegroundWindow() -eq $pillWindowHandle)
    $pageState = & "$PSScriptRoot\evaluate_in_pill_page.ps1" -JavaScriptExpression $pageStateExpression -PillPageUrlPattern $PillPageUrlPattern
    if ($nativeState -ne $previousNativeState -or $pageState -ne $previousPageState) {
        $captureCount++
        $captureName = "{0:D3}" -f $captureCount
        Save-PillCapture $pillWindowHandle $captureName
        "{0} [{1}] {2} || {3}" -f (Get-Date -Format 'HH:mm:ss.fff'), $captureName, $nativeState, $pageState
        $previousNativeState = $nativeState; $previousPageState = $pageState
    }
    Start-Sleep -Milliseconds $SamplingPauseMilliseconds
}
"Recording finished: $captureCount changes"
