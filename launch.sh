#!/bin/bash
cd /home/roman/interview-hunter
source "$HOME/.cargo/env" 2>/dev/null
fuser -k 5173/tcp 2>/dev/null
export PATH="$HOME/.cargo/bin:$PATH"
nohup npm run tauri dev > /tmp/interview-cracker.log 2>&1 &
