<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { listen } from '@tauri-apps/api/event'
  import { onMount, onDestroy } from 'svelte'
  import Chat from './lib/Chat.svelte'
  import Settings from './lib/Settings.svelte'
  import History from './lib/History.svelte'
  import Overlay from './lib/Overlay.svelte'
  import Toast from './lib/Toast.svelte'
  import KeyboardHelp from './lib/KeyboardHelp.svelte'

  type View = 'chat' | 'settings' | 'history'

  let currentView: View = $state('chat')
  let isSessionActive = $state(false)
  let showOverlay = $state(false)
  let stealthMode = $state(false)
  let stealthSupported = $state(true)
  let showKeyboardHelp = $state(false)

  // Timer
  let sessionStartTime = $state<Date | null>(null)
  let elapsedTime = $state('00:00:00')
  let timerInterval: ReturnType<typeof setInterval> | null = null

  // Toasts
  let toasts: { id: number; message: string; type: 'info' | 'success' | 'error' }[] = $state([])
  let toastId = 0

  let settings = $state({
    provider: 'openrouter-free',
    model: 'stepfun/step-3.5-flash:free',
    apiKey: '',
    instructions: '',
    autoMode: true,
    hotkey: 'Ctrl+Q',
    language: 'en',
  })

  async function checkStealthSupport() {
    try {
      stealthSupported = await invoke<boolean>('is_screen_capture_hiding_supported')
    } catch {
      stealthSupported = false
    }
  }

  function showToast(message: string, type: 'info' | 'success' | 'error' = 'info') {
    const id = ++toastId
    toasts = [...toasts, { id, message, type }]
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id)
    }, 3000)
  }

  function updateTimer() {
    if (sessionStartTime) {
      const now = new Date()
      const diff = Math.floor((now.getTime() - sessionStartTime.getTime()) / 1000)
      const hours = Math.floor(diff / 3600).toString().padStart(2, '0')
      const minutes = Math.floor((diff % 3600) / 60).toString().padStart(2, '0')
      const seconds = (diff % 60).toString().padStart(2, '0')
      elapsedTime = `${hours}:${minutes}:${seconds}`
    }
  }

  $effect(() => {
    checkStealthSupport()

    // Listen for global keyboard shortcut events
    const setupListeners = async () => {
      await listen('show-keyboard-help', () => {
        showKeyboardHelp = !showKeyboardHelp
      })
    }
    setupListeners()
  })

  // Load settings on mount
  onMount(async () => {
    try {
      const saved = await invoke<any>('load_app_settings')
      if (saved) {
        settings.provider = saved.provider || 'openrouter-free'
        settings.model = saved.model || 'stepfun/step-3.5-flash:free'
        settings.instructions = saved.instructions || ''
        settings.autoMode = saved.auto_mode ?? true
        settings.language = saved.language || 'en'
        
        // Load API key for current provider
        const keyMap: Record<string, string> = {
          'openrouter': saved.api_keys?.openrouter || '',
          'openrouter-free': saved.api_keys?.openrouter || '',
          'openai': saved.api_keys?.openai || '',
          'google': saved.api_keys?.google || '',
          'deepseek': saved.api_keys?.deepseek || '',
          'xai': saved.api_keys?.xai || '',
        }
        settings.apiKey = keyMap[settings.provider] || ''
        
        showToast('Settings loaded', 'success')
      }
    } catch (e) {
      console.log('No saved settings found')
    }
  })

  // Save settings when changed
  async function saveSettings() {
    try {
      const apiKeys: any = {}
      if (settings.provider.includes('openrouter')) {
        apiKeys.openrouter = settings.apiKey
      } else {
        apiKeys[settings.provider] = settings.apiKey
      }

      await invoke('save_app_settings', {
        appSettings: {
          provider: settings.provider,
          model: settings.model,
          api_keys: apiKeys,
          instructions: settings.instructions,
          auto_mode: settings.autoMode,
          language: settings.language,
          mini_mode: true,
        }
      })
    } catch (e) {
      console.error('Failed to save settings:', e)
    }
  }

  function handleStartSession() {
    isSessionActive = true
    sessionStartTime = new Date()
    timerInterval = setInterval(updateTimer, 1000)
    showToast('Session started! Good luck! 🍀', 'success')
  }

  function handleEndSession() {
    isSessionActive = false
    sessionStartTime = null
    elapsedTime = '00:00:00'
    if (timerInterval) {
      clearInterval(timerInterval)
      timerInterval = null
    }
    showToast('Session ended', 'info')
  }

  function toggleOverlay() {
    showOverlay = !showOverlay
  }

  async function toggleStealth() {
    try {
      if (!stealthMode) {
        await invoke('enable_stealth_mode')
        stealthMode = true
        showToast('Stealth mode enabled 🥷', 'success')
      } else {
        await invoke('disable_stealth_mode')
        stealthMode = false
        showToast('Stealth mode disabled', 'info')
      }
    } catch (e) {
      console.error('Failed to toggle stealth mode:', e)
      showToast('Failed to toggle stealth mode', 'error')
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === '?' && !e.ctrlKey && !e.metaKey) {
      const target = e.target as HTMLElement
      if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
        showKeyboardHelp = !showKeyboardHelp
      }
    }
    if (e.key === 'Escape') {
      showKeyboardHelp = false
    }
  }

  onDestroy(() => {
    if (timerInterval) {
      clearInterval(timerInterval)
    }
  })
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app">
  <header class="header">
    <div class="logo">
      <span class="logo-icon">🔥</span>
      <h1>Interview Cracker</h1>
    </div>
    <nav class="nav">
      <button
        class="nav-btn"
        class:active={currentView === 'chat'}
        onclick={() => currentView = 'chat'}
      >
        Chat
      </button>
      <button
        class="nav-btn"
        class:active={currentView === 'history'}
        onclick={() => currentView = 'history'}
      >
        History
      </button>
      <button
        class="nav-btn"
        class:active={currentView === 'settings'}
        onclick={() => currentView = 'settings'}
      >
        Settings
      </button>
    </nav>
    <div class="header-center">
      {#if isSessionActive}
        <div class="timer">
          <span class="timer-icon">⏱️</span>
          <span class="timer-value">{elapsedTime}</span>
        </div>
      {/if}
    </div>
    <div class="header-actions">
      <button
        class="btn btn-help"
        onclick={() => showKeyboardHelp = !showKeyboardHelp}
        title="Keyboard shortcuts (?)"
      >
        ⌨️
      </button>
      {#if stealthSupported}
        <button
          class="btn"
          class:btn-stealth-active={stealthMode}
          class:btn-secondary={!stealthMode}
          onclick={toggleStealth}
          title={stealthMode ? 'Window is hidden from screen capture' : 'Hide window from screen capture'}
        >
          {stealthMode ? '🥷 Stealth' : '👁 Visible'}
        </button>
      {/if}
      {#if isSessionActive}
        <button class="btn btn-danger" onclick={handleEndSession}>
          ⏹ End Session
        </button>
      {:else}
        <button class="btn btn-primary" onclick={handleStartSession}>
          ▶ Start Session
        </button>
      {/if}
      <button class="btn btn-secondary" onclick={toggleOverlay}>
        {showOverlay ? '✕ Hide' : '◫ Overlay'}
      </button>
    </div>
  </header>

  <main class="main">
    {#if currentView === 'chat'}
      <Chat {isSessionActive} {settings} />
    {:else if currentView === 'settings'}
      <Settings bind:settings onSave={saveSettings} />
    {:else if currentView === 'history'}
      <History />
    {/if}
  </main>

  {#if showOverlay}
    <Overlay {settings} {isSessionActive} {stealthMode} onClose={toggleOverlay} />
  {/if}

  {#if showKeyboardHelp}
    <KeyboardHelp onClose={() => showKeyboardHelp = false} />
  {/if}

  <!-- Toast notifications -->
  <div class="toast-container">
    {#each toasts as toast (toast.id)}
      <Toast message={toast.message} type={toast.type} />
    {/each}
  </div>
</div>

<style>
  :root {
    --bg-primary: #0a0a0a;
    --bg-secondary: #141414;
    --bg-tertiary: #1f1f1f;
    --text-primary: #e5e5e5;
    --text-secondary: #737373;
    --accent: #6366f1;
    --accent-hover: #818cf8;
    --danger: #ef4444;
    --success: #22c55e;
    --warning: #f59e0b;
    --border: #262626;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  :global(*) {
    box-sizing: border-box;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 1rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 1rem;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .logo-icon {
    font-size: 1.3rem;
  }

  .logo h1 {
    font-size: 1rem;
    margin: 0;
    font-weight: 700;
    background: linear-gradient(135deg, #f59e0b, #ef4444);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .nav {
    display: flex;
    gap: 0.25rem;
  }

  .nav-btn {
    padding: 0.4rem 0.75rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 6px;
    font-size: 0.85rem;
    transition: all 0.2s;
  }

  .nav-btn:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  .nav-btn.active {
    color: var(--text-primary);
    background: var(--accent);
  }

  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .timer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.75rem;
    background: var(--bg-tertiary);
    border-radius: 20px;
    border: 1px solid var(--border);
  }

  .timer-icon {
    font-size: 0.9rem;
  }

  .timer-value {
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--success);
  }

  .header-actions {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }

  .btn {
    padding: 0.4rem 0.75rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 500;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .btn-help {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    padding: 0.4rem 0.6rem;
  }

  .btn-help:hover {
    color: var(--text-primary);
    background: var(--border);
  }

  .btn-primary {
    background: var(--success);
    color: white;
  }

  .btn-primary:hover {
    filter: brightness(1.1);
  }

  .btn-danger {
    background: var(--danger);
    color: white;
  }

  .btn-danger:hover {
    filter: brightness(1.1);
  }

  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover {
    background: var(--border);
  }

  .btn-stealth-active {
    background: #7c3aed;
    color: white;
    border: 1px solid #8b5cf6;
    animation: stealth-pulse 2s infinite;
  }

  @keyframes stealth-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(124, 58, 237, 0.4); }
    50% { box-shadow: 0 0 0 4px rgba(124, 58, 237, 0); }
  }

  .main {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .toast-container {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 9999;
  }
</style>
