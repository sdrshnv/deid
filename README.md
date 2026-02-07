# De-Id

Local text de-identification tool. Paste text containing personal information, and De-Id redacts emails and names — all processing stays on your machine.

## Download

Grab the latest release for your platform from the [Releases page](https://github.com/sdrshnv/deid/releases).

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `De-Id_<version>_aarch64.dmg` |
| macOS (Intel) | `De-Id_<version>_x64.dmg` |
| Windows | `De-Id_<version>_x64-setup.exe` or `.msi` |
| Linux | `De-Id_<version>_amd64.deb` or `.AppImage` |

## Installation

**macOS:** Open the `.dmg` and drag De-Id to Applications. On first launch, macOS will block the unsigned app. Go to **System Settings > Privacy & Security**, scroll down, and click **Open Anyway**.

**Windows:** Run the `.exe` installer. Windows SmartScreen may warn about an unrecognized app — click **More info** then **Run anyway**.

**Linux (`.deb`):** `sudo dpkg -i De-Id_*.deb`

**Linux (`.AppImage`):** `chmod +x De-Id_*.AppImage && ./De-Id_*.AppImage`

## Prerequisites

De-Id detects **email addresses** using built-in pattern matching (always available).

For **name detection**, you need [Ollama](https://ollama.com) running locally with the `llama3.2:3b` model:

```sh
ollama pull llama3.2:3b
ollama serve   # if not already running
```

A status indicator in the app header shows whether Ollama is connected. Without Ollama, the app falls back to regex-only mode (emails only).

## How It Works

The app has a two-panel layout: paste text on the left, click **Redact**, and the de-identified text appears on the right with PII replaced by `[REDACTED-email]` or `[REDACTED-name]` placeholders. Click the output to copy it to your clipboard.

Detection uses a hybrid approach — regex for emails (instant) and Ollama for names (requires the local LLM). Overlapping detections are deduplicated automatically.

## Development

```sh
git clone https://github.com/sdrshnv/deid.git
cd deid
npm install
npm run tauri dev
```

Requires [Node.js](https://nodejs.org) (LTS) and [Rust](https://rustup.rs) (stable).

## Tech Stack

- [Tauri v2](https://v2.tauri.app) — desktop framework
- [SvelteKit](https://svelte.dev/docs/kit) + [Svelte 5](https://svelte.dev) — frontend
- [Ollama](https://ollama.com) — local LLM for name detection

## License

MIT
