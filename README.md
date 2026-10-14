# 📦 ShadowBox

<p align="center">
  <b>The zero-trust, copy-on-write runtime and interactive time-machine for AI coding agents.</b><br>
  <i>Run Claude Code, Cursor Composer, Aider, and OpenHands with 100% autonomy and 0% fear.</i>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg" alt="License" />
  <img src="https://img.shields.io/badge/Rust-2021-orange.svg" alt="Rust Edition" />
  <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-brightgreen.svg" alt="Platform" />
  <img src="https://img.shields.io/badge/Status-v0.1.0-blueviolet.svg" alt="Status" />
</p>

---

## ⚡ Why ShadowBox?

Autonomous AI coding agents are changing how software is built. But running them with `--dangerously-skip-permissions` or background unattended loops is terrifying:

- ❌ **Destructive Commands:** An agent can accidentally delete files (`rm -rf`), wipe database tables, or corrupt Git history.
- ❌ **Secret Exfiltration:** Rogue prompts or hallucinated scripts can read `.env`, `~/.aws`, or SSH keys and leak them to external APIs.
- ❌ **No Granular Rollback:** Traditional Git cannot undo non-git changes, background process side-effects, or uncommitted work.

**ShadowBox solves this with one command:**
```bash
shadowbox claude --dangerously-skip-permissions
```

---

## ✨ Features

- **⚡ Instant Zero-Copy Sandbox (CoW):** Uses APFS `clonefile()` on macOS (and fast CoW fallback on Linux). Clones a 50,000-file project in **15 milliseconds** with **0 bytes** of initial disk usage. Your host repository is never touched until you approve.
- **🛡️ SecretShield & DLP Guard:** Automatically strips sensitive environment variables (`AWS_*`, `SSH_*`, `GITHUB_TOKEN`, `DATABASE_URL`) from the agent's process and cloaks `.env` files with safe mocks.
- **🖥️ Dual-Pane Interactive TUI (Ratatui):**
  - **Left Pane:** Real-time list of modified files with additions/deletions stats (`[A]`, `[M]`, `[D]`).
  - **Right Pane:** Tabbed view featuring live PTY console stream, unified diff viewer, and security audit report.
- **⏪ One-Key Rollback & Atomic Commit:**
  - `[c]` **Commit:** Atomically syncs approved changes back to your host workspace.
  - `[q]` **Discard:** Instantly purges the scratch sandbox. Your code remains 100% pristine.

---

## 🚀 Quick Start

### Installation

```bash
# Using Cargo (from source)
cargo install shadowbox

# Using Homebrew (coming soon)
brew install shadowbox
```

### Usage

Wrap any agent command:
```bash
# Wrap Claude Code
shadowbox claude

# Wrap Aider with auto-commit
shadowbox aider

# Run any shell script or build tool safely
shadowbox npm run test-and-migrate

# Headless mode for CI or scripts
shadowbox --no-tui python agent.py
```

---

## ⌨️ TUI Keyboard Shortcuts

| Key | Action |
| :--- | :--- |
| `[c]` | **Commit & Apply** — Sync approved sandbox changes to host |
| `[q]` | **Discard & Exit** — Purge sandbox, leave host untouched |
| `[Tab]` | **Switch View** — Toggle between Live Terminal, Diff Viewer, and Security Audit |
| `[↑ / ↓]` | **Select File** — Browse modified files and inspect diffs |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      ShadowBox CLI                          │
│             (shadowbox claude / aider / npx)                │
└──────────────────────────────┬──────────────────────────────┘
                               │
       ┌───────────────────────┼───────────────────────┐
       ▼                       ▼                       ▼
┌──────────────┐       ┌──────────────┐       ┌─────────────────┐
│ Session CoW  │       │ PTY Process  │       │  Secret Shield  │
│   Engine     │       │   Runner     │       │  & DLP Guard    │
│ (APFS Clone) │       │ (portable-   │       │ (Env & File     │
│              │       │    pty)      │       │    Sanitizer)   │
└──────┬───────┘       └──────┬───────┘       └────────┬────────┘
       │                      │                        │
       └──────────────────────┼────────────────────────┘
                              ▼
               ┌──────────────────────────────┐
               │    Diff Engine & Watcher     │
               │      (notify + similar)      │
               └──────────────┬───────────────┘
                              ▼
               ┌──────────────────────────────┐
               │      TUI & Time-Machine      │
               │     (ratatui + crossterm)    │
               │  • Timeline & Step Browser   │
               │  • Split Diff & PTY View     │
               │  • Rewind / Commit / Abort   │
               └──────────────────────────────┘
```

---

## 🧪 Testing

Run the full verification suite:
```bash
cargo test --all
```

---

## 📄 License

Dual-licensed under MIT or Apache 2.0.
