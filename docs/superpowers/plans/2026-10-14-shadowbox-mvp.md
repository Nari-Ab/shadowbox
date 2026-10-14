# ShadowBox MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and verify ShadowBox (`shadowbox`), a zero-trust, copy-on-write sandbox runtime and interactive time-machine for AI coding agents on macOS and Linux.

**Architecture:** ShadowBox wraps any command in an APFS zero-copy clone directory (`.shadowbox/sessions/`), sanitizes secrets and `.env` files via SecretShield, executes the agent in a portable PTY, calculates unified AST/file diffs, and renders a dual-pane Ratatui TUI allowing real-time inspection, undo/rewind to past checkpoints, and atomic commits back to the host.

**Tech Stack:** Rust 2021, `cargo`, `clap` (CLI), `libc` (APFS `clonefile`), `portable-pty` (terminal emulation), `similar` (diffs), `notify` (file watching), `ratatui` + `crossterm` (TUI), `serde` + `serde_json`, `anyhow`.

**Spec:** `docs/superpowers/specs/2026-10-14-shadowbox-design.md`

## Global Constraints

- Must compile on macOS (Apple Silicon & Intel) using standard `cargo build`.
- Must not require root or `sudo` privileges.
- Zero-copy file cloning on macOS via APFS `libc::clonefile()`.
- Host files must remain 100% byte-for-byte untouched during session until user hits `[c]` Commit.
- Sensitive environment variables (`AWS_*`, `SSH_*`, `GITHUB_TOKEN`) must be stripped before spawning agent.

## Review Focus

1. **APFS Non-APFS Fallback:** When running on a non-APFS filesystem, gracefully fall back to recursive fast copying without failing.
2. **Process Signal Termination:** When user hits `Ctrl+C` or `[q]` (Discard), all child and grandchild processes in the PTY must be cleanly terminated with `SIGTERM` then `SIGKILL`.
3. **Empty or Read-Only Directories:** Handle zero-file workspaces and permission-locked files gracefully without panics.
4. **Symlink Boundary Traversal:** Ensure copy-on-write clone does not recursively follow symlinks pointing outside the repository root.
5. **Dirty Host Working Tree:** If host has uncommitted changes when session is committed, merge cleanly or alert the user rather than overwriting silently.

---

### Task 1: Project Scaffolding & Cargo Setup

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Test: `tests/test_scaffolding.rs`

**Interfaces:**
- Produces: `shadowbox::version() -> &'static str`, basic CLI argument parsing structure.

- [ ] **Step 1: Write the failing test**
```rust
// tests/test_scaffolding.rs
#[test]
fn test_version() {
    assert_eq!(shadowbox::version(), env!("CARGO_PKG_VERSION"));
}
```

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test test_scaffolding`
Expected: FAIL (crate does not exist or function not found)

- [ ] **Step 3: Implement `Cargo.toml`, `src/lib.rs`, and `src/main.rs`**
Define crate metadata and dependencies (`clap`, `anyhow`, `libc`, `portable-pty`, `ratatui`, `crossterm`, `similar`, `notify`, `serde`, `serde_json`, `tempfile`).

- [ ] **Step 4: Run test to verify it passes**
Run: `cargo test --test test_scaffolding`
Expected: PASS

- [ ] **Step 5: Commit**
```bash
GIT_CONFIG_GLOBAL=/dev/null git add Cargo.toml src/ tests/
GIT_CONFIG_GLOBAL=/dev/null git commit -m "chore: scaffold shadowbox crate with dependencies and version test"
```

---

### Task 2: Secret Shield & Environment Sanitizer

**Files:**
- Create: `src/shield/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/test_shield.rs`

**Interfaces:**
- Produces: `SecretShield::sanitize_env(vars: &HashMap<String, String>) -> HashMap<String, String>`
- Produces: `SecretShield::is_sensitive_file(path: &Path) -> bool`
- Produces: `SecretShield::cloak_sensitive_files(workspace: &Path) -> Result<Vec<PathBuf>>`

- [ ] **Step 1: Write the failing test**
```rust
// tests/test_shield.rs
use shadowbox::shield::SecretShield;
use std::collections::HashMap;

#[test]
fn test_sanitize_env_removes_secrets() {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/bin".to_string());
    env.insert("AWS_SECRET_ACCESS_KEY".to_string(), "xyz123".to_string());
    env.insert("GITHUB_TOKEN".to_string(), "ghp_abc".to_string());
    env.insert("SSH_AUTH_SOCK".to_string(), "/tmp/ssh".to_string());

    let sanitized = SecretShield::sanitize_env(&env);
    assert!(sanitized.contains_key("PATH"));
    assert!(!sanitized.contains_key("AWS_SECRET_ACCESS_KEY"));
    assert!(!sanitized.contains_key("GITHUB_TOKEN"));
    assert!(!sanitized.contains_key("SSH_AUTH_SOCK"));
}
```

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test test_shield`
Expected: FAIL (module `shield` not found)

- [ ] **Step 3: Implement `src/shield/mod.rs`**
Implement environment variable blocklist matching (prefixes `AWS_`, `SSH_`, `GITHUB_TOKEN`, `ANTHROPIC_API_KEY` unless configured, `OPENAI_API_KEY`, etc.) and sensitive file detection (`.env`, `.env.*`, `*.pem`, `id_rsa`).

- [ ] **Step 4: Run test to verify it passes**
Run: `cargo test --test test_shield`
Expected: PASS

- [ ] **Step 5: Commit**
```bash
GIT_CONFIG_GLOBAL=/dev/null git add src/shield/ tests/test_shield.rs src/lib.rs
GIT_CONFIG_GLOBAL=/dev/null git commit -m "feat(shield): implement environment variable and sensitive file cloaking"
```

---

### Task 3: Session Manager & CoW Engine (macOS APFS clonefile + Fallback)

**Files:**
- Create: `src/session/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/test_session.rs`

**Interfaces:**
- Produces: `Session::new(source_root: &Path, session_root: Option<&Path>) -> Result<Session>`
- Produces: `Session::commit(&self) -> Result<()>`
- Produces: `Session::discard(&self) -> Result<()>`
- Produces: `Session::work_dir(&self) -> &Path`

- [ ] **Step 1: Write the failing test**
```rust
// tests/test_session.rs
use shadowbox::session::Session;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cow_isolation_and_commit() {
    let host_dir = tempdir().unwrap();
    let original_file = host_dir.path().join("code.txt");
    fs::write(&original_file, "original content").unwrap();

    let session = Session::new(host_dir.path(), None).unwrap();
    let sandbox_file = session.work_dir().join("code.txt");
    
    // Mutate in sandbox
    fs::write(&sandbox_file, "mutated content").unwrap();

    // Verify host file is still pristine
    assert_eq!(fs::read_to_string(&original_file).unwrap(), "original content");

    // Commit session
    session.commit().unwrap();

    // Verify host file has been updated
    assert_eq!(fs::read_to_string(&original_file).unwrap(), "mutated content");
}
```

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test test_session`
Expected: FAIL (module `session` not found)

- [ ] **Step 3: Implement `src/session/mod.rs`**
Implement APFS `libc::clonefile()` cloning on macOS and standard file copy fallback. Implement `commit()` (syncing added/modified files back to host and removing deleted files) and `discard()` (removing the session folder).

- [ ] **Step 4: Run test to verify it passes**
Run: `cargo test --test test_session`
Expected: PASS

- [ ] **Step 5: Commit**
```bash
GIT_CONFIG_GLOBAL=/dev/null git add src/session/ tests/test_session.rs src/lib.rs
GIT_CONFIG_GLOBAL=/dev/null git commit -m "feat(session): implement copy-on-write sandbox isolation and atomic commit"
```

---

### Task 4: Diff Engine & Checkpoint Tracker

**Files:**
- Create: `src/diff/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/test_diff.rs`

**Interfaces:**
- Produces: `DiffEngine::diff_dirs(base_dir: &Path, current_dir: &Path) -> Result<Vec<FileDiff>>`
- Produces: `FileDiff { path: PathBuf, change_type: ChangeType, unified_diff: String, additions: usize, deletions: usize }`

- [ ] **Step 1: Write the failing test**
```rust
// tests/test_diff.rs
use shadowbox::diff::{DiffEngine, ChangeType};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_diff_engine_detects_changes() {
    let base = tempdir().unwrap();
    let current = tempdir().unwrap();

    fs::write(base.path().join("a.txt"), "hello\nworld\n").unwrap();
    fs::write(current.path().join("a.txt"), "hello\nbeautiful\nworld\n").unwrap();
    fs::write(current.path().join("new.txt"), "brand new\n").unwrap();

    let diffs = DiffEngine::diff_dirs(base.path(), current.path()).unwrap();
    assert_eq!(diffs.len(), 2);
}
```

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test test_diff`
Expected: FAIL (module `diff` not found)

- [ ] **Step 3: Implement `src/diff/mod.rs`**
Implement recursive directory comparison and unified diff generation using the `similar` crate.

- [ ] **Step 4: Run test to verify it passes**
Run: `cargo test --test test_diff`
Expected: PASS

- [ ] **Step 5: Commit**
```bash
GIT_CONFIG_GLOBAL=/dev/null git add src/diff/ tests/test_diff.rs src/lib.rs
GIT_CONFIG_GLOBAL=/dev/null git commit -m "feat(diff): implement directory diffing and unified diff calculation"
```

---

### Task 5: PTY Process Runner & Execution Coordinator

**Files:**
- Create: `src/runner/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/test_runner.rs`

**Interfaces:**
- Produces: `PtyRunner::spawn(command: &[String], cwd: &Path, env: &HashMap<String, String>) -> Result<PtyHandle>`
- Produces: `PtyHandle::read_output(&mut self) -> Result<Vec<u8>>`
- Produces: `PtyHandle::write_input(&mut self, input: &[u8]) -> Result<()>`
- Produces: `PtyHandle::wait(&mut self) -> Result<ExitStatus>`

- [ ] **Step 1: Write the failing test**
```rust
// tests/test_runner.rs
use shadowbox::runner::PtyRunner;
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_pty_runner_executes_command() {
    let dir = tempdir().unwrap();
    let env = HashMap::new();
    let cmd = vec!["echo".to_string(), "hello shadowbox".to_string()];
    
    let mut handle = PtyRunner::spawn(&cmd, dir.path(), &env).unwrap();
    let status = handle.wait().unwrap();
    assert!(status.success());
}
```

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test test_runner`
Expected: FAIL (module `runner` not found)

- [ ] **Step 3: Implement `src/runner/mod.rs`**
Implement PTY allocation, process spawning, non-blocking I/O reader using `portable-pty`.

- [ ] **Step 4: Run test to verify it passes**
Run: `cargo test --test test_runner`
Expected: PASS

- [ ] **Step 5: Commit**
```bash
GIT_CONFIG_GLOBAL=/dev/null git add src/runner/ tests/test_runner.rs src/lib.rs
GIT_CONFIG_GLOBAL=/dev/null git commit -m "feat(runner): implement pseudo-terminal command execution via portable-pty"
```

---

### Task 6: Interactive Terminal UI (TUI) & Checkpoint Timeline

**Files:**
- Create: `src/tui/mod.rs`
- Create: `src/tui/app.rs`
- Create: `src/tui/ui.rs`
- Modify: `src/lib.rs`
- Test: `tests/test_tui_state.rs`

**Interfaces:**
- Produces: `TuiApp::new(session: Session, command: Vec<String>) -> TuiApp`
- Produces: `TuiApp::handle_key(&mut self, key: KeyEvent) -> AppAction`
- Produces: `AppAction`: `Continue`, `CommitAndExit`, `DiscardAndExit`, `RewindToCheckpoint(usize)`

- [ ] **Step 1: Write the failing test**
```rust
// tests/test_tui_state.rs
use shadowbox::tui::app::{TuiApp, AppAction};
use shadowbox::session::Session;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::tempdir;

#[test]
fn test_tui_key_actions() {
    let dir = tempdir().unwrap();
    let session = Session::new(dir.path(), None).unwrap();
    let mut app = TuiApp::new(session, vec!["echo".to_string()]);

    // Press 'c' -> Commit
    let action = app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    assert_eq!(action, AppAction::CommitAndExit);

    // Press 'q' -> Discard
    let action = app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    assert_eq!(action, AppAction::DiscardAndExit);
}
```

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test test_tui_state`
Expected: FAIL (module `tui` not found)

- [ ] **Step 3: Implement `src/tui/`**
Implement `TuiApp` state machine, timeline list, diff viewer, and Ratatui rendering functions in `src/tui/ui.rs`.

- [ ] **Step 4: Run test to verify it passes**
Run: `cargo test --test test_tui_state`
Expected: PASS

- [ ] **Step 5: Commit**
```bash
GIT_CONFIG_GLOBAL=/dev/null git add src/tui/ tests/test_tui_state.rs src/lib.rs
GIT_CONFIG_GLOBAL=/dev/null git commit -m "feat(tui): implement dual-pane terminal interface and keyboard actions"
```

---

### Task 7: End-to-End CLI Integration & Full Verification

**Files:**
- Modify: `src/main.rs`
- Test: `tests/test_e2e_cli.rs`
- Create: `README.md`

**Interfaces:**
- Produces: Working `shadowbox` executable accepting commands like `shadowbox claude`, `shadowbox --dry-run <cmd>`, `shadowbox clean`.

- [ ] **Step 1: Write the failing E2E test**
```rust
// tests/test_e2e_cli.rs
use std::process::Command;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_shadowbox"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ShadowBox"));
}
```

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test test_e2e_cli`
Expected: FAIL (binary not built or CLI flags missing)

- [ ] **Step 3: Wire up `src/main.rs` and write comprehensive README**
Parse CLI flags (`--dry-run`, `--no-tui`, `--allow-secret`), orchestrate Session + Shield + PtyRunner + TuiApp. Write developer-friendly README with badges, GIF mockup, and install instructions.

- [ ] **Step 4: Run full verification suite**
Run: `cargo test --all`
Expected: All tests PASS with 0 errors.

- [ ] **Step 5: Commit and tag initial v0.1.0**
```bash
GIT_CONFIG_GLOBAL=/dev/null git add src/main.rs tests/ README.md
GIT_CONFIG_GLOBAL=/dev/null git commit -m "feat: complete end-to-end ShadowBox CLI implementation and documentation"
```
