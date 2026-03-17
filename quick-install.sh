#!/bin/bash
# Interview Cracker - One-Line Installer for Linux/macOS
# Usage: curl -sSL https://raw.githubusercontent.com/kirpic-m/interview-cracker/main/quick-install.sh | bash

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

GITHUB_REPO="https://github.com/kirpic-m/interview-cracker"
INSTALL_DIR="$HOME/interview-cracker"

echo ""
echo -e "${CYAN}Interview Cracker - AI Interview Assistant${NC}"
echo ""

install_rust() {
    if ! command -v rustc &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
        source "$HOME/.cargo/env" 2>/dev/null
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    echo "✓ Rust"
}

install_node() {
    if ! command -v node &> /dev/null; then
        echo "Installing Node.js..."
        if command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm nodejs npm
        elif command -v apt &> /dev/null; then
            sudo apt update && sudo apt install -y nodejs npm
        elif command -v brew &> /dev/null; then
            brew install node
        fi
    fi
    echo "✓ Node.js"
}

install_deps() {
    echo "Installing dependencies..."
    if command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm --needed webkit2gtk-4.1 gtk3 libsoup3 base-devel openssl librsvg libappindicator-gtk3
    elif command -v apt &> /dev/null; then
        sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev build-essential libssl-dev librsvg2-dev libayatana-appindicator3-dev
    fi
    echo "✓ Dependencies"
}

build_app() {
    echo "Downloading source..."
    rm -rf "$INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
    
    curl -sL "${GITHUB_REPO}/archive/refs/heads/main.zip" -o /tmp/ic.zip
    unzip -q /tmp/ic.zip -d /tmp
    mv /tmp/interview-cracker-main/* "$INSTALL_DIR/"
    rm -rf /tmp/ic.zip /tmp/interview-cracker-main
    
    cd "$INSTALL_DIR"
    echo "Installing npm packages..."
    npm install --quiet
    
    echo "Building (5-10 min first time)..."
    source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
    cd src-tauri && cargo build --release && cd ..
    
    echo "✓ Build complete"
}

create_shortcuts() {
    echo "Creating shortcuts..."
    
    cat > "$INSTALL_DIR/launch.sh" << 'LAUNCH'
#!/bin/bash
cd "$(dirname "$(readlink -f "$0")")"
source "$HOME/.cargo/env" 2>/dev/null

if [ -f "src-tauri/target/release/interview-cracker" ]; then
    exec src-tauri/target/release/interview-cracker "$@"
else
    export PATH="$HOME/.cargo/bin:$PATH"
    nohup npm run tauri dev > /tmp/interview-cracker.log 2>&1 &
fi
LAUNCH
    chmod +x "$INSTALL_DIR/launch.sh"
    
    DESKTOP_DIR="$HOME/.local/share/applications"
    mkdir -p "$DESKTOP_DIR"
    
    cat > "$DESKTOP_DIR/interview-cracker.desktop" << EOF
[Desktop Entry]
Name=Interview Cracker
Comment=AI-powered interview assistant
Exec=$INSTALL_DIR/launch.sh
Icon=$INSTALL_DIR/src-tauri/icons/128x128.png
Terminal=false
Type=Application
Categories=Utility;Development;
EOF
    
    chmod +x "$DESKTOP_DIR/interview-cracker.desktop"
    [ -d "$HOME/Desktop" ] && cp "$DESKTOP_DIR/interview-cracker.desktop" "$HOME/Desktop/"
    [ -d "$HOME/Робочий стіл" ] && cp "$DESKTOP_DIR/interview-cracker.desktop" "$HOME/Робочий стіл/"
    
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    echo "✓ Shortcuts created"
}

echo "[1/5] Rust"; install_rust
echo "[2/5] Node.js"; install_node
echo "[3/5] Dependencies"; install_deps
echo "[4/5] Building"; build_app
echo "[5/5] Shortcuts"; create_shortcuts

echo ""
echo "═══════════════════════════════════════"
echo "✓ Installation complete!"
echo "═══════════════════════════════════════"
echo ""
echo "Find 'Interview Cracker' in:"
echo "  • Applications menu"
echo "  • Desktop icon"
echo ""
