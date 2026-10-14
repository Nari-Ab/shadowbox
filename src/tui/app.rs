use crate::diff::{DiffEngine, FileDiff};
use crate::session::Session;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Continue,
    CommitAndExit,
    DiscardAndExit,
    ToggleDiffView,
    NextTab,
    PrevTab,
}

pub struct TuiApp {
    pub session: Session,
    pub command: Vec<String>,
    pub diffs: Vec<FileDiff>,
    pub terminal_lines: Vec<String>,
    pub active_tab: usize, // 0: Terminal, 1: Diffs, 2: Security
    pub selected_diff: usize,
    pub is_running: bool,
    pub exit_code: Option<i32>,
    pub cloaked_secrets_count: usize,
    pub blocked_vars_count: usize,
}

impl TuiApp {
    pub fn new(session: Session, command: Vec<String>) -> Self {
        Self {
            session,
            command,
            diffs: Vec::new(),
            terminal_lines: Vec::new(),
            active_tab: 0,
            selected_diff: 0,
            is_running: true,
            exit_code: None,
            cloaked_secrets_count: 0,
            blocked_vars_count: 0,
        }
    }

    /// Re-evaluates file differences between host and sandbox.
    pub fn refresh_diffs(&mut self) -> Result<()> {
        let diffs = DiffEngine::diff_dirs(self.session.source_dir(), self.session.work_dir())?;
        self.diffs = diffs;
        if self.selected_diff >= self.diffs.len() && !self.diffs.is_empty() {
            self.selected_diff = self.diffs.len() - 1;
        }
        Ok(())
    }

    /// Appends output bytes from the PTY terminal into terminal_lines.
    pub fn append_terminal_output(&mut self, bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes);
        for line in text.lines() {
            self.terminal_lines.push(line.to_string());
        }
    }

    /// Processes keyboard events from the user.
    pub fn handle_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            KeyCode::Char('c') => AppAction::CommitAndExit,
            KeyCode::Char('q') => AppAction::DiscardAndExit,
            KeyCode::Char('d') => AppAction::ToggleDiffView,
            KeyCode::Tab => {
                self.active_tab = (self.active_tab + 1) % 3;
                AppAction::NextTab
            }
            KeyCode::BackTab => {
                if self.active_tab == 0 {
                    self.active_tab = 2;
                } else {
                    self.active_tab -= 1;
                }
                AppAction::PrevTab
            }
            KeyCode::Up => {
                if self.selected_diff > 0 {
                    self.selected_diff -= 1;
                }
                AppAction::Continue
            }
            KeyCode::Down => {
                if !self.diffs.is_empty() && self.selected_diff + 1 < self.diffs.len() {
                    self.selected_diff += 1;
                }
                AppAction::Continue
            }
            _ => AppAction::Continue,
        }
    }
}
