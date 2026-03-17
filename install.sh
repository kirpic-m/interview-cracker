#!/bin/bash
set -e

# Interview Cracker - Universal Installer for Linux
# Usage: curl -sSL https://.../install.sh | bash
# Or: ./install.sh

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

INSTALL_DIR="$HOME/interview-cracker"

echo ""
echo -e "${CYAN}  ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗${NC}"
echo -e "${CYAN}  ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║${NC}"
echo -e "${CYAN}  ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝${NC}"
echo ""
echo -e "${GREEN}AI-powered interview assistant${NC}"
echo ""

# Detect OS and package manager
detect_os() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    if [[ "$OS" == "Linux" ]]; then
        if command -v pacman &> /dev/null; then
            PKG_MANAGER="pacman"
            DISTRO="arch"
        elif command -v apt &> /dev/null; then
            PKG_MANAGER="apt"
            DISTRO="debian"
        elif command -v dnf &> /dev/null; then
            PKG_MANAGER="dnf"
            DISTRO="fedora"
        else
            echo -e "${RED}Unsupported Linux distribution${NC}"
            exit 1
        fi
    elif [[ "$OS" == "Darwin" ]]; then
        PKG_MANAGER="brew"
        DISTRO="macos"
    fi
    
    echo -e "${BLUE}System:${NC} $DISTRO ($ARCH)"
}

# Install Rust
install_rust() {
    if command -v rustc &> /dev/null; then
        echo -e "${GREEN}✓ Rust $(rustc --version | cut -d' ' -f2)${NC}"
        return
    fi
    
    echo -e "${YELLOW}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env" 2>/dev/null || source "$HOME/.cargo/env.fish" 2>/dev/null
    export PATH="$HOME/.cargo/bin:$PATH"
    echo -e "${GREEN}✓ Rust installed${NC}"
}

# Install Node.js
install_node() {
    if command -v node &> /dev/null; then
        echo -e "${GREEN}✓ Node.js $(node --version)${NC}"
        return
    fi
    
    echo -e "${YELLOW}Installing Node.js...${NC}"
    case $PKG_MANAGER in
        pacman) sudo pacman -S --noconfirm nodejs npm ;;
        apt)    sudo apt update && sudo apt install -y nodejs npm ;;
        dnf)    sudo dnf install -y nodejs npm ;;
        brew)   brew install node ;;
    esac
    echo -e "${GREEN}✓ Node.js installed${NC}"
}

# Install system dependencies
install_deps() {
    echo -e "${YELLOW}Installing dependencies...${NC}"
    
    case $PKG_MANAGER in
        pacman)
            sudo pacman -S --noconfirm --needed \
                webkit2gtk-4.1 gtk3 libsoup3 base-devel openssl \
                librsvg libappindicator-gtk3 poppler imagemagick
            ;;
        apt)
            sudo apt update
            sudo apt install -y \
                libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev \
                build-essential libssl-dev librsvg2-dev \
                libayatana-appindicator3-dev poppler-utils imagemagick
            ;;
        dnf)
            sudo dnf install -y \
                webkit2gtk4.1-devel gtk3-devel libsoup3-devel \
                gcc-c++ openssl-devel librsvg2-devel \
                libappindicator-gtk3-devel poppler-utils ImageMagick
            ;;
        brew)
            echo -e "${GREEN}✓ macOS deps handled by Tauri${NC}"
            ;;
    esac
    
    echo -e "${GREEN}✓ Dependencies installed${NC}"
}

# Clone and build
build_app() {
    echo -e "${YELLOW}Building Interview Cracker...${NC}"
    
    if [ -d "$INSTALL_DIR" ]; then
        echo "Updating..."
        cd "$INSTALL_DIR"
        git pull 2>/dev/null || true
    else
        echo "Cloning..."
        # For now, copy from current directory
        SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
        if [ -f "$SCRIPT_DIR/package.json" ]; then
            cp -r "$SCRIPT_DIR" "$INSTALL_DIR"
        else
            echo -e "${RED}Cannot find source files${NC}"
            exit 1
        fi
        cd "$INSTALL_DIR"
    fi
    
    npm install
    
    source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
    
    # Create icons
    mkdir -p src-tauri/icons
    cd src-tauri/icons
    convert -size 32x32 xc:'rgba(245,158,11,1)' PNG32:32x32.png 2>/dev/null || true
    convert -size 128x128 xc:'rgba(245,158,11,1)' PNG32:128x128.png 2>/dev/null || true
    convert 128x128.png -resize 256x256 PNG32:128x128@2x.png 2>/dev/null || true
    cp 128x128.png icon.icns 2>/dev/null || true
    cd "$INSTALL_DIR"
    
    # Build release
    cd src-tauri
    cargo build --release
    cd ..
    
    echo -e "${GREEN}✓ Build complete${NC}"
}

# Install desktop shortcut
install_desktop() {
    echo -e "${YELLOW}Installing desktop shortcut...${NC}"
    
    DESKTOP_DIR="$HOME/.local/share/applications"
    mkdir -p "$DESKTOP_DIR"
    
    # Create launcher script
    cat > "$INSTALL_DIR/launch.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$(readlink -f "$0")")"
source "$HOME/.cargo/env" 2>/dev/null
fuser -k 5173/tcp 2>/dev/null

if [ -f "src-tauri/target/release/interview-cracker" ]; then
    exec src-tauri/target/release/interview-cracker
elif [ -f "target/release/interview-cracker" ]; then
    exec target/release/interview-cracker
else
    exec npm run tauri dev
fi
EOF
    chmod +x "$INSTALL_DIR/launch.sh"
    
    cat > "$DESKTOP_DIR/interview-cracker.desktop" << EOF
[Desktop Entry]
Name=Interview Cracker
Comment=AI-powered interview assistant
Exec=$INSTALL_DIR/launch.sh
Icon=$INSTALL_DIR/src-tauri/icons/128x128.png
Terminal=false
Type=Application
Categories=Utility;Development;
Keywords=interview;ai;
StartupNotify=true
EOF
    
    # Desktop icon
    if [ -d "$HOME/Desktop" ]; then
        cp "$DESKTOP_DIR/interview-cracker.desktop" "$HOME/Desktop/"
        chmod +x "$HOME/Desktop/interview-cracker.desktop"
    fi
    
    if [ -d "$HOME/Робочий стіл" ]; then
        cp "$DESKTOP_DIR/interview-cracker.desktop" "$HOME/Робочий стіл/"
        chmod +x "$HOME/Робочий стіл/interview-cracker.desktop"
    fi
    
    # Create start script
    cat > "$INSTALL_DIR/start.sh" << 'EOF'
#!/bin/bash
source "$HOME/.cargo/env" 2>/dev/null
cd "$(dirname "$0")"
fuser -k 5173/tcp 2>/dev/null
npm run tauri dev
EOF
    chmod +x "$INSTALL_DIR/start.sh"
    
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    
    echo -e "${GREEN}✓ Desktop shortcut installed${NC}"
}

# Main
main() {
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo ""
    
    detect_os
    echo ""
    
    echo -e "${BLUE}[1/5]${NC} Rust"
    install_rust
    echo ""
    
    echo -e "${BLUE}[2/5]${NC} Node.js"
    install_node
    echo ""
    
    echo -e "${BLUE}[3/5]${NC} Dependencies"
    install_deps
    echo ""
    
    echo -e "${BLUE}[4/5]${NC} Building"
    build_app
    echo ""
    
    echo -e "${BLUE}[5/5]${NC} Desktop shortcut"
    install_desktop
    echo ""
    
    echo -e "${GREEN}═══════════════════════════════════════${NC}"
    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════${NC}"
    echo ""
    echo -e "Find ${CYAN}Interview Cracker${NC} in your applications menu"
    echo -e "Or double-click the desktop icon"
    echo ""
    echo -e "Dev mode: ${YELLOW}$INSTALL_DIR/start.sh${NC}"
}

main "$@"
