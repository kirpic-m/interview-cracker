#!/bin/bash
pkill -f interview-cracker 2>/dev/null; sleep 1
cd /home/roman/interview-cracker
exec ./src-tauri/target/release/interview-cracker "$@"
