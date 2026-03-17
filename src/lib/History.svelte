<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'

  interface Session {
    id: string
    title: string
    instructions: string | null
    created_at: string
    ended_at: string | null
  }

  interface Message {
    id: number
    session_id: string
    role: string
    content: string
    created_at: string
  }

  let sessions: Session[] = $state([])
  let selectedSession: Session | null = $state(null)
  let messages: Message[] = $state([])

  async function loadSessions() {
    try {
      sessions = await invoke<Session[]>('get_sessions')
    } catch (e) {
      console.error('Failed to load sessions:', e)
    }
  }

  async function selectSession(session: Session) {
    selectedSession = session
    try {
      messages = await invoke<Message[]>('get_session_messages', {
        sessionId: session.id,
      })
    } catch (e) {
      console.error('Failed to load messages:', e)
    }
  }

  $effect(() => {
    loadSessions()
  })
</script>

<div class="history">
  <div class="sidebar">
    <h3>Sessions</h3>
    {#if sessions.length === 0}
      <p class="empty">No sessions yet</p>
    {:else}
      {#each sessions as session}
        <button
          class="session-item"
          class:active={selectedSession?.id === session.id}
          onclick={() => selectSession(session)}
        >
          <span class="session-title">{session.title}</span>
          <span class="session-date">
            {new Date(session.created_at).toLocaleDateString()}
          </span>
        </button>
      {/each}
    {/if}
  </div>

  <div class="content">
    {#if selectedSession}
      <div class="session-header">
        <h2>{selectedSession.title}</h2>
        <span class="session-meta">
          {new Date(selectedSession.created_at).toLocaleString()}
        </span>
      </div>

      <div class="messages">
        {#each messages as message}
          <div class="message {message.role}">
            <div class="msg-role">{message.role}</div>
            <div class="msg-content">{message.content}</div>
            <div class="msg-time">{new Date(message.created_at).toLocaleTimeString()}</div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <p>Select a session to view history</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .history {
    display: flex;
    height: 100%;
  }

  .sidebar {
    width: 250px;
    border-right: 1px solid var(--border);
    padding: 1rem;
    overflow-y: auto;
    background: var(--bg-secondary);
  }

  .sidebar h3 {
    margin: 0 0 1rem 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .empty {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .session-item {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 0.75rem;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
    margin-bottom: 0.5rem;
  }

  .session-item:hover {
    background: var(--bg-tertiary);
  }

  .session-item.active {
    background: var(--accent);
    border-color: var(--accent);
  }

  .session-title {
    font-weight: 500;
    font-size: 0.9rem;
  }

  .session-date {
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-top: 0.25rem;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .session-header {
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
  }

  .session-header h2 {
    margin: 0;
    font-size: 1.2rem;
  }

  .session-meta {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .messages {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .message {
    padding: 0.75rem 1rem;
    border-radius: 8px;
    background: var(--bg-tertiary);
  }

  .message.user {
    background: rgba(99, 102, 241, 0.2);
  }

  .msg-role {
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-bottom: 0.25rem;
  }

  .msg-content {
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .msg-time {
    font-size: 0.7rem;
    color: var(--text-secondary);
    margin-top: 0.5rem;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }
</style>
