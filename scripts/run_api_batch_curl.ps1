param(
    [string]$BaseUrl = "http://localhost:8080",
    [string]$AdminEmail = "admin@system.com",
    [string]$AdminPassword = "password123",
    [switch]$SkipAdmin
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:PassCount = 0
$script:FailCount = 0

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

Write-Host "Running API batch with curl.exe against: $BaseUrl" -ForegroundColor Cyan

$suffix = Get-Date -Format "yyyyMMddHHmmss"
$userName = "test$suffix"
$userEmail = "test$suffix@example.com"
$userPassword = "password123"
$fromDate = "2026-03-01"
$toDate = "2026-03-31"

# Public APIs
Invoke-CurlJson -Name "Root" -Method "GET" -Path "/" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Health Live" -Method "GET" -Path "/health/live" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Health Ready" -Method "GET" -Path "/health/ready" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "OpenAPI" -Method "GET" -Path "/api-docs/openapi.json" -ExpectStatus @(200) | Out-Null
Invoke-CurlJson -Name "Docs" -Method "GET" -Path "/docs" -ExpectStatus @(200) | Out-Null

# Auth + user flow
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

if ($link1Res.Ok -and $link1Res.Json) {
    $shortCode1 = [string]$link1Res.Json.short_code
    Invoke-CurlJson -Name "Redirect by short code" -Method "GET" -Path "/$shortCode1" -ExpectStatus @(302, 303, 307, 308) | Out-Null

    $linkId1 = [int64]$link1Res.Json.id
    Invoke-CurlJson -Name "Soft delete my link #1" -Method "DELETE" -Path "/links/$linkId1" -Token $userAccessToken -ExpectStatus @(200) | Out-Null
}

Invoke-CurlJson -Name "Logout user" -Method "POST" -Path "/logout" -JsonBody $refreshBody -ExpectStatus @(200) | Out-Null

# Optional admin flow
if (-not $SkipAdmin) {
    $adminLoginBody = @{
        email = $AdminEmail
        password = $AdminPassword
    } | ConvertTo-Json -Compress
    $adminLoginRes = Invoke-CurlJson -Name "Login admin" -Method "POST" -Path "/login" -JsonBody $adminLoginBody -ExpectStatus @(200)

    if ($adminLoginRes.Ok -and $adminLoginRes.Json) {
        $adminAccessToken = [string]$adminLoginRes.Json.access_token

        Invoke-CurlJson -Name "Admin list users" -Method "GET" -Path "/admin/users" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
        Invoke-CurlJson -Name "Admin get user by id" -Method "GET" -Path "/admin/users/$userId" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
        Invoke-CurlJson -Name "Admin list links" -Method "GET" -Path "/admin/links" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null

        if ($link2Res.Ok -and $link2Res.Json) {
            $linkId2 = [int64]$link2Res.Json.id
            Invoke-CurlJson -Name "Admin disable link #2" -Method "DELETE" -Path "/admin/links/$linkId2" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
        }

        Invoke-CurlJson -Name "Admin soft delete user" -Method "DELETE" -Path "/admin/users/$userId" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
        Invoke-CurlJson -Name "Admin hard delete user" -Method "DELETE" -Path "/admin/users/$userId/hard" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
    } else {
        Write-Host "Skipping admin-only endpoints because admin login failed." -ForegroundColor Yellow
    }
}

Write-Host "" 
Write-Host "Batch finished. Pass=$($script:PassCount), Fail=$($script:FailCount)" -ForegroundColor Cyan

if ($script:FailCount -gt 0) {
    exit 1
}
