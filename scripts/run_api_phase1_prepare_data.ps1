param(
    [string]$BaseUrl = "http://localhost:8080"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:PassCount = 0
$script:FailCount = 0
$statePath = Join-Path $PSScriptRoot "api_batch_state.json"

function Write-Pass {
    param([string]$Message)
    $script:PassCount++
    Write-Host "[PASS] $Message" -ForegroundColor Green
}

function Write-Fail {
    param(
        [string]$Message,
        [int]$Status,
        [string]$Body
    )
    $script:FailCount++
    Write-Host "[FAIL] $Message (status=$Status)" -ForegroundColor Red
    if ($Body) {
        Write-Host "       body: $Body" -ForegroundColor DarkRed
    }
}

function Invoke-CurlJson {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Path,
        [string]$Token = "",
        [string]$JsonBody = "",
        [int[]]$ExpectStatus = @(200)
    )

    $url = "$BaseUrl$Path"
    $args = @("-sS", "-X", $Method, $url, "-H", "Accept: application/json")
    $tempBodyFile = ""

    if ($Token) {
        $args += @("-H", "Authorization: Bearer $Token")
    }

    if ($JsonBody) {
        $tempBodyFile = [System.IO.Path]::GetTempFileName()
        Set-Content -Path $tempBodyFile -Value $JsonBody -NoNewline -Encoding ascii
        $args += @("-H", "Content-Type: application/json", "--data-binary", "@$tempBodyFile")
    }

    $args += @("-w", "`n%{http_code}")

    try {
        $raw = & curl.exe @args
    }
    finally {
        if ($tempBodyFile -and (Test-Path $tempBodyFile)) {
            Remove-Item $tempBodyFile -Force -ErrorAction SilentlyContinue
        }
    }

    if ($LASTEXITCODE -ne 0) {
        $script:FailCount++
        Write-Host "[FAIL] $Name (curl exit code $LASTEXITCODE)" -ForegroundColor Red
        return [pscustomobject]@{ Ok = $false; Status = 0; Body = ""; Json = $null }
    }

    $parts = $raw -split "`n"
    $status = [int]$parts[-1]
    $body = ""
    if ($parts.Length -gt 1) {
        $body = ($parts[0..($parts.Length - 2)] -join "`n").Trim()
    }

    $ok = $ExpectStatus -contains $status
    if ($ok) {
        Write-Pass "$Name [$Method $Path]"
    } else {
        Write-Fail -Message "$Name [$Method $Path]" -Status $status -Body $body
    }

    $json = $null
    if ($body) {
        try { $json = $body | ConvertFrom-Json } catch { }
    }

    return [pscustomobject]@{
        Ok = $ok
        Status = $status
        Body = $body
        Json = $json
    }
}

Write-Host "Phase 1: Prepare test data and pause for screenshots" -ForegroundColor Cyan
Write-Host "Target: $BaseUrl" -ForegroundColor Cyan

if (Test-Path $statePath) {
    Remove-Item $statePath -Force
}

$suffix = Get-Date -Format "yyyyMMddHHmmss"
$userName = "test$suffix"
$userEmail = "test$suffix@example.com"
$userPassword = "password123"
$fromDate = "2026-03-01"
$toDate = "2026-03-31"

Invoke-CurlJson -Name "Root" -Method "GET" -Path "/" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Health Live" -Method "GET" -Path "/health/live" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Health Ready" -Method "GET" -Path "/health/ready" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "OpenAPI" -Method "GET" -Path "/api-docs/openapi.json" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Docs" -Method "GET" -Path "/docs" -ExpectStatus @(200) | Out-Null

$registerBody = @{
    username = $userName
    email = $userEmail
    password = $userPassword
} | ConvertTo-Json -Compress
$registerRes = Invoke-CurlJson -Name "Register user" -Method "POST" -Path "/register" -JsonBody $registerBody -ExpectStatus @(200)

if (-not $registerRes.Ok -or -not $registerRes.Json) {
    Write-Host "Cannot continue because user registration failed." -ForegroundColor Yellow
    exit 1
}

$userId = [int64]$registerRes.Json.id

$loginBody = @{
    email = $userEmail
    password = $userPassword
} | ConvertTo-Json -Compress
$loginRes = Invoke-CurlJson -Name "Login user" -Method "POST" -Path "/login" -JsonBody $loginBody -ExpectStatus @(200)

if (-not $loginRes.Ok -or -not $loginRes.Json) {
    Write-Host "Cannot continue because login failed." -ForegroundColor Yellow
    exit 1
}

$userAccessToken = [string]$loginRes.Json.access_token
$userRefreshToken = [string]$loginRes.Json.refresh_token

$refreshBody = @{
    refresh_token = $userRefreshToken
} | ConvertTo-Json -Compress
Invoke-CurlJson -Name "Refresh access token" -Method "POST" -Path "/refresh" -JsonBody $refreshBody -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Get my profile" -Method "GET" -Path "/users/me" -Token $userAccessToken -ExpectStatus @(200) | Out-Null

$linkBody1 = @{
    original_url = "https://google.com"
    title = "Google One"
} | ConvertTo-Json -Compress
$link1Res = Invoke-CurlJson -Name "Create link #1" -Method "POST" -Path "/links" -Token $userAccessToken -JsonBody $linkBody1 -ExpectStatus @(200)

$linkBody2 = @{
    original_url = "https://example.com"
    title = "Example Two"
} | ConvertTo-Json -Compress
$link2Res = Invoke-CurlJson -Name "Create link #2" -Method "POST" -Path "/links" -Token $userAccessToken -JsonBody $linkBody2 -ExpectStatus @(200)

Invoke-CurlJson -Name "List my links" -Method "GET" -Path "/links/my-links" -Token $userAccessToken -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Daily analytics" -Method "GET" -Path "/links/analytics?from=$fromDate&to=$toDate" -Token $userAccessToken -ExpectStatus @(200) | Out-Null

if (-not $link1Res.Ok -or -not $link1Res.Json -or -not $link2Res.Ok -or -not $link2Res.Json) {
    Write-Host "Cannot continue because link creation failed." -ForegroundColor Yellow
    exit 1
}

$state = [pscustomobject]@{
    base_url = $BaseUrl
    created_at = (Get-Date).ToString("s")
    user = [pscustomobject]@{
        id = $userId
        username = $userName
        email = $userEmail
        password = $userPassword
        access_token = $userAccessToken
        refresh_token = $userRefreshToken
    }
    links = [pscustomobject]@{
        link1_id = [int64]$link1Res.Json.id
        link1_short_code = [string]$link1Res.Json.short_code
        link2_id = [int64]$link2Res.Json.id
        link2_short_code = [string]$link2Res.Json.short_code
    }
}

$state | ConvertTo-Json -Depth 6 | Set-Content -Path $statePath -Encoding ascii

Write-Host "" 
Write-Host "Phase 1 finished. Pass=$($script:PassCount), Fail=$($script:FailCount)" -ForegroundColor Cyan
Write-Host "State saved at: $statePath" -ForegroundColor Cyan
Write-Host "Now you can take screenshots of changed data, then run phase 2." -ForegroundColor Yellow

if ($script:FailCount -gt 0) {
    exit 1
}
