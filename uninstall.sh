#!/bin/bash
# Interview Cracker - Uninstaller
# Usage: ./uninstall.sh

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo ""
echo -e "${CYAN}  ╦╔╦╗╔═╗╦═╗╦╔╦╗╔═╗╦    ╔═╗╦ ╦╔═╗╔╦╗╔═╗╔╦╗${NC}"
echo -e "${CYAN}  ║║║║╠═╝╠╦╝║ ║ ╠═╣║    ╚═╗║ ║╚═╗ ║ ║╣  ║║${NC}"
echo -e "${CYAN}  ╩ ╩ ╩  ╩╚═╩ ╩ ╩ ╩╩═╝  ╚═╝╚═╝╚═╝ ╩ ╚═╝═╩╝${NC}"
echo ""
echo -e "${RED}Uninstalling Interview Cracker...${NC}"
echo ""

# Kill running processes
echo -e "${YELLOW}[1/5]${NC} Stopping processes..."
pkill -f interview-cracker 2>/dev/null || true
fuser -k 5173/tcp 2>/dev/null || true
echo -e "${GREEN}  ✓ Stopped${NC}"

# Remove application directory
echo -e "${YELLOW}[2/5]${NC} Removing application files..."
rm -rf "$HOME/.interview-cracker"
rm -rf "$HOME/interview-cracker"
echo -e "${GREEN}  ✓ Removed${NC}"

# Remove desktop shortcuts
echo -e "${YELLOW}[3/5]${NC} Removing shortcuts..."
rm -f "$HOME/.local/share/applications/interview-cracker.desktop"
rm -f "$HOME/Desktop/interview-cracker.desktop"
rm -f "$HOME/Робочий стіл/interview-cracker.desktop"
update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
echo -e "${GREEN}  ✓ Removed${NC}"

# Remove settings (ask user)
echo -e "${YELLOW}[4/5]${NC} Settings..."
if [ -d "$HOME/.config/interview-cracker" ]; then
    echo -e "  Found settings in: $HOME/.config/interview-cracker"
    read -p "  Delete settings and API keys? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$HOME/.config/interview-cracker"
        echo -e "${GREEN}  ✓ Settings deleted${NC}"
    else
        echo -e "${YELLOW}  ⚠ Settings kept${NC}"
    fi
else
    echo -e "${GREEN}  ✓ No settings found${NC}"
fi

# Remove logs
echo -e "${YELLOW}[5/5]${NC} Cleaning logs..."
rm -f /tmp/interview-cracker.log
rm -rf "$HOME/.local/share/interview-cracker"
echo -e "${GREEN}  ✓ Cleaned${NC}"

echo ""
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}✓ Interview Cracker uninstalled${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Note:${NC} Rust and Node.js were NOT removed (shared with other apps)"
echo ""
