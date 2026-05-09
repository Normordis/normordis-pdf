<#
.SYNOPSIS
Backs up the normordis-pdf repository to a configurable target folder.

.DESCRIPTION
This script compresses the repository contents into a timestamped `.tar.zip` archive.
It defaults to `D:\Backup\normordis-pdf` but accepts any target path.
The script supports a dry run and excludes `.git` by default.

.PARAMETER TargetPath
Destination folder for the backup archive.

.PARAMETER SourcePath
Repository source folder to back up. Defaults to the repository root.

.PARAMETER DryRun
Shows the archive name and contents without actually writing the file.

.PARAMETER IncludeGit
Include the `.git` directory in the backup.

.EXAMPLE
./scripts/backup.ps1

.EXAMPLE
./scripts/backup.ps1 -TargetPath 'E:\Backups\normordis-pdf' -DryRun
#>

[CmdletBinding(SupportsShouldProcess=$true)]
param(
    [Parameter(Position = 0)]
    [string]$TargetPath = 'D:\Backup\normordis-pdf',

    [Parameter(Position = 1)]
    [string]$SourcePath = (Split-Path -Parent $PSScriptRoot),

    [switch]$DryRun,
    [switch]$IncludeGit
)

function Get-AbsolutePath {
    param([string]$PathValue)
    return (Resolve-Path -Path $PathValue -ErrorAction Stop).ProviderPath
}

try {
    $source = Get-AbsolutePath -PathValue $SourcePath
} catch {
    Write-Error "Source path does not exist: $SourcePath"
    exit 1
}

$target = [System.IO.Path]::GetFullPath($TargetPath)
$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$archiveFileName = "normordis-pdf-backup-$timestamp.tar.zip"
$archivePath = Join-Path $target $archiveFileName

Write-Host "Backup source: $source"
Write-Host "Backup target directory: $target"
Write-Host "Archive file: $archivePath"
Write-Host "Include .git: $IncludeGit"
Write-Host "Dry run: $DryRun"

if ($DryRun) {
    Write-Host "DRY RUN: would create archive $archivePath"
    if (-not (Test-Path -Path $target)) {
        Write-Host "DRY RUN: would create target directory $target"
    }
    $items = Get-ChildItem -Path $source -Force | Where-Object { ($IncludeGit -or $_.Name -ne '.git') -and $_.Name -ne 'target' }
    foreach ($item in $items) {
        Write-Host "DRY RUN: include item: $($item.Name)"
    }
    exit 0
}

if ($PSCmdlet.ShouldProcess($target, 'Create backup target directory') -and -not (Test-Path -Path $target)) {
    New-Item -ItemType Directory -Path $target -Force | Out-Null
}

$items = Get-ChildItem -Path $source -Force | Where-Object { ($IncludeGit -or $_.Name -ne '.git') -and $_.Name -ne 'target' }

if ($PSCmdlet.ShouldProcess($archivePath, 'Create compressed backup archive')) {
    try {
        Write-Host "Creating archive $archivePath"
        Compress-Archive -Path $items -DestinationPath $archivePath -Force
        Write-Host "Backup created: $archivePath"
        exit 0
    } catch {
        Write-Error "Failed to create archive: $_"
        exit 1
    }
} else {
    Write-Host 'Operation skipped by ShouldProcess.'
    exit 0
}
