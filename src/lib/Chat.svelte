<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'

  interface Props {
    isSessionActive: boolean
    settings: {
      provider: string
      model: string
      apiKey: string
      instructions: string
      autoMode: boolean
      hotkey: string
      language: string
    }
  }

  let { isSessionActive, settings }: Props = $props()

  interface Message {
    role: 'user' | 'assistant' | 'system'
    content: string
    timestamp: Date
  }

  let messages: Message[] = $state([])
  let inputText = $state('')
  let isLoading = $state(false)

  async function sendMessage() {
    if (!inputText.trim() || !isSessionActive || !settings.apiKey) return

    const userMessage: Message = {
      role: 'user',
      content: inputText.trim(),
      timestamp: new Date(),
    }
    messages = [...messages, userMessage]
    inputText = ''
    isLoading = true

    try {
      const response = await invoke<string>('ask_ai', {
        provider: settings.provider,
        model: settings.model,
        apiKey: settings.apiKey,
        message: userMessage.content,
        instructions: settings.instructions || null,
        contextDocuments: [],
      })

      const assistantMessage: Message = {
        role: 'assistant',
        content: response,
        timestamp: new Date(),
      }
      messages = [...messages, assistantMessage]

      await invoke('save_message', {
        sessionId: 'current',
        role: 'user',
        content: userMessage.content,
      })
      await invoke('save_message', {
        sessionId: 'current',
        role: 'assistant',
        content: response,
      })
    } catch (error) {
      const errorMessage: Message = {
        role: 'system',
        content: `Error: ${error}`,
        timestamp: new Date(),
      }
      messages = [...messages, errorMessage]
    } finally {
      isLoading = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }
</script>

<div class="chat">
  <div class="messages">
    {#if !isSessionActive}
      <div class="empty-state">
        <p>Start a session to begin chatting with AI</p>
      </div>
    {:else if messages.length === 0}
      <div class="empty-state">
        <p>Type a message or ask a question during your interview</p>
        <p class="hint">AI will provide real-time answers and tips</p>
      </div>
    {:else}
      {#each messages as message}
        <div class="message {message.role}">
          <div class="message-header">
            <span class="role">{message.role}</span>
            <span class="time">{message.timestamp.toLocaleTimeString()}</span>
          </div>
          <div class="message-content">{message.content}</div>
        </div>
      {/each}
    {/if}

    {#if isLoading}
      <div class="message assistant loading">
        <div class="typing-indicator">
          <span></span><span></span><span></span>
        </div>
      </div>
    {/if}
  </div>

  <div class="input-area">
    <textarea
      bind:value={inputText}
      onkeydown={handleKeydown}
      placeholder={isSessionActive ? 'Type your question...' : 'Start a session first'}
      disabled={!isSessionActive}
      rows="2"
    ></textarea>
    <button
      class="send-btn"
      onclick={sendMessage}
      disabled={!isSessionActive || !inputText.trim() || isLoading}
    >
      Send
    </button>
  </div>
</div>

<style>
  .chat {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
    text-align: center;
  }

  .hint {
    font-size: 0.85rem;
    margin-top: 0.5rem;
    opacity: 0.7;
  }

  .message {
    padding: 0.75rem 1rem;
    border-radius: 12px;
    max-width: 85%;
  }

  .message.user {
    background: var(--accent);
    align-self: flex-end;
  }

  .message.assistant {
    background: var(--bg-tertiary);
    align-self: flex-start;
  }

  .message.system {
    background: rgba(239, 68, 68, 0.2);
    align-self: center;
    border: 1px solid var(--danger);
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.25rem;
    font-size: 0.75rem;
    opacity: 0.7;
  }

  .message-content {
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .input-area {
    display: flex;
    gap: 0.5rem;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border);
    background: var(--bg-secondary);
  }

  textarea {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    resize: none;
    font-family: inherit;
    font-size: 0.9rem;
  }

  textarea:focus {
    outline: none;
    border-color: var(--accent);
  }

  .send-btn {
    padding: 0.75rem 1.5rem;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 500;
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .typing-indicator {
    display: flex;
    gap: 4px;
    padding: 4px 0;
  }

  .typing-indicator span {
    width: 8px;
    height: 8px;
    background: var(--text-secondary);
    border-radius: 50%;
    animation: bounce 1.4s infinite ease-in-out both;
  }

  .typing-indicator span:nth-child(1) { animation-delay: -0.32s; }
  .typing-indicator span:nth-child(2) { animation-delay: -0.16s; }

  @keyframes bounce {
    0%, 80%, 100% { transform: scale(0); }
    40% { transform: scale(1); }
  }
</style>
