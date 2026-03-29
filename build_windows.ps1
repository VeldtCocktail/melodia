# build_windows.ps1
# Builds Melodia for Windows x86-64
# Run from the melodia\ directory in a PowerShell terminal.

param(
    [switch]$Debug,
    [switch]$Install
)

$ErrorActionPreference = "Stop"

Write-Host "=== Melodia Windows Build ===" -ForegroundColor Cyan

# Check for Rust
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found. Installing via rustup..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    Start-Process -Wait "$env:TEMP\rustup-init.exe" -ArgumentList "-y", "--default-toolchain", "stable"
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

Write-Host "Rust version: $(rustc --version)" -ForegroundColor Green
Write-Host "Cargo version: $(cargo --version)" -ForegroundColor Green

# Make sure we have the Windows target
rustup target add x86_64-pc-windows-msvc 2>$null

if ($Debug) {
    Write-Host "`nBuilding DEBUG build..." -ForegroundColor Yellow
    cargo build --target x86_64-pc-windows-msvc
    $outDir = "target\x86_64-pc-windows-msvc\debug"
} else {
    Write-Host "`nBuilding RELEASE build (optimized)..." -ForegroundColor Yellow
    cargo build --release --target x86_64-pc-windows-msvc
    $outDir = "target\x86_64-pc-windows-msvc\release"
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nBuild FAILED!" -ForegroundColor Red
    exit 1
}

$exePath = "$outDir\melodia.exe"
Write-Host "`nBuild succeeded!" -ForegroundColor Green
Write-Host "Binary: $((Resolve-Path $exePath).Path)" -ForegroundColor Cyan
Write-Host "Size:   $([math]::Round((Get-Item $exePath).Length / 1MB, 2)) MB" -ForegroundColor Cyan

if ($Install) {
    $installDir = "$env:LOCALAPPDATA\Melodia"
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    Copy-Item $exePath "$installDir\melodia.exe"

    # Create Start Menu shortcut
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Melodia.lnk")
    $shortcut.TargetPath = "$installDir\melodia.exe"
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Description = "Melodia Music Player"
    $shortcut.Save()

    Write-Host "`nInstalled to: $installDir" -ForegroundColor Green
    Write-Host "Start Menu shortcut created." -ForegroundColor Green
}

Write-Host "`nDone! Run with: .\$exePath" -ForegroundColor Green
