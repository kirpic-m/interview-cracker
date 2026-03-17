#!/bin/bash
# Interview Cracker - One-Line Installer
# 
# Linux/macOS: curl -sSL https://interviewcracker.app/install | bash
# Or: bash <(curl -sSL https://interviewcracker.app/install)
#
# This script downloads and installs everything automatically.

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

GITHUB_REPO="https://github.com/your-username/interview-cracker"
INSTALL_DIR="$HOME/.interview-cracker"
BIN_DIR="$HOME/.local/bin"

echo ""
echo -e "${CYAN}${BOLD}"
echo "  ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗"
echo "  ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║"
echo "  ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝"
echo -e "${NC}"
echo -e "${BOLD}AI interview assistant${NC}"
echo ""
echo -e "${BLUE}Installing...${NC}"
echo ""

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

# Check for pre-built binary
check_prebuilt() {
    echo -e "Checking for pre-built release..."
    
    # Determine platform
    case "$OS" in
        Linux)  PLATFORM="linux" ;;
        Darwin) PLATFORM="macos" ;;
        *)      PLATFORM="unknown" ;;
    esac
    
    # Try to download pre-built binary
    DOWNLOAD_URL="${GITHUB_REPO}/releases/latest/download/interview-cracker-${PLATFORM}-${ARCH}"
    
    if command -v curl &> /dev/null; then
        if curl -sL --head "$DOWNLOAD_URL" | grep -q "200 OK"; then
            echo -e "${GREEN}Found pre-built binary! Downloading...${NC}"
            mkdir -p "$INSTALL_DIR" "$BIN_DIR"
            curl -sL "$DOWNLOAD_URL" -o "$INSTALL_DIR/interview-cracker"
            chmod +x "$INSTALL_DIR/interview-cracker"
            ln -sf "$INSTALL_DIR/interview-cracker" "$BIN_DIR/interview-cracker"
            return 0
        fi
    fi
    
    return 1
}

# Quick install for Linux
install_linux() {
    echo -e "${YELLOW}Installing dependencies...${NC}"
    
    if command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm --needed webkit2gtk-4.1 gtk3 libsoup3 base-devel openssl librsvg libappindicator-gtk3
    elif command -v apt &> /dev/null; then
        sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev build-essential libssl-dev librsvg2-dev libayatana-appindicator3-dev
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y webkit2gtk4.1-devel gtk3-devel libsoup3-devel gcc-c++ openssl-devel librsvg2-devel
    fi
}

# Quick install for macOS
install_macos() {
    if ! command -v brew &> /dev/null; then
        echo -e "${YELLOW}Installing Homebrew...${NC}"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    echo -e "${GREEN}✓ Dependencies ready${NC}"
}

# Install Rust
install_rust() {
    if ! command -v rustc &> /dev/null; then
        echo -e "${YELLOW}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
        source "$HOME/.cargo/env" 2>/dev/null
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
}

# Install Node.js  
install_node() {
    if ! command -v node &> /dev/null; then
        echo -e "${YELLOW}Installing Node.js...${NC}"
        if command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm nodejs npm
        elif command -v apt &> /dev/null; then
            sudo apt install -y nodejs npm
        elif command -v brew &> /dev/null; then
            brew install node
        fi
    fi
}

# Build from source
build_from_source() {
    echo -e "${YELLOW}Downloading source...${NC}"
    
    if command -v git &> /dev/null; then
        git clone --depth 1 "$GITHUB_REPO.git" "$INSTALL_DIR" 2>/dev/null || {
            # If no git, download zip
            curl -sL "${GITHUB_REPO}/archive/main.zip" -o /tmp/ic.zip
            unzip -q /tmp/ic.zip -d /tmp
            mv /tmp/interview-cracker-main "$INSTALL_DIR"
        }
    else
        curl -sL "${GITHUB_REPO}/archive/main.zip" -o /tmp/ic.zip
        unzip -q /tmp/ic.zip -d /tmp
        mv /tmp/interview-cracker-main "$INSTALL_DIR"
    fi
    
    cd "$INSTALL_DIR"
    
    echo -e "${YELLOW}Installing npm packages...${NC}"
    npm install --quiet
    
    echo -e "${YELLOW}Building...${NC}"
    source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
    cd src-tauri && cargo build --release && cd ..
    
    # Create icons
    mkdir -p src-tauri/icons
    if command -v convert &> /dev/null; then
        convert -size 128x128 xc:'rgba(245,158,11,1)' PNG32:src-tauri/icons/128x128.png 2>/dev/null
    fi
}

# Create desktop entry
create_desktop_entry() {
    echo -e "${YELLOW}Creating shortcuts...${NC}"
    
    DESKTOP_FILE="$HOME/.local/share/applications/interview-cracker.desktop"
    mkdir -p "$HOME/.local/share/applications"
    
    cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Name=Interview Cracker
Comment=AI interview assistant
Exec=$INSTALL_DIR/target/release/interview-cracker
Icon=$INSTALL_DIR/src-tauri/icons/128x128.png
Terminal=false
Type=Application
Categories=Utility;Development;
Keywords=interview;ai;
EOF
    
    # Desktop
    if [ -d "$HOME/Desktop" ]; then
        cp "$DESKTOP_FILE" "$HOME/Desktop/"
        chmod +x "$HOME/Desktop/interview-cracker.desktop"
    fi
    if [ -d "$HOME/Робочий стіл" ]; then
        cp "$DESKTOP_FILE" "$HOME/Робочий стіл/"
        chmod +x "$HOME/Робочий стіл/interview-cracker.desktop"
    fi
    
    update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
}

# Main
main() {
    # Try pre-built first
    if check_prebuilt; then
        create_desktop_entry
        echo ""
        echo -e "${GREEN}${BOLD}✓ Installed from pre-built binary!${NC}"
        echo -e "Run: ${CYAN}interview-cracker${NC} or click the desktop icon"
        exit 0
    fi
    
    # Install system deps
    case "$OS" in
        Linux)  install_linux ;;
        Darwin) install_macos ;;
        *)      echo "Unsupported OS"; exit 1 ;;
    esac
    
    # Install build tools
    install_rust
    install_node
    
    # Build
    build_from_source
    
    # Create shortcuts
    create_desktop_entry
    
    echo ""
    echo -e "${GREEN}${BOLD}═══════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}✓ Installation complete!${NC}"
    echo -e "${GREEN}${BOLD}═══════════════════════════════════════${NC}"
    echo ""
    echo -e "Find ${CYAN}${BOLD}Interview Cracker${NC} in:"
    echo -e "  📱 Applications menu"
    echo -e "  🖥️  Desktop icon"
    echo ""
    echo -e "Run: ${CYAN}$INSTALL_DIR/target/release/interview-cracker${NC}"
    echo ""
}

main "$@"
