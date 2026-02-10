# De-Id

Local text de-identification tool. Paste text containing personal information, and De-Id redacts emails, names, and file paths — all processing stays on your machine.

<img width="978" height="589" alt="image" src="https://github.com/user-attachments/assets/0e7e1c75-2048-48d5-8cc7-f8983eaa2379" />


## Download

Grab the latest release for your platform from the [Releases page](https://github.com/sdrshnv/deid/releases).

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `De-Id_<version>_aarch64.dmg` |
| macOS (Intel) | `De-Id_<version>_x64.dmg` |
| Windows | `De-Id_<version>_x64-setup.exe` or `.msi` |
| Linux | `De-Id_<version>_amd64.deb` or `.AppImage` |

## Installation

**macOS:** Open the `.dmg` and drag De-Id to Applications. On first launch, macOS will block the unsigned app. If you see a "damaged and can't be opened" error, run:

```sh
xattr -cr /Applications/De-Id.app
```

Otherwise, go to **System Settings > Privacy & Security**, scroll down, and click **Open Anyway**.

**Windows:** Run the `.exe` installer. Windows SmartScreen may warn about an unrecognized app — click **More info** then **Run anyway**.

**Linux (`.deb`):** `sudo dpkg -i De-Id_*.deb`

**Linux (`.AppImage`):** `chmod +x De-Id_*.AppImage && ./De-Id_*.AppImage`

## Prerequisites

De-Id detects **email addresses** and **file paths** using built-in pattern matching (always available).

For **name detection**, you need [Ollama](https://ollama.com) running locally with the `qwen3:4b` model:

```sh
ollama pull qwen3:4b
ollama serve   # if not already running
```

A status indicator in the app header shows whether Ollama is connected. Without Ollama, the app falls back to regex-only mode (emails and file paths only).

## How It Works

The app has a two-panel layout: paste text on the left, click **De-Id**, and the de-identified text appears on the right with PII replaced by `[REDACTED-email]`, `[REDACTED-name]`, or `[REDACTED-file]` placeholders. Click the output to copy it to your clipboard.

Detection uses a hybrid approach — regex for emails and file paths (instant) and Ollama for names (requires the local LLM). Overlapping detections are deduplicated automatically.

## Tech Stack

- [Tauri v2](https://v2.tauri.app) — desktop framework
- [SvelteKit](https://svelte.dev/docs/kit) + [Svelte 5](https://svelte.dev) — frontend
- [Ollama](https://ollama.com) — local LLM for name detection

## License

MIT
