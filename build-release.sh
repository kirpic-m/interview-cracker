#!/bin/bash
# Interview Cracker - Build Release Installers
# Creates installers for Linux, macOS, and Windows
# Usage: ./build-release.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
RELEASE_DIR="$PROJECT_DIR/releases"
VERSION="1.0.0"

echo ""
echo -e "${CYAN}  ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗${NC}"
echo -e "${CYAN}  ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║${NC}"
echo -e "${CYAN}  ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝${NC}"
echo ""
echo -e "${GREEN}Building release installers v${VERSION}${NC}"
echo ""

# Clean and prepare
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Build frontend
echo -e "${BLUE}[1/4]${NC} Building frontend..."
npm run build
echo -e "${GREEN}  ✓ Frontend built${NC}"

# Build Rust backend
echo -e "${BLUE}[2/4]${NC} Building Rust backend..."
source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
cd "$PROJECT_DIR/src-tauri"

# Create icons if missing
mkdir -p icons
if [ ! -f "icons/32x32.png" ]; then
    convert -size 32x32 xc:'rgba(245,158,11,1)' PNG32:icons/32x32.png 2>/dev/null
    convert -size 128x128 xc:'rgba(245,158,11,1)' PNG32:icons/128x128.png 2>/dev/null
    convert icons/128x128.png -resize 256x256 PNG32:icons/128x128@2x.png 2>/dev/null
    cp icons/128x128.png icons/icon.icns 2>/dev/null
    convert icons/128x128.png icons/icon.ico 2>/dev/null
fi

cargo build --release
echo -e "${GREEN}  ✓ Backend built${NC}"

# Create packages
echo -e "${BLUE}[3/4]${NC} Creating installers..."

cd "$PROJECT_DIR"

# Linux AppImage
echo -e "${YELLOW}  Creating Linux AppImage...${NC}"
LINUX_DIR="$RELEASE_DIR/linux"
mkdir -p "$LINUX_DIR/usr/bin" "$LINUX_DIR/usr/share/applications" "$LINUX_DIR/usr/share/icons/hicolor/128x128/apps"

cp src-tauri/target/release/interview-cracker "$LINUX_DIR/usr/bin/"
cp src-tauri/icons/128x128.png "$LINUX_DIR/usr/share/icons/hicolor/128x128/apps/interview-cracker.png"

cat > "$LINUX_DIR/usr/share/applications/interview-cracker.desktop" << 'EOF'
[Desktop Entry]
Name=Interview Cracker
Comment=AI interview assistant
Exec=interview-cracker
Icon=interview-cracker
Terminal=false
Type=Application
Categories=Utility;
EOF

cat > "$LINUX_DIR/AppRun" << 'APPEOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/interview-cracker" "$@"
APPEOF
chmod +x "$LINUX_DIR/AppRun"

cat > "$LINUX_DIR/interview-cracker.desktop" << 'EOF'
[Desktop Entry]
Name=Interview Cracker
Exec=interview-cracker
Icon=interview-cracker
Type=Application
Categories=Utility;
EOF

# Download appimagetool if not present
if [ ! -f "/tmp/appimagetool" ]; then
    curl -L -o /tmp/appimagetool "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x /tmp/appimagetool
fi

cd "$LINUX_DIR"
/tmp/appimagetool . "$RELEASE_DIR/Interview-Cracker-${VERSION}-x86_64.AppImage" 2>/dev/null || \
    echo -e "${YELLOW}  (AppImage creation requires appimagetool)${NC}"
cd "$PROJECT_DIR"

# Create .deb package
echo -e "${YELLOW}  Creating Debian package...${NC}"
DEB_DIR="$RELEASE_DIR/deb"
mkdir -p "$DEB_DIR/DEBIAN" "$DEB_DIR/usr/bin" "$DEB_DIR/usr/share/applications" "$DEB_DIR/usr/share/icons/hicolor/128x128/apps"

cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: interview-cracker
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Depends: libwebkit2gtk-4.1-0, libgtk-3-0, libsoup-3.0-0, libssl3
Maintainer: Interview Cracker <support@interviewcracker.app>
Description: AI-powered interview assistant
 Crack any interview with AI-powered real-time assistance.
EOF

cp src-tauri/target/release/interview-cracker "$DEB_DIR/usr/bin/"
cp src-tauri/icons/128x128.png "$DEB_DIR/usr/share/icons/hicolor/128x128/apps/interview-cracker.png"
cp "$LINUX_DIR/usr/share/applications/interview-cracker.desktop" "$DEB_DIR/usr/share/applications/"

cd "$DEB_DIR"
dpkg-deb --build . "$RELEASE_DIR/Interview-Cracker-${VERSION}-amd64.deb" 2>/dev/null || \
    echo -e "${YELLOW}  (deb creation requires dpkg-deb)${NC}"
cd "$PROJECT_DIR"

# macOS .dmg (if on macOS)
if [[ "$(uname)" == "Darwin" ]]; then
    echo -e "${YELLOW}  Creating macOS .dmg...${NC}"
    cargo tauri build --target aarch64-apple-darwin
    cargo tauri build --target x86_64-apple-darwin
    echo -e "${GREEN}  ✓ macOS builds created${NC}"
fi

# Windows installer script
echo -e "${YELLOW}  Creating Windows installer script...${NC}"
cat > "$RELEASE_DIR/install-windows.bat" << 'BAT'
@echo off
title Interview Cracker Installer
echo.
echo   ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗
echo   ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║
echo   ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝
echo.
echo   AI-powered interview assistant
echo.

set INSTALL_DIR=%LOCALAPPDATA%\InterviewCracker

echo [1/3] Installing to %INSTALL_DIR%...
if exist "%INSTALL_DIR%" rmdir /s /q "%INSTALL_DIR%"
mkdir "%INSTALL_DIR%"

echo [2/3] Copying files...
xcopy /s /e /y "%~dp0app\*" "%INSTALL_DIR%\"

echo [3/3] Creating shortcuts...
powershell -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('%USERPROFILE%\Desktop\Interview Cracker.lnk'); $s.TargetPath = '%INSTALL_DIR%\Interview Cracker.exe'; $s.IconLocation = '%INSTALL_DIR%\icon.ico'; $s.Save()"
powershell -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('%APPDATA%\Microsoft\Windows\Start Menu\Programs\Interview Cracker.lnk'); $s.TargetPath = '%INSTALL_DIR%\Interview Cracker.exe'; $s.IconLocation = '%INSTALL_DIR%\icon.ico'; $s.Save()"

echo.
echo ═══════════════════════════════════════
echo ✓ Installation complete!
echo ═══════════════════════════════════════
echo.
echo Find "Interview Cracker" on your:
echo   - Desktop
echo   - Start Menu
echo.
pause
BAT

echo -e "${GREEN}  ✓ Installers created${NC}"

echo ""
echo -e "${BLUE}[4/4]${NC} Summary"
echo ""

echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}✓ Release v${VERSION} built!${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
echo -e "Installers in: ${CYAN}$RELEASE_DIR/${NC}"
echo ""
ls -la "$RELEASE_DIR/" 2>/dev/null | grep -E '\.(AppImage|deb|exe|dmg|bat)$' || echo "Files ready"
echo ""
echo -e "${YELLOW}How users install:${NC}"
echo -e "  Linux:   ${CYAN}double-click .AppImage or .deb${NC}"
echo -e "  Windows: ${CYAN}double-click install-windows.bat${NC}"
echo -e "  macOS:   ${CYAN}double-click .dmg${NC}"
