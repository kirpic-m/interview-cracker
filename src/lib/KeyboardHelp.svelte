<script lang="ts">
  interface Props {
    onClose: () => void
  }

  let { onClose }: Props = $props()

  const shortcuts = [
    { key: 'Ctrl+A', description: 'Toggle Auto Mode' },
    { key: 'Ctrl+R', description: 'Start/Stop Manual Recording' },
    { key: 'Ctrl+Q', description: 'Transcribe & Ask AI' },
    { key: 'Ctrl+S', description: 'Screenshot & Analyze' },
    { key: '?', description: 'Show/Hide this help' },
    { key: 'Esc', description: 'Close dialogs' },
  ]
</script>

<div class="keyboard-help-overlay" onclick={onClose} role="dialog">
  <div class="keyboard-help" onclick={(e) => e.stopPropagation()} role="document">
    <div class="help-header">
      <h3>⌨️ Keyboard Shortcuts</h3>
      <button class="close-btn" onclick={onClose}>×</button>
    </div>
    <div class="shortcuts-list">
      {#each shortcuts as shortcut}
        <div class="shortcut-item">
          <kbd class="shortcut-key">{shortcut.key}</kbd>
          <span class="shortcut-desc">{shortcut.description}</span>
        </div>
      {/each}
    </div>
    <div class="help-footer">
      Press <kbd>?</kbd> to toggle this help
    </div>
  </div>
</div>

<style>
  .keyboard-help-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
    animation: fadeIn 0.2s ease-out;
  }

  .keyboard-help {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 1.5rem;
    min-width: 320px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }

  .help-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .help-header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem;
    background: var(--bg-tertiary);
    border-radius: 8px;
  }

  .shortcut-key {
    padding: 0.25rem 0.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.8rem;
    color: var(--accent);
  }

  .shortcut-desc {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .help-footer {
    margin-top: 1rem;
    text-align: center;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .help-footer kbd {
    padding: 0.15rem 0.4rem;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: inherit;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
