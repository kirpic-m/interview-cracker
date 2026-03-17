<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'

  interface Settings {
    provider: string
    model: string
    apiKey: string
    instructions: string
    autoMode: boolean
    hotkey: string
    language: string
  }

  interface Props {
    settings: Settings
    onSave?: () => void
  }

  let { settings = $bindable(), onSave }: Props = $props()

  interface Provider {
    id: string
    name: string
    models: string[]
  }

  interface Document {
    id: string
    name: string
    doc_type: string
    content: string
    file_path: string | null
    uploaded_at: string
  }

  let providers: Provider[] = $state([])
  let selectedProvider = $derived(providers.find(p => p.id === settings.provider))
  let documents: Document[] = $state([])
  let newTextName = $state('')
  let newTextType = $state('resume')
  let newTextContent = $state('')
  let showTextForm = $state(false)

  async function loadProviders() {
    try {
      providers = await invoke<Provider[]>('get_providers')
    } catch (e) {
      console.error('Failed to load providers:', e)
    }
  }

  async function loadDocuments() {
    try {
      documents = await invoke<Document[]>('get_documents')
    } catch (e) {
      console.error('Failed to load documents:', e)
    }
  }

  $effect(() => {
    loadProviders()
    loadDocuments()
  })

  function onProviderChange(e: Event) {
    const target = e.target as HTMLSelectElement
    settings.provider = target.value
    const provider = providers.find(p => p.id === target.value)
    if (provider && provider.models.length > 0) {
      settings.model = provider.models[0]
    }
  }

  async function uploadFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Documents',
          extensions: ['txt', 'md', 'pdf', 'doc', 'docx', 'json']
        }]
      })

      if (selected) {
        const doc = await invoke<Document>('upload_document', {
          filePath: selected,
          docType: newTextType,
        })
        documents = [...documents, doc]
      }
    } catch (e) {
      console.error('Failed to upload document:', e)
    }
  }

  async function addTextDocument() {
    if (!newTextName.trim() || !newTextContent.trim()) return

    try {
      const doc = await invoke<Document>('add_text_document', {
        name: newTextName,
        docType: newTextType,
        content: newTextContent,
      })
      documents = [...documents, doc]
      newTextName = ''
      newTextContent = ''
      showTextForm = false
    } catch (e) {
      console.error('Failed to add document:', e)
    }
  }

  async function removeDocument(docId: string) {
    try {
      await invoke('remove_document', { docId })
      documents = documents.filter(d => d.id !== docId)
    } catch (e) {
      console.error('Failed to remove document:', e)
    }
  }
</script>

<div class="settings">
  <div class="settings-group">
    <h3>AI Provider</h3>
    <label>
      <span>Provider</span>
      <select value={settings.provider} onchange={onProviderChange}>
        {#each providers as provider}
          <option value={provider.id}>{provider.name}</option>
        {/each}
      </select>
    </label>

    <label>
      <span>Model</span>
      <select bind:value={settings.model}>
        {#if selectedProvider}
          {#each selectedProvider.models as model}
            <option value={model}>{model}</option>
          {/each}
        {/if}
      </select>
    </label>

    <label>
      <span>API Key</span>
      <div class="api-key-row">
        <input
          type="password"
          bind:value={settings.apiKey}
          placeholder="Enter your API key"
        />
        <button class="btn-save-key" onclick={() => onSave?.()} title="Save settings">
          💾
        </button>
      </div>
    </label>
  </div>

  <div class="settings-group">
    <h3>Documents</h3>
    <p class="doc-info">Upload your resume, job description, or other documents to give AI more context.</p>

    <div class="doc-actions">
      <button class="btn-upload" onclick={uploadFile}>
        📄 Upload File
      </button>
      <button class="btn-text" onclick={() => showTextForm = !showTextForm}>
        📝 Paste Text
      </button>
    </div>

    {#if showTextForm}
      <div class="text-form">
        <input
          type="text"
          bind:value={newTextName}
          placeholder="Document name (e.g., My Resume)"
        />
        <select bind:value={newTextType}>
          <option value="resume">Resume / CV</option>
          <option value="job">Job Description</option>
          <option value="tech">Technical Document</option>
          <option value="other">Other</option>
        </select>
        <textarea
          bind:value={newTextContent}
          placeholder="Paste document content here..."
          rows="6"
        ></textarea>
        <div class="form-actions">
          <button class="btn-save" onclick={addTextDocument}>Save</button>
          <button class="btn-cancel" onclick={() => showTextForm = false}>Cancel</button>
        </div>
      </div>
    {/if}

    {#if documents.length > 0}
      <div class="doc-list">
        {#each documents as doc}
          <div class="doc-item">
            <div class="doc-info-row">
              <span class="doc-icon">
                {#if doc.doc_type === 'Resume'}📄
                {:else if doc.doc_type === 'Job Description'}💼
                {:else}📎
                {/if}
              </span>
              <div class="doc-details">
                <span class="doc-name">{doc.name}</span>
                <span class="doc-type">{doc.doc_type}</span>
              </div>
              <button class="btn-remove" onclick={() => removeDocument(doc.id)}>×</button>
            </div>
            <div class="doc-preview">{doc.content.slice(0, 100)}...</div>
          </div>
        {/each}
      </div>
    {:else}
      <p class="no-docs">No documents uploaded yet</p>
    {/if}
  </div>

  <div class="settings-group">
    <h3>Behavior</h3>
    <label class="checkbox">
      <input type="checkbox" bind:checked={settings.autoMode} />
      <span>Auto Mode (AI detects when to answer)</span>
    </label>

    <label>
      <span>Hotkey</span>
      <input
        type="text"
        bind:value={settings.hotkey}
        placeholder="Ctrl+Q"
      />
    </label>

    <label>
      <span>Language</span>
      <select bind:value={settings.language}>
        <option value="en">English</option>
        <option value="uk">Українська</option>
        <option value="ru">Русский</option>
        <option value="de">Deutsch</option>
        <option value="fr">Français</option>
        <option value="es">Español</option>
      </select>
    </label>
  </div>

  <div class="settings-group">
    <h3>Custom Instructions</h3>
    <textarea
      bind:value={settings.instructions}
      placeholder="Set your role and context for the AI...

Example:
- I am a software engineer interviewing for a senior backend position
- Focus on Go, distributed systems, and system design
- Keep answers concise with code examples"
      rows="6"
    ></textarea>
  </div>

  <div class="settings-group">
    <h3>About</h3>
    <p class="about-text">
      Interview Cracker v0.2.0 — AI-powered interview assistant
    </p>
    <p class="about-text">
      Supports: OpenRouter, OpenAI, Google AI, NVIDIA NIM
    </p>
    <p class="about-text">
      Press <kbd>?</kbd> for keyboard shortcuts
    </p>
  </div>
</div>

<style>
  .settings {
    padding: 1.5rem;
    overflow-y: auto;
    max-width: 600px;
    margin: 0 auto;
  }

  .settings-group {
    margin-bottom: 2rem;
  }

  .settings-group h3 {
    margin: 0 0 1rem 0;
    color: var(--text-secondary);
    text-transform: uppercase;
    font-size: 0.8rem;
    letter-spacing: 0.05em;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-bottom: 1rem;
  }

  label span {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  input[type="text"],
  input[type="password"],
  select,
  textarea {
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-family: inherit;
    font-size: 0.9rem;
  }

  input:focus,
  select:focus,
  textarea:focus {
    outline: none;
    border-color: var(--accent);
  }

  textarea {
    resize: vertical;
    min-height: 120px;
  }

  .checkbox {
    flex-direction: row;
    align-items: center;
    gap: 0.75rem;
  }

  .checkbox input {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }

  .doc-info {
    color: var(--text-secondary);
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }

  .doc-actions {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .btn-upload, .btn-text {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .btn-upload:hover, .btn-text:hover {
    background: var(--border);
  }

  .text-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
    background: var(--bg-tertiary);
    border-radius: 8px;
    margin-bottom: 1rem;
  }

  .form-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-save {
    flex: 1;
    padding: 0.5rem;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  .btn-cancel {
    flex: 1;
    padding: 0.5rem;
    background: var(--border);
    color: var(--text-primary);
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  .doc-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .doc-item {
    padding: 0.75rem;
    background: var(--bg-tertiary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .doc-info-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .doc-icon {
    font-size: 1.2rem;
  }

  .doc-details {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .doc-name {
    font-weight: 500;
    font-size: 0.9rem;
  }

  .doc-type {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .btn-remove {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1.2rem;
    padding: 0.25rem;
  }

  .btn-remove:hover {
    color: var(--danger);
  }

  .doc-preview {
    margin-top: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-docs {
    color: var(--text-secondary);
    font-size: 0.85rem;
    text-align: center;
    padding: 1rem;
    background: var(--bg-tertiary);
    border-radius: 8px;
  }

  .about-text {
    color: var(--text-secondary);
    font-size: 0.85rem;
    margin-bottom: 0.25rem;
  }

  .api-key-row {
    display: flex;
    gap: 0.5rem;
  }

  .api-key-row input {
    flex: 1;
  }

  .btn-save-key {
    padding: 0.75rem;
    background: var(--accent);
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1rem;
  }

  .btn-save-key:hover {
    filter: brightness(1.1);
  }
</style>
