# 🔥 Interview Cracker

AI-powered interview assistant with real-time answers. Crack any interview!

## ✨ Features

- 🤖 **AI Chat** — 5 providers (OpenRouter, OpenAI, Google, DeepSeek, xAI)
- 🎤 **Audio Capture** — microphone + system audio
- 🗣️ **Transcription** — Whisper, Google Speech
- 🤖 **Auto Mode** — AI detects questions automatically
- 🥷 **Stealth Mode** — hidden from screen sharing
- 📸 **Screenshot Analysis** — Vision AI for code/questions
- 📄 **Documents** — upload resume, job description
- 💾 **History** — all sessions saved locally
- ⌨️ **Hotkeys** — Ctrl+A, Ctrl+R, Ctrl+Q, Ctrl+S

## 🚀 Installation

### One-Line Install (Recommended)

**Linux/macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/kirpic-m/interview-cracker/main/quick-install.sh | bash
```

**Windows (PowerShell as Admin):**
```powershell
irm https://raw.githubusercontent.com/kirpic-m/interview-cracker/main/quick-install.ps1 | iex
```

### Manual Install

**Linux:**
```bash
git clone https://github.com/kirpic-m/interview-cracker.git
cd interview-cracker
bash install.sh
```

**Windows:**
```powershell
git clone https://github.com/kirpic-m/interview-cracker.git
cd interview-cracker
.\install.ps1
```

### Requirements
- Node.js 18+
- Rust
- System deps: WebKit GTK, GTK3 (Linux)

## 🎮 Usage

1. **Launch** — click desktop icon or run `interview-cracker`
2. **Set API key** — OpenRouter recommended (free models available)
3. **Start session** — click "Start Session"
4. **Use features:**
   - Press `Ctrl+A` for Auto Mode (AI listens)
   - Press `Ctrl+R` for Manual Recording
   - Press `Ctrl+Q` to Transcribe & Ask
   - Press `Ctrl+S` to Screenshot & Analyze

## 🔑 API Keys

Get free API keys:
- [OpenRouter](https://openrouter.ai) — has free models!
- [OpenAI](https://platform.openai.com)
- [Google AI](https://aistudio.google.com)
- [DeepSeek](https://platform.deepseek.com)
- [xAI](https://console.x.ai)

## ⌨️ Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+A` | Toggle Auto Mode |
| `Ctrl+R` | Start/Stop Recording |
| `Ctrl+Q` | Transcribe & Ask |
| `Ctrl+S` | Screenshot & Analyze |
| `Ctrl+M` | Mini/Full mode toggle |
| `?` | Show keyboard help |

## 🛠️ Development

```bash
# Install
npm install
cd src-tauri && cargo build && cd ..

# Run dev
npm run tauri dev

# Build release
npm run tauri build
```

## 📄 License

MIT License
Copyright (c) 2026 kirpic-m
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND...

## 🤝 Contributing

PRs welcome!
