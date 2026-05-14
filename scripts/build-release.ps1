# build-release.ps1 — Build normordis-pdf library and CLI tools in release mode.
#
# Usage:
#   .\scripts\build-release.ps1 [-Target <triple>] [-OutDir <dir>]
#
# Options:
#   -Target <triple>   Cross-compile target (e.g. x86_64-unknown-linux-musl).
#                      Defaults to the host target.
#   -OutDir <dir>      Directory where binaries are copied after build.
#                      Defaults to .\dist\
#
# Output:
#   dist\dotx2ndt.exe
#   dist\ndt-tools.exe

param(
    [string]$Target  = "",
    [string]$OutDir  = "dist"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Resolve workspace root ────────────────────────────────────────────────────

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

# ── Build arguments ───────────────────────────────────────────────────────────

$CargoArgs = @("build", "--release", "-p", "normordis-pdf", "-p", "dotx2ndt", "-p", "ndt-tools")
$TargetDir = Join-Path $Root "target\release"

if ($Target -ne "") {
    $CargoArgs += @("--target", $Target)
    $TargetDir = Join-Path $Root "target\$Target\release"
}

# ── Build ─────────────────────────────────────────────────────────────────────

Write-Host "==> Building normordis-pdf workspace (release)..." -ForegroundColor Cyan
& cargo @CargoArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# ── Copy binaries to output directory ─────────────────────────────────────────

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$Bins = @("dotx2ndt", "ndt-tools")
foreach ($bin in $Bins) {
    $src = Join-Path $TargetDir "$bin.exe"
    $dst = Join-Path $OutDir "$bin.exe"
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        $size = (Get-Item $dst).Length
        $sizeMb = [math]::Round($size / 1MB, 1)
        Write-Host "    $dst  ($sizeMb MB)" -ForegroundColor Green
    } else {
        Write-Warning "Expected binary not found: $src"
    }
}

Write-Host ""
Write-Host "Done. Binaries in $OutDir\" -ForegroundColor Cyan
