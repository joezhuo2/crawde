param([string]$ConfigDirectory = (Join-Path $env:APPDATA 'com.joezhuo.crawde'))
$ErrorActionPreference = 'Stop'
$crawdeToken = Get-Content -LiteralPath (Join-Path $ConfigDirectory 'bridge-token') -Raw
$crawdeHeaders = @{ Authorization = "Bearer $crawdeToken" }
$crawdeEndpoint = 'http://127.0.0.1:47631/events'

function Send-CrawdeEvent([string]$EventName, [hashtable]$Extra = @{}) {
    $crawdePayload = @{ session_id = 'crawde-smoke'; hook_event_name = $EventName; cwd = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path }
    foreach ($crawdeKey in $Extra.Keys) { $crawdePayload[$crawdeKey] = $Extra[$crawdeKey] }
    Invoke-RestMethod -Method Post -Uri $crawdeEndpoint -Headers $crawdeHeaders -ContentType 'application/json' -Body ($crawdePayload | ConvertTo-Json -Depth 8 -Compress) | Out-Null
}

function Assert-CrawdeRejected([hashtable]$Headers, [int]$Expected) {
    try {
        Invoke-RestMethod -Method Post -Uri $crawdeEndpoint -Headers $Headers -ContentType 'application/json' -Body '{}' | Out-Null
        throw 'The bridge accepted a request that should have been rejected.'
    } catch {
        if (-not $_.Exception.Response -or [int]$_.Exception.Response.StatusCode -ne $Expected) { throw }
    }
}

Assert-CrawdeRejected @{} 403
Assert-CrawdeRejected @{ Authorization = "Bearer $crawdeToken"; Origin = 'https://example.com' } 403
try {
    Send-CrawdeEvent 'SessionStart'
    Send-CrawdeEvent 'UserPromptSubmit' @{ prompt = 'Verify Crawde connection' }
    Send-CrawdeEvent 'PreToolUse' @{ tool_name = 'Read'; tool_input = @{ file_path = 'README.md' } }
    Send-CrawdeEvent 'SubagentStart' @{ agent_id = 'smoke-agent' }
    Send-CrawdeEvent 'SubagentStop' @{ agent_id = 'smoke-agent' }
    Send-CrawdeEvent 'PermissionRequest' @{ tool_name = 'Bash' }
    Send-CrawdeEvent 'PostToolUse' @{ tool_name = 'Bash' }
    Send-CrawdeEvent 'PostToolUseFailure' @{ tool_name = 'Bash' }
    Send-CrawdeEvent 'Stop'
    Write-Output 'PASS: authentication, origin rejection, and nine lifecycle events.'
} finally {
    Send-CrawdeEvent 'SessionEnd'
}
