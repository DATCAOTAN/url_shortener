param(
    [string]$BaseUrl = "",
    [string]$AdminEmail = "admin@system.com",
    [string]$AdminPassword = "password123"
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

    $url = "$script:ResolvedBaseUrl$Path"
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

if (-not (Test-Path $statePath)) {
    Write-Host "State file not found: $statePath" -ForegroundColor Red
    Write-Host "Run phase 1 first to generate state data." -ForegroundColor Yellow
    exit 1
}

$state = Get-Content -Path $statePath -Raw | ConvertFrom-Json
$script:ResolvedBaseUrl = if ([string]::IsNullOrWhiteSpace($BaseUrl)) { [string]$state.base_url } else { $BaseUrl }

Write-Host "Phase 2: Redirect short code and delete APIs" -ForegroundColor Cyan
Write-Host "Target: $script:ResolvedBaseUrl" -ForegroundColor Cyan

$userId = [int64]$state.user.id
$userAccessToken = [string]$state.user.access_token
$userRefreshToken = [string]$state.user.refresh_token
$link1Id = [int64]$state.links.link1_id
$link2Id = [int64]$state.links.link2_id
$link1ShortCode = [string]$state.links.link1_short_code

$refreshBody = @{
    refresh_token = $userRefreshToken
} | ConvertTo-Json -Compress
$refreshRes = Invoke-CurlJson -Name "Refresh access token" -Method "POST" -Path "/refresh" -JsonBody $refreshBody -ExpectStatus @(200)
if ($refreshRes.Ok -and $refreshRes.Json) {
    $userAccessToken = [string]$refreshRes.Json.access_token
}

Invoke-CurlJson -Name "Redirect by short code" -Method "GET" -Path "/$link1ShortCode" -ExpectStatus @(302, 303, 307, 308) | Out-Null
Invoke-CurlJson -Name "Soft delete my link #1" -Method "DELETE" -Path "/links/$link1Id" -Token $userAccessToken -ExpectStatus @(200) | Out-Null

$adminLoginBody = @{
    email = $AdminEmail
    password = $AdminPassword
} | ConvertTo-Json -Compress
$adminLoginRes = Invoke-CurlJson -Name "Login admin" -Method "POST" -Path "/login" -JsonBody $adminLoginBody -ExpectStatus @(200)

if ($adminLoginRes.Ok -and $adminLoginRes.Json) {
    $adminAccessToken = [string]$adminLoginRes.Json.access_token

    Invoke-CurlJson -Name "Admin disable link #2" -Method "DELETE" -Path "/admin/links/$link2Id" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
    Invoke-CurlJson -Name "Admin soft delete user" -Method "DELETE" -Path "/admin/users/$userId" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
    Invoke-CurlJson -Name "Admin hard delete user" -Method "DELETE" -Path "/admin/users/$userId/hard" -Token $adminAccessToken -ExpectStatus @(200) | Out-Null
} else {
    Write-Host "Skipping admin delete APIs because admin login failed." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Phase 2 finished. Pass=$($script:PassCount), Fail=$($script:FailCount)" -ForegroundColor Cyan

if ($script:FailCount -gt 0) {
    exit 1
}
