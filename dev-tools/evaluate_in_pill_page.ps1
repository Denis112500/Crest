# Runs one JavaScript expression inside the pill's page and prints the result, through the
# Chrome DevTools Protocol. Crest must be started with the debugging port open:
#   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9223"; npm run tauri dev
# The dev page is served by Vite (localhost:1420); a release build serves it as tauri.localhost.
param(
    [Parameter(Mandatory)] [string]$JavaScriptExpression,
    [int]$DebuggingPort = 9223,
    [string]$PillPageUrlPattern = 'http://localhost:1420*'
)

$debugTargets = Invoke-RestMethod "http://127.0.0.1:$DebuggingPort/json"
$pillPageTarget = $debugTargets | Where-Object { $_.type -eq 'page' -and $_.url -like $PillPageUrlPattern } | Select-Object -First 1
if (-not $pillPageTarget) { throw "No page matching '$PillPageUrlPattern' among the debug targets" }

$webSocket = [System.Net.WebSockets.ClientWebSocket]::new()
$webSocket.ConnectAsync([Uri]$pillPageTarget.webSocketDebuggerUrl, [Threading.CancellationToken]::None).Wait()
$requestJson = @{ id = 1; method = 'Runtime.evaluate'; params = @{ expression = $JavaScriptExpression; returnByValue = $true; awaitPromise = $true } } | ConvertTo-Json -Depth 5 -Compress
$requestBytes = [Text.Encoding]::UTF8.GetBytes($requestJson)
$webSocket.SendAsync([ArraySegment[byte]]::new($requestBytes), 'Text', $true, [Threading.CancellationToken]::None).Wait()

# A long result arrives in several frames; read until the message is complete.
$responseText = ''
$receiveBuffer = [byte[]]::new(65536)
do {
    $receiveResult = $webSocket.ReceiveAsync([ArraySegment[byte]]::new($receiveBuffer), [Threading.CancellationToken]::None).Result
    $responseText += [Text.Encoding]::UTF8.GetString($receiveBuffer, 0, $receiveResult.Count)
} until ($receiveResult.EndOfMessage)
$webSocket.Dispose()

$response = $responseText | ConvertFrom-Json
if ($response.result.exceptionDetails) {
    "JS error: $($response.result.exceptionDetails.exception.description)"
} else {
    $response.result.result.value | ConvertTo-Json -Depth 5
}
