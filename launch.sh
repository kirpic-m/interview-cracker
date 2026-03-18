#!/bin/bash
cd /home/roman/interview-cracker

# Clean ports
fuser -k 5173/tcp 2>/dev/null
fuser -k 1420/tcp 2>/dev/null
pkill -f "interview-cracker" 2>/dev/null
sleep 1

source "$HOME/.cargo/env" 2>/dev/null
export PATH="$HOME/.cargo/bin:$PATH"

if [ -f "src-tauri/target/release/interview-cracker" ]; then
    exec src-tauri/target/release/interview-cracker "$@"
else
    npm run tauri dev 2>&1
fi
