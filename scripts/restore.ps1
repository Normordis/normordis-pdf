<#
.SYNOPSIS
Restores a normordis-pdf backup from a configurable source folder.

.DESCRIPTION
This script restores backup files into the repository root or a custom destination.
It defaults to `D:\Backup\normordis-pdf` as the backup source and the repository root as destination.
The script supports a dry run to preview changes.

.PARAMETER SourcePath
Backup source folder. Defaults to `D:\Backup\normordis-pdf`.

.PARAMETER DestinationPath
Destination folder for restore. Defaults to the repository root.

.PARAMETER DryRun
Shows the files that would be restored without actually writing them.

.EXAMPLE
./scripts/restore.ps1

.EXAMPLE
./scripts/restore.ps1 -SourcePath 'E:\Backups\normordis-pdf' -DryRun
#>

[CmdletBinding(SupportsShouldProcess=$true)]
param(
    [Parameter(Position = 0)]
    [string]$SourcePath = 'D:\Backup\normordis-pdf',

    [Parameter(Position = 1)]
    [string]$DestinationPath = (Split-Path -Parent $PSScriptRoot),

    [switch]$DryRun
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

$destination = [System.IO.Path]::GetFullPath($DestinationPath)

Write-Host "Restore source: $source"
Write-Host "Restore destination: $destination"
Write-Host "Dry run: $DryRun"

if ($DryRun) {
    if ($source -match '\.tar\.zip$' -or $source -match '\.zip$') {
        Write-Host "DRY RUN: would extract archive $source to $destination"
    } else {
        Write-Host "DRY RUN: would restore directory $source to $destination"
    }
    exit 0
}

if (-not (Test-Path -Path $destination)) {
    if ($PSCmdlet.ShouldProcess($destination, 'Create restore destination directory')) {
        Write-Host "Creating destination: $destination"
        New-Item -ItemType Directory -Path $destination -Force | Out-Null
    }
}

if ($source -match '\.tar\.zip$' -or $source -match '\.zip$') {
    if ($PSCmdlet.ShouldProcess($destination, 'Extract backup archive')) {
        try {
            Write-Host "Extracting archive $source to $destination"
            Expand-Archive -Path $source -DestinationPath $destination -Force
            Write-Host "Restore completed successfully."
            exit 0
        } catch {
            Write-Error "Restore failed while extracting archive: $_"
            exit 1
        }
    }
    Write-Host 'Operation skipped by ShouldProcess.'
    exit 0
}

$robocopyArgs = @(
    '/E',
    '/Z',
    '/R:3',
    '/W:5',
    '/NP',
    '/NDL',
    '/NFL'
)

$command = @('robocopy', $source, $destination) + $robocopyArgs
Write-Host "Running: $($command -join ' ')"

if ($PSCmdlet.ShouldProcess($destination, 'Restore repository from backup')) {
    & robocopy.exe $source $destination $robocopyArgs
    $exitCode = $LASTEXITCODE
} else {
    Write-Host 'Operation skipped by ShouldProcess.'
    exit 0
}

if ($exitCode -le 3) {
    Write-Host "Restore completed successfully. Robocopy exit code: $exitCode"
    exit 0
}

Write-Error "Restore failed. Robocopy exit code: $exitCode"
exit $exitCode
