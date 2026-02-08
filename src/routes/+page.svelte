<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";

  let inputText = $state("");
  let outputText = $state("");
  let isProcessing = $state(false);
  let ollamaConnected = $state(false);
  let copied = $state(false);

  type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';
  let updateStatus: UpdateStatus = $state('idle');
  let updateVersion = $state('');
  let updateError = $state('');
  let downloadedBytes = $state(0);
  let downloadTotal = $state(0);
  let currentVersion = $state('');
  let updateObject: Awaited<ReturnType<typeof check>> = $state(null);

  let downloadPercent = $derived(
    downloadTotal > 0 ? Math.round((downloadedBytes / downloadTotal) * 100) : 0
  );

  async function checkForUpdate(silent = false) {
    if (updateStatus === 'checking' || updateStatus === 'downloading') return;
    updateStatus = 'checking';
    updateError = '';
    try {
      const update = await check();
      if (update) {
        updateObject = update;
        updateVersion = update.version;
        updateStatus = 'available';
      } else {
        updateStatus = 'idle';
      }
    } catch (e) {
      if (silent) {
        updateStatus = 'idle';
      } else {
        updateError = String(e);
        updateStatus = 'error';
      }
    }
  }

  async function downloadUpdate() {
    if (!updateObject || updateStatus === 'downloading') return;
    updateStatus = 'downloading';
    downloadedBytes = 0;
    downloadTotal = 0;
    try {
      await updateObject.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            downloadTotal = event.data.contentLength ?? 0;
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength;
            break;
          case 'Finished':
            break;
        }
      });
      updateStatus = 'ready';
    } catch (e) {
      updateError = String(e);
      updateStatus = 'error';
    }
  }

  async function installAndRestart() {
    await relaunch();
  }

  function dismissError() {
    updateStatus = 'idle';
    updateError = '';
  }

  onMount(async () => {
    ollamaConnected = await invoke<boolean>("check_ollama");
    try {
      currentVersion = await getVersion();
    } catch {}
    checkForUpdate(true);
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
    <div class="header-left">
      <h1>De-Id</h1>
      {#if currentVersion}
        <button
          class="version-badge"
          onclick={() => checkForUpdate(false)}
          disabled={updateStatus === 'checking'}
          title="Check for updates"
        >
          {#if updateStatus === 'checking'}
            <span class="spinner-small"></span>
          {/if}
          v{currentVersion}
        </button>
      {/if}
    </div>
    <div class="status" title={ollamaConnected ? "Name detection active" : "Name detection unavailable — LLM not connected"}>
      <span class="status-dot" class:connected={ollamaConnected} class:disconnected={!ollamaConnected}></span>
      <span class="status-label">{ollamaConnected ? "LLM connected" : "LLM unavailable"}</span>
    </div>
  </header>

  {#if updateStatus === 'available'}
    <div class="update-banner">
      <span>Update available: v{updateVersion}</span>
      <button class="update-btn" onclick={downloadUpdate}>Download</button>
    </div>
  {:else if updateStatus === 'downloading'}
    <div class="update-banner">
      <span>Downloading v{updateVersion}... {downloadPercent}%</span>
      <div class="progress-bar">
        <div class="progress-fill" style="width: {downloadPercent}%"></div>
      </div>
    </div>
  {:else if updateStatus === 'ready'}
    <div class="update-banner ready">
      <span>Update v{updateVersion} ready</span>
      <button class="update-btn install" onclick={installAndRestart}>Install &amp; Restart</button>
    </div>
  {:else if updateStatus === 'error'}
    <div class="update-banner error">
      <span class="error-text">Update error: {updateError}</span>
      <button class="update-btn dismiss" onclick={dismissError}>Dismiss</button>
    </div>
  {/if}

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

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--primary);
  }

  .version-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.15rem 0.5rem;
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-muted);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 999px;
    cursor: pointer;
    transition: border-color var(--transition), color var(--transition);
  }

  .version-badge:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
  }

  .version-badge:disabled {
    cursor: default;
    opacity: 0.7;
  }

  .spinner-small {
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .update-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.6rem 1rem;
    background: var(--surface);
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    font-size: 0.85rem;
    color: var(--text);
    box-shadow: var(--shadow);
  }

  .update-banner.ready {
    border-color: #00b894;
  }

  .update-banner.error {
    border-color: #d63031;
  }

  .error-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .update-btn {
    padding: 0.3rem 0.75rem;
    font-size: 0.8rem;
    font-weight: 600;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
    white-space: nowrap;
    background: var(--accent);
    color: #fff;
    transition: opacity var(--transition);
  }

  .update-btn:hover {
    opacity: 0.85;
  }

  .update-btn.install {
    background: #00b894;
  }

  .update-btn.dismiss {
    background: var(--text-muted);
  }

  .progress-bar {
    flex: 1;
    max-width: 200px;
    height: 6px;
    background: var(--border);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
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
