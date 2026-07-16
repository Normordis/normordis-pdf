# build-release.ps1 — Build normordis-pdf library and CLI tools in release mode.
#
# Usage:
#   .\scripts\powershell\build-release.ps1 [-Target <triple>] [-OutDir <dir>]
#                                               [-CargoTargetDir <dir>]
#
# Options:
#   -Target <triple>   Target Rust (e.g. x86_64-pc-windows-gnu).
#                      Defaults to the host target.
#   -OutDir <dir>      Directory where the library, header and CLIs are copied.
#                      Defaults to .\dist\
#   -CargoTargetDir <dir>
#                      Rust build cache. Defaults to the local machine cache,
#                      outside the repository.
#
# Output:
#   dist\normordis_pdf.dll
#   dist\include\normordis_pdf.h
#   dist\dotx2ndt.exe, dist\ndt-tools.exe

param(
    [string]$Target  = "",
    [string]$OutDir  = "dist",
    [string]$CargoTargetDir = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Resolve workspace root ────────────────────────────────────────────────────

$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $Root

if ($CargoTargetDir -eq "") {
    $CargoTargetDir = Join-Path $env:LOCALAPPDATA "NORMORDIS\normordis-pdf\target"
}

# ── Build arguments ───────────────────────────────────────────────────────────

$CargoArgs = @("build", "--release", "-p", "normordis-pdf", "-p", "dotx2ndt", "-p", "ndt-tools")
$TargetDir = Join-Path $CargoTargetDir "release"

if ($Target -ne "") {
    $CargoArgs += @("--target", $Target)
    $TargetDir = Join-Path $CargoTargetDir "$Target\release"
}

# Derive artifact names from the requested target, not from the build host.
$Platform = $Target
if ($Platform -eq "") {
    $Platform = (& rustc -vV | Where-Object { $_ -like "host: *" } | ForEach-Object { $_.Substring(6) })
}

$BinExt = ""
$Library = "libnormordis_pdf.so"
if ($Platform -match "windows") {
    $BinExt = ".exe"
    $Library = "normordis_pdf.dll"
} elseif ($Platform -match "apple-darwin") {
    $Library = "libnormordis_pdf.dylib"
}

# ── Build ─────────────────────────────────────────────────────────────────────

Write-Host "==> Building normordis-pdf workspace (release)..." -ForegroundColor Cyan
Write-Host "    Cargo target directory: $CargoTargetDir" -ForegroundColor DarkGray
$env:CARGO_TARGET_DIR = $CargoTargetDir
& cargo @CargoArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# ── Copy binaries to output directory ─────────────────────────────────────────

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutDir "include") | Out-Null

$Bins = @("dotx2ndt", "ndt-tools")
foreach ($bin in $Bins) {
    $src = Join-Path $TargetDir "$bin$BinExt"
    $dst = Join-Path $OutDir "$bin$BinExt"
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        $size = (Get-Item $dst).Length
        $sizeMb = [math]::Round($size / 1MB, 1)
        Write-Host "    $dst  ($sizeMb MB)" -ForegroundColor Green
    } else {
        Write-Warning "Expected binary not found: $src"
    }
}

$librarySrc = Join-Path $TargetDir $Library
$libraryDst = Join-Path $OutDir $Library
if (Test-Path $librarySrc) {
    Copy-Item $librarySrc $libraryDst -Force
    $sizeMb = [math]::Round((Get-Item $libraryDst).Length / 1MB, 1)
    Write-Host "    $libraryDst  ($sizeMb MB)" -ForegroundColor Green
} else {
    Write-Warning "Expected C library not found: $librarySrc"
}

$headerDst = Join-Path $OutDir "include\normordis_pdf.h"
Copy-Item (Join-Path $Root "normordis_pdf.h") $headerDst -Force
Write-Host "    $headerDst" -ForegroundColor Green

Write-Host ""
Write-Host "Done. Release artifacts for $Platform in $OutDir\" -ForegroundColor Cyan
