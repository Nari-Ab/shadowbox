use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use shadowbox::session::Session;
use shadowbox::tui::app::{AppAction, TuiApp};
use tempfile::tempdir;

#[test]
fn test_tui_key_actions() {
    let dir = tempdir().unwrap();
    let session = Session::new(dir.path(), None).unwrap();
    let mut app = TuiApp::new(session, vec!["echo".to_string()]);

    // Press 'c' -> CommitAndExit
    let action = app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    assert_eq!(action, AppAction::CommitAndExit);

    // Press 'q' -> DiscardAndExit
    let action = app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    assert_eq!(action, AppAction::DiscardAndExit);

    // Press 'Tab' -> NextTab
    let action = app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert_eq!(action, AppAction::NextTab);
    assert_eq!(app.active_tab, 1);

    // Press 'd' -> ToggleDiffView
    let action = app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    assert_eq!(action, AppAction::ToggleDiffView);
}
