# Interview Cracker - One-Line Installer for Windows
# PowerShell (Admin): irm https://interviewcracker.app/install.ps1 | iex
#
# Downloads and installs everything automatically

$ErrorActionPreference = "SilentlyContinue"
$ProgressPreference = "SilentlyContinue"

$Bold = "`e[1m"
$Green = "`e[32m"
$Cyan = "`e[36m"
$Yellow = "`e[33m"
$Reset = "`e[0m"

Write-Host ""
Write-Host "  ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗" -ForegroundColor Cyan
Write-Host "  ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║" -ForegroundColor Cyan
Write-Host "  ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "AI interview assistant" -ForegroundColor White
Write-Host ""
Write-Host "Installing..." -ForegroundColor Yellow
Write-Host ""

$InstallDir = "$env:LOCALAPPDATA\InterviewCracker"
$GITHUB_REPO = "https://github.com/kirpic-m/interview-cracker"
$ZIP_URL = "$GITHUB_REPO/archive/refs/heads/main.zip"

# Check for pre-built release
function Try-Prebuilt {
    Write-Host "Checking for pre-built release..." -ForegroundColor Gray
    
    $downloadUrl = "$GITHUB_REPO/releases/latest/download/interview-cracker-windows-x64.msi"
    
    try {
        $response = Invoke-WebRequest -Uri $downloadUrl -Method Head -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            Write-Host "Found! Downloading..." -ForegroundColor Green
            
            $msiPath = "$env:TEMP\interview-cracker.msi"
            Invoke-WebRequest -Uri $downloadUrl -OutFile $msiPath
            
            Write-Host "Installing MSI..." -ForegroundColor Yellow
            Start-Process msiexec.exe -ArgumentList "/i `"$msiPath`" /quiet /norestart" -Wait
            
            return $true
        }
    } catch {
        # No pre-built, continue with source build
    }
    
    return $false
}

# Install Chocolatey
function Install-Choco {
    if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Host "Installing package manager..." -ForegroundColor Yellow
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = 3072
        iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    }
}

# Install Rust
function Install-Rust {
    if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Host "Installing Rust..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "$env:TEMP\rustup-init.exe"
        & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable --quiet
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User") + ";$env:USERPROFILE\.cargo\bin"
    }
}

# Install Node.js
function Install-Node {
    if (!(Get-Command node -ErrorAction SilentlyContinue)) {
        Write-Host "Installing Node.js..." -ForegroundColor Yellow
        choco install nodejs-lts -y --no-progress
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    }
}

# Install Build Tools
function Install-BuildTools {
    Write-Host "Installing build tools..." -ForegroundColor Yellow
    choco install visualstudio2022buildtools -y --no-progress
    choco install visualstudio2022-workload-vctools -y --no-progress
    choco install webview2-runtime -y --no-progress
}

# Build from source
function Build-FromSource {
    Write-Host "Downloading source..." -ForegroundColor Yellow
    
    if (Test-Path $InstallDir) {
        Remove-Item -Recurse -Force $InstallDir
    }
    
    # Download zip (no login required)
    $zipPath = "$env:TEMP\interview-cracker.zip"
    Invoke-WebRequest -Uri $ZIP_URL -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath "$env:TEMP\ic"
    Move-Item "$env:TEMP\ic\interview-cracker-main" $InstallDir
    Remove-Item $zipPath, "$env:TEMP\ic" -Recurse -Force
    
    Set-Location $InstallDir
    
    Write-Host "Installing packages..." -ForegroundColor Yellow
    npm install --silent 2>$null
    
    Write-Host "Building (this takes a few minutes)..." -ForegroundColor Yellow
    & "$env:USERPROFILE\.cargo\bin\cargo.exe" build --manifest-path src-tauri\Cargo.toml --release
    
    # Create icons
    if (Get-Command magick -ErrorAction SilentlyContinue) {
        magick convert -size 128x128 xc:"rgba(245,158,11,1)" "src-tauri\icons\128x128.png" 2>$null
        magick convert "src-tauri\icons\128x128.png" "src-tauri\icons\icon.ico" 2>$null
    }
}

# Create shortcuts
function Create-Shortcuts {
    Write-Host "Creating shortcuts..." -ForegroundColor Yellow
    
    # VBS launcher (no console)
    @"
Set WshShell = CreateObject("WScript.Shell")
WshShell.CurrentDirectory = "$($InstallDir.Replace('\', '\\'))"
WshShell.Run "src-tauri\target\release\interview-cracker.exe", 1, False
"@ | Out-File -FilePath "$InstallDir\launch.vbs" -Encoding ASCII
    
    $WshShell = New-Object -ComObject WScript.Shell
    
    # Desktop
    $desktop = [Environment]::GetFolderPath("Desktop")
    $shortcut = $WshShell.CreateShortcut("$desktop\Interview Cracker.lnk")
    $shortcut.TargetPath = "$InstallDir\launch.vbs"
    $shortcut.IconLocation = "$InstallDir\src-tauri\icons\icon.ico"
    $shortcut.Save()
    
    # Start Menu
    $startMenu = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
    $shortcut2 = $WshShell.CreateShortcut("$startMenu\Interview Cracker.lnk")
    $shortcut2.TargetPath = "$InstallDir\launch.vbs"
    $shortcut2.IconLocation = "$InstallDir\src-tauri\icons\icon.ico"
    $shortcut2.Save()
    
    # Add to PATH
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir\src-tauri\target\release", "User")
    }
}

# Main
function Main {
    # Check admin
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (!$isAdmin) {
        Write-Host "Restarting as Administrator..." -ForegroundColor Yellow
        Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -Command `"irm https://interviewcracker.app/install.ps1 | iex`""
        exit
    }
    
    # Try pre-built
    if (Try-Prebuilt) {
        Write-Host ""
        Write-Host "═══════════════════════════════════════" -ForegroundColor Green
        Write-Host "✓ Installed!" -ForegroundColor Green
        Write-Host "═══════════════════════════════════════" -ForegroundColor Green
        Write-Host ""
        Write-Host "Find 'Interview Cracker' on Desktop or Start Menu" -ForegroundColor White
        pause
        exit
    }
    
    # Install everything
    Install-Choco
    Install-Rust
    Install-Node
    Install-BuildTools
    Build-FromSource
    Create-Shortcuts
    
    Write-Host ""
    Write-Host "═══════════════════════════════════════" -ForegroundColor Green
    Write-Host "✓ Installation complete!" -ForegroundColor Green
    Write-Host "═══════════════════════════════════════" -ForegroundColor Green
    Write-Host ""
    Write-Host "Find 'Interview Cracker' on:" -ForegroundColor White
    Write-Host "  • Desktop icon" -ForegroundColor Cyan
    Write-Host "  • Start Menu" -ForegroundColor Cyan
    Write-Host ""
    
    $run = Read-Host "Run now? (y/n)"
    if ($run -eq 'y') {
        & "$InstallDir\launch.vbs"
    }
}

Main
