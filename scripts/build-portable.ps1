param(
    [string]$Configuration = "release"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$DistRoot = Join-Path $RepoRoot "dist"
$AppDir = Join-Path $DistRoot "VideoProcessManager"
$ExeSource = Join-Path $RepoRoot "target\$Configuration\video-process-manager.exe"
$ExeTarget = Join-Path $AppDir "video-process-manager.exe"
$PresetsSource = Join-Path $RepoRoot "presets.toml"
$ConfigTarget = Join-Path $AppDir "config.toml"

Push-Location $RepoRoot
try {
    cargo build --release

    if (Test-Path $AppDir) {
        Remove-Item -LiteralPath $AppDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $AppDir | Out-Null

    Copy-Item -LiteralPath $ExeSource -Destination $ExeTarget
    Copy-Item -LiteralPath $PresetsSource -Destination (Join-Path $AppDir "presets.toml")

    @"
custom_apps = []
"@ | Set-Content -Path $ConfigTarget -Encoding UTF8

    Write-Host "Portable build created:"
    Write-Host $AppDir
}
finally {
    Pop-Location
}
