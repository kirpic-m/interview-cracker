#!/bin/bash
# Interview Cracker Launcher
cd "$(dirname "$(readlink -f "$0")")"

# Kill old processes
fuser -k 5173/tcp 2>/dev/null

# Source Rust env
source "$HOME/.cargo/env" 2>/dev/null

# Run in dev mode (shows terminal)
konsole -e bash -c "cd '$PWD' && npm run tauri dev; read -p 'Press Enter to close...'" 2>/dev/null ||
gnome-terminal -- bash -c "cd '$PWD' && npm run tauri dev; read -p 'Press Enter to close...'" 2>/dev/null ||
xterm -e "cd '$PWD' && npm run tauri dev; read -p 'Press Enter'" 2>/dev/null ||
alacritty -e bash -c "cd '$PWD' && npm run tauri dev; read -p 'Press Enter'" 2>/dev/null ||
# Fallback: run in background
nohup bash -c "cd '$PWD' && npm run tauri dev" > /tmp/interview-cracker.log 2>&1 &
