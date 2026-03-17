<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { listen } from '@tauri-apps/api/event'
  import { onMount, onDestroy } from 'svelte'

  interface Props {
    settings: {
      provider: string
      model: string
      apiKey: string
      instructions: string
      autoMode: boolean
      hotkey: string
      language: string
    }
    isSessionActive: boolean
    stealthMode: boolean
    onClose: () => void
  }

  let { settings, isSessionActive, stealthMode, onClose }: Props = $props()

  let transcript = $state('')
  let aiResponse = $state('')
  let isProcessing = $state(false)
  let isRecording = $state(false)
  let isAutoMode = $state(false)
  let miniMode = $state(true)
  let autoTranscript = $state('')
  let autoResponses: { question: string; answer: string; time: Date }[] = $state([])
  let autoStats = $state({ questions_detected: 0, responses_given: 0 })

  let unlisteners: (() => void)[] = []

  onMount(async () => {
    const unlisten1 = await listen<string>('auto-transcription', (event) => {
      autoTranscript = event.payload
      transcript = event.payload
    })
    unlisteners.push(unlisten1)

    const unlisten2 = await listen<string>('auto-response', (event) => {
      aiResponse = event.payload
      autoResponses = [{
        question: autoTranscript,
        answer: event.payload,
        time: new Date()
      }, ...autoResponses.slice(0, 9)]
      autoStats.responses_given++
    })
    unlisteners.push(unlisten2)

    const unlisten3 = await listen<string>('auto-question-detected', () => {
      autoStats.questions_detected++
    })
    unlisteners.push(unlisten3)

    const unlisten4 = await listen<string>('auto-error', (event) => {
      aiResponse = `Error: ${event.payload}`
    })
    unlisteners.push(unlisten4)
  })

  onDestroy(() => {
    unlisteners.forEach(fn => fn())
  })

  async function startAutoMode() {
    if (!settings.apiKey) { aiResponse = 'Set API key in Settings'; return }
    try {
      await invoke('start_auto_mode', {
        aiProvider: settings.provider,
        aiModel: settings.model,
        aiApiKey: settings.apiKey,
        transcriptionProvider: settings.provider === 'openrouter-free' || settings.provider === 'openrouter' ? 'openrouter' : 'openai',
        transcriptionApiKey: settings.apiKey,
        instructions: settings.instructions || null,
        language: settings.language || null,
        respondToAll: false,
      })
      isAutoMode = true
      isRecording = true
    } catch (e) { aiResponse = `Failed: ${e}` }
  }

  async function stopAutoMode() {
    try {
      await invoke('stop_auto_mode')
      await invoke('stop_audio_capture')
      isAutoMode = false
      isRecording = false
    } catch (e) { console.error(e) }
  }

  async function startRecording() {
    try {
      await invoke('start_audio_capture', { captureMic: true, captureSystem: false })
      isRecording = true
    } catch (e) { console.error(e) }
  }

  async function stopRecording() {
    try {
      await invoke('stop_audio_capture')
      isRecording = false
    } catch (e) { console.error(e) }
  }

  async function transcribeAndAsk() {
    if (!settings.apiKey) return
    isProcessing = true
    try {
      const response = await invoke<string>('transcribe_and_ask', {
        aiProvider: settings.provider,
        aiModel: settings.model,
        aiApiKey: settings.apiKey,
        transcriptionProvider: 'openrouter',
        transcriptionApiKey: settings.apiKey,
        instructions: settings.instructions || null,
        language: settings.language || null,
      })
      aiResponse = response
    } catch (e) { aiResponse = `Error: ${e}` }
    finally { isProcessing = false }
  }

  async function takeScreenshot() {
    if (!settings.apiKey) return
    isProcessing = true
    aiResponse = 'Analyzing...'
    try {
      const response = await invoke<string>('screenshot_and_analyze', {
        provider: settings.provider,
        model: settings.model,
        apiKey: settings.apiKey,
        prompt: `Interview assistant. Analyze this screenshot and help answer questions. Context: ${settings.instructions || 'None'}`,
      })
      aiResponse = response
    } catch (e) { aiResponse = `Error: ${e}` }
    finally { isProcessing = false }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'q') {
      e.preventDefault()
      if (isRecording && !isAutoMode) transcribeAndAsk()
    }
    if (e.ctrlKey && e.key === 'r') {
      e.preventDefault()
      if (isAutoMode) stopAutoMode()
      else if (isRecording) stopRecording()
      else startRecording()
    }
    if (e.ctrlKey && e.key === 'a') {
      e.preventDefault()
      if (isAutoMode) stopAutoMode()
      else startAutoMode()
    }
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault()
      takeScreenshot()
    }
    if (e.ctrlKey && e.key === 'm') {
      e.preventDefault()
      miniMode = !miniMode
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay" class:mini={miniMode}>
  <div class="overlay-header">
    <span class="dot" class:active={isSessionActive}></span>
    {#if stealthMode}<span class="badge stealth">🥷</span>{/if}
    {#if isAutoMode}<span class="badge auto">🤖</span>{/if}
    {#if isRecording && !isAutoMode}<span class="badge rec">●</span>{/if}
    <span class="spacer"></span>
    <button class="icon-btn" onclick={() => miniMode = !miniMode} title="Toggle size (Ctrl+M)">
      {miniMode ? '□' : '▬'}
    </button>
    <button class="icon-btn close" onclick={onClose}>×</button>
  </div>

  {#if miniMode}
    <!-- MINI WIDGET -->
    <div class="mini-body">
      {#if aiResponse}
        <div class="mini-text">{aiResponse.slice(0, 150)}{aiResponse.length > 150 ? '...' : ''}</div>
      {:else if autoTranscript}
        <div class="mini-text muted">{autoTranscript.slice(0, 100)}...</div>
      {:else}
        <div class="mini-text muted">Ready...</div>
      {/if}
      <div class="mini-bar">
        {#if !isAutoMode && !isRecording}
          <button class="ibtn" onclick={startAutoMode} disabled={!isSessionActive || !settings.apiKey} title="Auto (Ctrl+A)">🤖</button>
          <button class="ibtn" onclick={startRecording} disabled={!isSessionActive} title="Record (Ctrl+R)">🎤</button>
          <button class="ibtn" onclick={takeScreenshot} disabled={!isSessionActive || !settings.apiKey} title="Screenshot (Ctrl+S)">📸</button>
        {:else if isAutoMode}
          <button class="ibtn stop" onclick={stopAutoMode} title="Stop">⏹</button>
          <span class="mini-stats">Q:{autoStats.questions_detected} A:{autoStats.responses_given}</span>
        {:else}
          <button class="ibtn stop" onclick={stopRecording} title="Stop">⏹</button>
          <button class="ibtn" onclick={transcribeAndAsk} disabled={isProcessing} title="Ask (Ctrl+Q)">⚡</button>
        {/if}
      </div>
    </div>
  {:else}
    <!-- FULL MODE -->
    <div class="full-body">
      <div class="controls">
        {#if !isAutoMode && !isRecording}
          <button class="btn auto" onclick={startAutoMode} disabled={!isSessionActive || !settings.apiKey}>🤖 Auto Mode</button>
          <button class="btn rec" onclick={startRecording} disabled={!isSessionActive}>🎤 Record</button>
          <button class="btn shot" onclick={takeScreenshot} disabled={!isSessionActive || !settings.apiKey || isProcessing}>📸 Screenshot</button>
        {:else if isAutoMode}
          <button class="btn stop" onclick={stopAutoMode}>⏹ Stop Auto</button>
          <div class="stats">Q: {autoStats.questions_detected} | A: {autoStats.responses_given}</div>
        {:else}
          <button class="btn stop" onclick={stopRecording}>⏹ Stop</button>
          <button class="btn ask" onclick={transcribeAndAsk} disabled={isProcessing}>⚡ Ask AI</button>
        {/if}
      </div>

      {#if aiResponse}
        <div class="response">{aiResponse}</div>
      {/if}

      {#if autoResponses.length > 0}
        <details class="history">
          <summary>History ({autoResponses.length})</summary>
          {#each autoResponses as qa}
            <div class="qa">
              <div class="q">Q: {qa.question.slice(0, 80)}...</div>
              <div class="a">A: {qa.answer.slice(0, 80)}...</div>
            </div>
          {/each}
        </details>
      {/if}
    </div>
  {/if}
</div>

<style>
  .overlay {
    position: fixed;
    top: 20px;
    right: 20px;
    background: rgba(15, 15, 15, 0.95);
    backdrop-filter: blur(20px);
    border: 1px solid #333;
    border-radius: 12px;
    z-index: 10000;
    overflow: hidden;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    font-size: 13px;
  }

  .overlay.mini {
    width: 280px;
  }

  .overlay:not(.mini) {
    width: 380px;
  }

  .overlay-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-bottom: 1px solid #222;
    cursor: move;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ef4444;
  }
  .dot.active { background: #22c55e; }

  .badge {
    font-size: 11px;
    padding: 1px 4px;
    border-radius: 3px;
  }
  .badge.stealth { background: rgba(124,58,237,0.3); }
  .badge.auto { background: rgba(59,130,246,0.3); }
  .badge.rec { color: #ef4444; animation: blink 1s infinite; }

  @keyframes blink { 0%,100%{opacity:1} 50%{opacity:0.3} }

  .spacer { flex: 1; }

  .icon-btn {
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    padding: 2px 4px;
    font-size: 14px;
    line-height: 1;
  }
  .icon-btn:hover { color: #fff; }
  .icon-btn.close:hover { color: #ef4444; }

  /* MINI MODE */
  .mini-body {
    padding: 8px 10px;
  }

  .mini-text {
    font-size: 12px;
    line-height: 1.4;
    color: #e0e0e0;
    margin-bottom: 8px;
    max-height: 60px;
    overflow-y: auto;
  }
  .mini-text.muted { color: #666; font-style: italic; }

  .mini-bar {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .ibtn {
    padding: 4px 8px;
    background: #252525;
    border: 1px solid #333;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }
  .ibtn:hover { background: #333; }
  .ibtn:disabled { opacity: 0.4; cursor: not-allowed; }
  .ibtn.stop { background: #7f1d1d; border-color: #ef4444; }

  .mini-stats {
    font-size: 11px;
    color: #666;
    margin-left: 4px;
  }

  /* FULL MODE */
  .full-body {
    padding: 10px;
    max-height: 400px;
    overflow-y: auto;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 10px;
  }

  .btn {
    padding: 8px 12px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    text-align: left;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.auto { background: #3b82f6; color: white; }
  .btn.rec { background: #22c55e; color: white; }
  .btn.shot { background: #f59e0b; color: white; }
  .btn.stop { background: #ef4444; color: white; }
  .btn.ask { background: #6366f1; color: white; }

  .stats {
    text-align: center;
    font-size: 12px;
    color: #666;
    padding: 4px;
    background: #1a1a1a;
    border-radius: 6px;
  }

  .response {
    padding: 10px;
    background: #1a1a1a;
    border-radius: 8px;
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    max-height: 200px;
    overflow-y: auto;
  }

  .history {
    margin-top: 10px;
  }
  .history summary {
    cursor: pointer;
    font-size: 12px;
    color: #666;
  }
  .qa {
    padding: 6px;
    background: #1a1a1a;
    border-radius: 6px;
    margin-top: 4px;
    font-size: 11px;
  }
  .q { color: #6366f1; }
  .a { color: #888; margin-top: 2px; }
</style>
