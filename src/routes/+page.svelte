<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { onMount } from "svelte";

  let inputText = $state("");
  let outputText = $state("");
  let isProcessing = $state(false);
  let ollamaConnected = $state(false);
  let copied = $state(false);

  onMount(async () => {
    ollamaConnected = await invoke<boolean>("check_ollama");
  });

  async function handleRedact() {
    if (!inputText.trim() || isProcessing) return;
    isProcessing = true;
    try {
      outputText = await invoke<string>("redact_text", { text: inputText });
    } catch (e) {
      outputText = `Error: ${e}`;
    } finally {
      isProcessing = false;
    }
  }

  async function copyOutput() {
    if (!outputText) return;
    try {
      await writeText(outputText);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }
</script>

<main class="app">
  <header>
    <h1>De-Id</h1>
    <div class="status" title={ollamaConnected ? "Ollama connected" : "Ollama unavailable — name detection disabled"}>
      <span class="status-dot" class:connected={ollamaConnected} class:disconnected={!ollamaConnected}></span>
      <span class="status-label">{ollamaConnected ? "Ollama connected" : "Ollama unavailable"}</span>
    </div>
  </header>

  <div class="panels">
    <div class="panel">
      <label for="input">Input</label>
      <textarea
        id="input"
        bind:value={inputText}
        placeholder="Paste text containing PII here..."
      ></textarea>
    </div>

    <div class="controls">
      <button
        class="redact-btn"
        onclick={handleRedact}
        disabled={!inputText.trim() || isProcessing}
      >
        {#if isProcessing}
          <span class="spinner"></span>
          Processing...
        {:else}
          De-Id
        {/if}
      </button>
      {#if !ollamaConnected}
        <p class="note">Regex-only mode (emails)</p>
      {/if}
    </div>

    <div class="panel">
      <label for="output">Output <span class="hint">(click to copy)</span></label>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        id="output"
        class="output-area"
        class:has-content={!!outputText}
        onclick={copyOutput}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyOutput(); }}
        role="button"
        tabindex="0"
      >
        {#if outputText}
          {outputText}
        {:else}
          <span class="placeholder">Redacted text will appear here...</span>
        {/if}
        {#if copied}
          <div class="toast">Copied!</div>
        {/if}
      </div>
    </div>
  </div>
</main>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(:root) {
    --bg: #f8f9fa;
    --surface: #ffffff;
    --text: #1a1a2e;
    --text-muted: #6c757d;
    --border: #dee2e6;
    --primary: #2d3436;
    --primary-hover: #1a1a2e;
    --accent: #0984e3;
    --shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.1);
    --radius: 8px;
    --transition: 0.2s ease;

    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 15px;
    line-height: 1.5;
    color: var(--text);
    background: var(--bg);
    -webkit-font-smoothing: antialiased;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 1.5rem;
    gap: 1.25rem;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--primary);
  }

  .status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-dot.connected {
    background: #00b894;
    box-shadow: 0 0 6px rgba(0, 184, 148, 0.4);
  }

  .status-dot.disconnected {
    background: #d63031;
    box-shadow: 0 0 6px rgba(214, 48, 49, 0.4);
  }

  .panels {
    display: flex;
    gap: 1rem;
    flex: 1;
    min-height: 0;
  }

  .panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }

  label {
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  .hint {
    font-weight: 400;
    text-transform: none;
    letter-spacing: normal;
  }

  textarea {
    flex: 1;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    font-family: "SF Mono", "Fira Code", "Cascadia Code", monospace;
    font-size: 0.9rem;
    line-height: 1.6;
    resize: none;
    outline: none;
    box-shadow: var(--shadow);
    transition: border-color var(--transition), box-shadow var(--transition);
  }

  textarea:focus {
    border-color: var(--accent);
    box-shadow: var(--shadow), 0 0 0 3px rgba(9, 132, 227, 0.1);
  }

  textarea::placeholder {
    color: var(--text-muted);
    opacity: 0.6;
  }

  .output-area {
    flex: 1;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    font-family: "SF Mono", "Fira Code", "Cascadia Code", monospace;
    font-size: 0.9rem;
    line-height: 1.6;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
    box-shadow: var(--shadow);
    position: relative;
    cursor: default;
    transition: border-color var(--transition), box-shadow var(--transition);
  }

  .output-area.has-content {
    cursor: pointer;
  }

  .output-area.has-content:hover {
    border-color: var(--accent);
    box-shadow: var(--shadow-md);
  }

  .placeholder {
    color: var(--text-muted);
    opacity: 0.6;
  }

  .controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0 0.5rem;
  }

  .redact-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 2rem;
    background: var(--primary);
    color: #fff;
    border: none;
    border-radius: var(--radius);
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: background var(--transition), transform var(--transition), box-shadow var(--transition);
    box-shadow: var(--shadow-md);
    white-space: nowrap;
  }

  .redact-btn:hover:not(:disabled) {
    background: var(--primary-hover);
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.15);
  }

  .redact-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .redact-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .note {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: center;
    max-width: 120px;
  }

  .toast {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--primary);
    color: #fff;
    padding: 0.5rem 1.25rem;
    border-radius: var(--radius);
    font-size: 0.85rem;
    font-weight: 600;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    box-shadow: var(--shadow-md);
    animation: fadeInOut 1.5s ease forwards;
    pointer-events: none;
  }

  @keyframes fadeInOut {
    0% { opacity: 0; transform: translate(-50%, -50%) scale(0.9); }
    15% { opacity: 1; transform: translate(-50%, -50%) scale(1); }
    70% { opacity: 1; }
    100% { opacity: 0; }
  }

  @media (max-width: 700px) {
    .panels {
      flex-direction: column;
    }

    .controls {
      flex-direction: row;
      padding: 0.5rem 0;
    }
  }

  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #1a1a2e;
      --surface: #22223b;
      --text: #e8e8e8;
      --text-muted: #8b8b9e;
      --border: #3a3a5c;
      --primary: #4a4e69;
      --primary-hover: #5a5e79;
      --shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
      --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    textarea {
      background: var(--surface);
    }
  }
</style>
