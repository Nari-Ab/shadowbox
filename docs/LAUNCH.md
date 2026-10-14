# 🚀 ShadowBox Launch Kit

Ready-to-use launch drafts tailored for each developer platform.

---

## 1. Hacker News (Show HN)

**URL:** `https://news.ycombinator.com/submit`  
**Title:** Show HN: ShadowBox – Run autonomous coding agents in an APFS CoW sandbox with one-key rollback  
**URL:** `https://github.com/Nari-Ab/shadowbox`  
**Text (if posting as text or top comment):**

> Hi HN,
> 
> Like many of you, I've been using autonomous AI coding agents like Claude Code, Cursor, and Aider with `--dangerously-skip-permissions` or unattended loops. But there's always that nagging anxiety: What if the agent deletes files, leaks an API key from `.env`, or introduces breaking changes into the working branch?
> 
> Traditional Git doesn't prevent side-effects, and Docker is too heavy for fast local CLI scripts.
> 
> I built **ShadowBox** ([github.com/Nari-Ab/shadowbox](https://github.com/Nari-Ab/shadowbox)) in Rust:
> 
> 1. **Zero-Copy CoW Scratchpad:** On macOS, it leverages APFS `clonefile()`. It clones a 50,000-file project into an ephemeral session in ~14ms with 0 additional disk usage. Real host files are never touched.
> 2. **SecretShield:** Automatically strips high-risk environment variables (`AWS_*`, `SSH_*`, `GITHUB_TOKEN`, `DATABASE_URL`) from the agent subprocess and masks `.env` files with safe mocks.
> 3. **Time-Machine TUI:** A dual-pane Ratatui interface showing real-time file diffs and command streams. Hit `[c]` to atomically apply changes back to your host, or hit `[q]` to discard the session with zero trace.
> 
> Install:
> `curl -fsSL https://raw.githubusercontent.com/Nari-Ab/shadowbox/main/install.sh | bash`
> 
> Run:
> `shadowbox claude --dangerously-skip-permissions`
> 
> It's open source (Apache-2.0 / MIT). Would love your feedback!

---

## 2. Reddit (`r/programming`, `r/LocalLLaMA`, `r/ClaudeAI`)

**Title:** I built an open-source zero-copy sandbox & flight recorder for Claude Code and Cursor agents

**Body:**

> Hey everyone,
> 
> Running autonomous coding agents in "unattended" mode is awesome until an agent wipes files, corrupts Git state, or reads your `.env`.
> 
> I built **ShadowBox**: a lightweight (~1.9 MB), single-binary Rust tool that wraps any agent command in an instant copy-on-write scratchpad:
> 
> - **15ms APFS Isolation:** All writes happen on an isolated copy-on-write clone. Your host files remain 100% untouched until you approve.
> - **SecretShield:** Blocks sensitive env vars (`AWS_*`, `SSH_*`, etc.) and masks `.env` to prevent credential leakage to LLM APIs.
> - **One-key rollback:** If the agent did what you wanted, press `[c]` to commit changes. If it broke something, press `[q]` to purge without touching your code.
> 
> GitHub: https://github.com/Nari-Ab/shadowbox  
> 
> You can install it with:  
> `curl -fsSL https://raw.githubusercontent.com/Nari-Ab/shadowbox/main/install.sh | bash`  
> 
> Feedback and contributions welcome!

---

## 3. Twitter / X Thread

**Post 1 (Hook):**
> Autonomous coding agents are incredible, but running `--dangerously-skip-permissions` is terrifying.
> 
> What if the agent deletes your database, corrupts Git, or leaks your `.env`?
> 
> Introducing ShadowBox: a zero-trust, copy-on-write sandbox & time-machine for AI agents. 📦⚡
> 
> [Attach docs/assets/case-study.gif]
> 
> https://github.com/Nari-Ab/shadowbox

**Post 2:**
> How it works:
> 
> ⚡ Zero-Copy APFS: Clones a 50k-file workspace in ~14ms using 0 bytes of disk. The agent works on the scratchpad; your real repo is 100% untouched.
> 
> 🔒 SecretShield: Strips credentials (`AWS_*`, `SSH_*`, `GITHUB_TOKEN`) and cloaks `.env` with harmless mocks.

**Post 3:**
> ⏪ One-Key Rollback:
> 
> A live Ratatui dual-pane TUI streams agent commands and tracks modified files.
> 
> • Press `[c]` to atomically apply changes to host
> • Press `[q]` to discard everything cleanly
> 
> Single binary (1.9MB). Single command:
> `shadowbox claude`
> 
> ⭐ Star on GitHub: https://github.com/Nari-Ab/shadowbox

---

## 4. Product Hunt

**Tagline:** Zero-trust sandbox & time-machine for AI coding agents  
**Topics:** Developer Tools, Artificial Intelligence, Open Source, Cybersecurity  
**Pricing:** Free & Open Source (Apache-2.0)  
**Short Description:** Run Claude Code, Cursor, Aider, and OpenHands with 100% autonomy and 0% fear. Zero-copy APFS isolation, secret redaction, and instant one-key rollback.
