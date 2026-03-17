# Interview Cracker - Universal Installer for Windows
# Run in PowerShell as Administrator:
# irm https://.../install.ps1 | iex
# Or: .\install.ps1

$ErrorActionPreference = "Stop"
$InstallDir = "$env:USERPROFILE\interview-cracker"

function Write-Color($Text, $Color = "White") {
    Write-Host $Text -ForegroundColor $Color
}

function Test-Admin {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Header
Write-Host ""
Write-Host "  ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗" -ForegroundColor Cyan
Write-Host "  ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║" -ForegroundColor Cyan
Write-Host "  ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "AI-powered interview assistant" -ForegroundColor Green
Write-Host ""

# Check admin
if (!(Test-Admin)) {
    Write-Color "Please run PowerShell as Administrator!" "Red"
    Write-Host "Right-click PowerShell -> Run as Administrator"
    pause
    exit 1
}

Write-Color "═══════════════════════════════════════" "Blue"
Write-Host ""

# Step 1: Chocolatey
Write-Color "[1/6] Package Manager" "Blue"
if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Color "  Installing Chocolatey..." "Yellow"
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = 3072
    iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}
Write-Color "  ✓ Chocolatey" "Green"
Write-Host ""

# Step 2: Rust
Write-Color "[2/6] Rust" "Blue"
if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Color "  Installing..." "Yellow"
    Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User") + ";$env:USERPROFILE\.cargo\bin"
}
Write-Color "  ✓ Rust $(rustc --version 2>$null | Select-String '\d+\.\d+\.\d+' | ForEach-Object { $_.Matches[0].Value })" "Green"
Write-Host ""

# Step 3: Node.js
Write-Color "[3/6] Node.js" "Blue"
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Color "  Installing..." "Yellow"
    choco install nodejs-lts -y
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}
Write-Color "  ✓ Node.js $(node --version 2>$null)" "Green"
Write-Host ""

# Step 4: Build Tools
Write-Color "[4/6] Build Tools" "Blue"
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (!(Test-Path $vswhere) -or !(& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null)) {
    Write-Color "  Installing VS Build Tools..." "Yellow"
    choco install visualstudio2022buildtools -y --no-progress
    choco install visualstudio2022-workload-vctools -y --no-progress
}
Write-Color "  ✓ Build Tools" "Green"
Write-Host ""

# WebView2
if (!(Test-Path "${env:ProgramFiles(x86)}\Microsoft\EdgeWebView\Application")) {
    choco install webview2-runtime -y --no-progress
}
Write-Color "  ✓ WebView2" "Green"
Write-Host ""

# Step 5: Build
Write-Color "[5/6] Building" "Blue"
if (Test-Path $InstallDir) {
    Write-Color "  Updating..." "Yellow"
    Set-Location $InstallDir
    git pull 2>$null
} else {
    $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    if (Test-Path "$ScriptDir\package.json") {
        Write-Color "  Copying files..." "Yellow"
        Copy-Item -Path $ScriptDir -Destination $InstallDir -Recurse -Force
    } else {
        Write-Color "  Cloning..." "Yellow"
        git clone https://github.com/your-username/interview-cracker.git $InstallDir
    }
    Set-Location $InstallDir
}

npm install 2>$null
Write-Color "  ✓ Dependencies" "Green"

# Create icons
Set-Location "$InstallDir\src-tauri\icons"
magick convert -size 32x32 xc:"rgba(245,158,11,1)" PNG32:32x32.png 2>$null
magick convert -size 128x128 xc:"rgba(245,158,11,1)" PNG32:128x128.png 2>$null
magick convert 128x128.png -resize 256x256 PNG32:128x128@2x.png 2>$null
magick convert 128x128.png icon.ico 2>$null
Set-Location $InstallDir
Write-Color "  ✓ Icons" "Green"

# Build release
Set-Location "$InstallDir\src-tauri"
cargo build --release
Set-Location $InstallDir
Write-Color "  ✓ Build complete" "Green"
Write-Host ""

# Step 6: Desktop Shortcut
Write-Color "[6/6] Desktop Shortcut" "Blue"

# Create VBS launcher (no console)
@"
Set WshShell = CreateObject("WScript.Shell")
WshShell.CurrentDirectory = "$($InstallDir.Replace('\', '\\'))"
WshShell.Run "target\release\interview-cracker.exe", 1, False
"@ | Out-File -FilePath "$InstallDir\launch.vbs" -Encoding ASCII

# Create shortcut
$WshShell = New-Object -ComObject WScript.Shell
$DesktopPath = [Environment]::GetFolderPath("Desktop")

$Shortcut = $WshShell.CreateShortcut("$DesktopPath\Interview Cracker.lnk")
$Shortcut.TargetPath = "$InstallDir\launch.vbs"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.IconLocation = "$InstallDir\src-tauri\icons\icon.ico"
$Shortcut.Description = "AI-powered interview assistant"
$Shortcut.Save()

# Start menu shortcut
$StartMenuPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$Shortcut2 = $WshShell.CreateShortcut("$StartMenuPath\Interview Cracker.lnk")
$Shortcut2.TargetPath = "$InstallDir\launch.vbs"
$Shortcut2.WorkingDirectory = $InstallDir
$Shortcut2.IconLocation = "$InstallDir\src-tauri\icons\icon.ico"
$Shortcut2.Save()

Write-Color "  ✓ Desktop + Start Menu" "Green"
Write-Host ""

# Done
Write-Color "═══════════════════════════════════════" "Green"
Write-Color "✓ Installation complete!" "Green"
Write-Color "═══════════════════════════════════════" "Green"
Write-Host ""
Write-Host "Find 'Interview Cracker' on your:" -ForegroundColor White
Write-Host "  • Desktop" -ForegroundColor Cyan
Write-Host "  • Start Menu" -ForegroundColor Cyan
Write-Host ""

$run = Read-Host "Run now? (y/n)"
if ($run -eq 'y') {
    & "$InstallDir\launch.vbs"
}
