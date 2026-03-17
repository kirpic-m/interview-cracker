<script lang="ts">
  interface Props {
    message: string
    type?: 'info' | 'success' | 'error'
  }

  let { message, type = 'info' }: Props = $props()
</script>

<div class="toast {type}">
  <span class="toast-icon">
    {#if type === 'success'}✓
    {:else if type === 'error'}✕
    {:else}ℹ
    {/if}
  </span>
  <span class="toast-message">{message}</span>
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
    animation: slideIn 0.3s ease-out;
    min-width: 200px;
  }

  .toast.success {
    border-color: var(--success);
  }

  .toast.success .toast-icon {
    color: var(--success);
  }

  .toast.error {
    border-color: var(--danger);
  }

  .toast.error .toast-icon {
    color: var(--danger);
  }

  .toast.info .toast-icon {
    color: var(--accent);
  }

  .toast-icon {
    font-size: 1rem;
    font-weight: bold;
  }

  .toast-message {
    font-size: 0.85rem;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
</style>
