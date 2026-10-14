<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/logo-dark.svg">
  <img src="docs/assets/logo.svg" alt="ShadowBox" width="340">
</picture>

### Zero-trust copy-on-write sandbox and time-machine for AI coding agents

[![CI](https://github.com/Nari-Ab/shadowbox/actions/workflows/ci.yml/badge.svg)](https://github.com/Nari-Ab/shadowbox/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/shadowbox.svg?color=e05d44)](https://crates.io/crates/shadowbox)
[![npm](https://img.shields.io/npm/v/shadowbox-cli.svg?color=cb3837)](https://www.npmjs.com/package/shadowbox-cli)
[![downloads](https://img.shields.io/github/downloads/Nari-Ab/shadowbox/total?label=downloads&color=0b7285)](https://github.com/Nari-Ab/shadowbox/releases)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

</div>

ShadowBox runs autonomous AI coding agents (Claude Code, Cursor, Aider, OpenHands) inside an ephemeral copy-on-write scratchpad on macOS and Linux. It strips secret credentials before the agent process spawns, isolates file mutations on an APFS zero-copy layer, and lets you review or discard changes before anything touches your real repository.

```bash
# curl (macOS / Linux):
curl -fsSL https://raw.githubusercontent.com/Nari-Ab/shadowbox/main/install.sh | bash

# npx (instant, zero install):
npx shadowbox-cli claude

# cargo:
cargo install shadowbox

# run:
shadowbox claude --dangerously-skip-permissions
```

Needs macOS (APFS) or Linux.

What a session looks like:

```text
shadowbox session — zero-trust copy-on-write runtime

2 file(s) modified in sandbox:
  ● [A] src/auth/jwt.ts (+38 lines)
  ● [M] package.json (+1 line)

SECRETS BLOCKED (3)
  ● AWS_SECRET_ACCESS_KEY — stripped from process env
  ● GITHUB_TOKEN          — stripped from process env
  ● .env                  — masked with [SHADOWBOX_REDACTED]

HOST REPOSITORY
  ● 100% untouched, 0 unapproved writes

CONTROLS
  [c] Commit approved changes to host
  [q] Discard sandbox and leave host untouched
  [Tab] Toggle diff viewer
```

<img src="docs/assets/case-study.gif" alt="ShadowBox running Claude Code in an isolated APFS scratchpad with secret protection and atomic commit" width="100%">

- **Zero-Copy CoW.** Uses APFS `clonefile()` on macOS. Clones a 50,000-file project in ~14ms with 0 bytes additional disk usage. Real files remain untouched until you commit.
- **SecretShield.** Strips sensitive env vars (`AWS_*`, `SSH_*`, `GITHUB_TOKEN`, `DATABASE_URL`) from the agent's process environment and cloaks `.env` files with harmless mocks.
- **One-key rollback.** Interactive dual-pane TUI or headless mode. Commit with `[c]`, abort and wipe with `[q]`.

## Docs

- [Architecture & Design Spec](docs/superpowers/specs/2026-10-14-shadowbox-design.md)
- [Implementation Plan](docs/superpowers/plans/2026-10-14-shadowbox-mvp.md)
- [Launch Kit](docs/LAUNCH.md)
