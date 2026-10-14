use crate::diff::ChangeType;
use crate::tui::app::TuiApp;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap};
use ratatui::Frame;

pub fn render(app: &mut TuiApp, frame: &mut Frame) {
    let size = frame.area();

    // Vertical layout: Header (3), Main Body (Min 0), Footer (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(size);

    // 1. Header
    let status_text = if app.is_running {
        Span::styled(" RUNNING ⚡ ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD))
    } else {
        match app.exit_code {
            Some(0) => Span::styled(" FINISHED (EXIT 0) ✅ ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Some(code) => Span::styled(format!(" FAILED (EXIT {}) ❌ ", code), Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
            None => Span::styled(" STOPPED ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        }
    };

    let cmd_str = app.command.join(" ");
    let header_line = Line::from(vec![
        Span::styled(" SHADOWBOX ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" | Target Command: "),
        Span::styled(cmd_str, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" | Status: "),
        status_text,
    ]);

    let header_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let header = Paragraph::new(header_line).block(header_block);
    frame.render_widget(header, chunks[0]);

    // 2. Main Body: Left Pane (Modified Files - 35%), Right Pane (Content - 65%)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    // 2a. Left Pane: Modified Files List
    let items: Vec<ListItem> = if app.diffs.is_empty() {
        vec![ListItem::new(Span::styled("  No file modifications yet", Style::default().fg(Color::DarkGray)))]
    } else {
        app.diffs
            .iter()
            .enumerate()
            .map(|(i, diff)| {
                let is_selected = i == app.selected_diff;
                let (tag, color) = match diff.change_type {
                    ChangeType::Added => ("[A]", Color::Green),
                    ChangeType::Modified => ("[M]", Color::Yellow),
                    ChangeType::Deleted => ("[D]", Color::Red),
                };

                let prefix = if is_selected { "▶ " } else { "  " };
                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ", tag), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(diff.path.display().to_string(), if is_selected { Style::default().fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Gray) }),
                    Span::styled(format!(" (+{} -{})", diff.additions, diff.deletions), Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

    let files_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Modified Files ({}) ", app.diffs.len()))
        .style(Style::default().fg(Color::White));
    let files_list = List::new(items).block(files_block);
    frame.render_widget(files_list, body_chunks[0]);

    // 2b. Right Pane: Tabs & Content
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(2)])
        .split(body_chunks[1]);

    let tab_titles = vec!["1. Live Terminal", "2. Diff Viewer", "3. Security Audit"];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" Views [Tab] "))
        .select(app.active_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, right_chunks[0]);

    match app.active_tab {
        0 => {
            // Live Terminal
            let visible_lines: Vec<Line> = app
                .terminal_lines
                .iter()
                .rev()
                .take(30)
                .rev()
                .map(|l| Line::from(l.as_str()))
                .collect();

            let terminal_block = Block::default()
                .borders(Borders::ALL)
                .title(" PTY Console Stream ")
                .style(Style::default().fg(Color::Green));
            let term_widget = Paragraph::new(visible_lines)
                .block(terminal_block)
                .wrap(Wrap { trim: false });
            frame.render_widget(term_widget, right_chunks[1]);
        }
        1 => {
            // Diff Viewer
            let diff_content = if let Some(diff) = app.diffs.get(app.selected_diff) {
                let lines: Vec<Line> = diff
                    .unified_diff
                    .lines()
                    .map(|l| {
                        if l.starts_with('+') && !l.starts_with("+++") {
                            Line::from(Span::styled(l, Style::default().fg(Color::Green)))
                        } else if l.starts_with('-') && !l.starts_with("---") {
                            Line::from(Span::styled(l, Style::default().fg(Color::Red)))
                        } else if l.starts_with('@') {
                            Line::from(Span::styled(l, Style::default().fg(Color::Cyan)))
                        } else {
                            Line::from(Span::raw(l))
                        }
                    })
                    .collect();
                lines
            } else {
                vec![Line::from(Span::styled("No file selected", Style::default().fg(Color::DarkGray)))]
            };

            let title = if let Some(diff) = app.diffs.get(app.selected_diff) {
                format!(" Diff: {} ", diff.path.display())
            } else {
                " Diff Viewer ".to_string()
            };

            let diff_block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().fg(Color::Yellow));
            let diff_widget = Paragraph::new(diff_content).block(diff_block);
            frame.render_widget(diff_widget, right_chunks[1]);
        }
        _ => {
            // Security Audit
            let audit_lines = vec![
                Line::from(Span::styled("🔒 SecretShield Audit Summary", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![
                    Span::raw("• Blocked Environment Variables: "),
                    Span::styled(format!("{}", app.blocked_vars_count), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" (credentials stripped from agent context)"),
                ]),
                Line::from(vec![
                    Span::raw("• Cloaked Sensitive Files: "),
                    Span::styled(format!("{}", app.cloaked_secrets_count), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(" (.env files sanitized with safe mocks)"),
                ]),
                Line::from("• Isolation Status: Active (APFS Copy-on-Write sandbox)"),
                Line::from("• Host Workspace Status: 100% Protected (No unapproved writes)"),
            ];

            let audit_block = Block::default()
                .borders(Borders::ALL)
                .title(" DLP & Security Audit ")
                .style(Style::default().fg(Color::Magenta));
            let audit_widget = Paragraph::new(audit_lines).block(audit_block);
            frame.render_widget(audit_widget, right_chunks[1]);
        }
    }

    // 3. Footer Bar
    let footer_text = Line::from(vec![
        Span::styled(" [c] ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" Commit & Apply to Host  "),
        Span::styled(" [q] ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Discard Sandbox  "),
        Span::styled(" [Tab] ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" Switch View  "),
        Span::styled(" [↑/↓] ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" Select Diff"),
    ]);

    let footer_block = Block::default().borders(Borders::ALL);
    let footer = Paragraph::new(footer_text).block(footer_block);
    frame.render_widget(footer, chunks[2]);
}
