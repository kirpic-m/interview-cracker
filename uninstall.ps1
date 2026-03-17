# Interview Cracker - Uninstaller for Windows
# Run: .\uninstall.ps1

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗" -ForegroundColor Cyan
Write-Host "  ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║" -ForegroundColor Cyan
Write-Host "  ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Uninstalling Interview Cracker..." -ForegroundColor Red
Write-Host ""

# Stop processes
Write-Host "[1/5] Stopping processes..." -ForegroundColor Yellow
Get-Process interview-cracker -ErrorAction SilentlyContinue | Stop-Process -Force
Write-Host "  ✓ Stopped" -ForegroundColor Green

# Remove application
Write-Host "[2/5] Removing application files..." -ForegroundColor Yellow
$paths = @(
    "$env:LOCALAPPDATA\InterviewCracker",
    "$env:USERPROFILE\interview-cracker"
)
foreach ($path in $paths) {
    if (Test-Path $path) {
        Remove-Item -Recurse -Force $path
        Write-Host "  Removed: $path" -ForegroundColor Gray
    }
}
Write-Host "  ✓ Removed" -ForegroundColor Green

# Remove shortcuts
Write-Host "[3/5] Removing shortcuts..." -ForegroundColor Yellow
$shortcuts = @(
    "$env:USERPROFILE\Desktop\Interview Cracker.lnk",
    "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Interview Cracker.lnk"
)
foreach ($shortcut in $shortcuts) {
    if (Test-Path $shortcut) {
        Remove-Item -Force $shortcut
        Write-Host "  Removed: $shortcut" -ForegroundColor Gray
    }
}
Write-Host "  ✓ Removed" -ForegroundColor Green

# Remove settings
Write-Host "[4/5] Settings..." -ForegroundColor Yellow
$configPath = "$env:APPDATA\interview-cracker"
if (Test-Path $configPath) {
    $confirm = Read-Host "  Delete settings and API keys? (y/n)"
    if ($confirm -eq 'y') {
        Remove-Item -Recurse -Force $configPath
        Write-Host "  ✓ Settings deleted" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ Settings kept" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ✓ No settings found" -ForegroundColor Green
}

# Remove from PATH
Write-Host "[5/5] Cleaning PATH..." -ForegroundColor Yellow
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -like "*InterviewCracker*") {
    $newPath = ($currentPath -split ';' | Where-Object { $_ -notlike "*InterviewCracker*" }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "  ✓ Removed from PATH" -ForegroundColor Green
} else {
    Write-Host "  ✓ PATH clean" -ForegroundColor Green
}

Write-Host ""
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host "✓ Interview Cracker uninstalled" -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Host "Note: Rust, Node.js, VS Build Tools were NOT removed" -ForegroundColor Yellow
Write-Host "      (they are shared with other applications)" -ForegroundColor Yellow
Write-Host ""

pause
