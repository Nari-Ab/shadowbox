use shadowbox::diff::{ChangeType, DiffEngine};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_diff_engine_detects_changes() {
    let base = tempdir().unwrap();
    let current = tempdir().unwrap();

    // 1. Modified file
    fs::write(base.path().join("a.txt"), "hello\nworld\n").unwrap();
    fs::write(current.path().join("a.txt"), "hello\nbeautiful\nworld\n").unwrap();

    // 2. Added file
    fs::write(current.path().join("new.txt"), "brand new line\n").unwrap();

    // 3. Deleted file
    fs::write(base.path().join("deleted.txt"), "goodbye\n").unwrap();

    // 4. Untouched file
    fs::write(base.path().join("same.txt"), "no change\n").unwrap();
    fs::write(current.path().join("same.txt"), "no change\n").unwrap();

    let diffs = DiffEngine::diff_dirs(base.path(), current.path()).unwrap();
    assert_eq!(diffs.len(), 3);

    // Verify modified
    let mod_diff = diffs.iter().find(|d| d.path.ends_with("a.txt")).unwrap();
    assert_eq!(mod_diff.change_type, ChangeType::Modified);
    assert_eq!(mod_diff.additions, 1);
    assert_eq!(mod_diff.deletions, 0);
    assert!(mod_diff.unified_diff.contains("+beautiful"));

    // Verify added
    let add_diff = diffs.iter().find(|d| d.path.ends_with("new.txt")).unwrap();
    assert_eq!(add_diff.change_type, ChangeType::Added);
    assert_eq!(add_diff.additions, 1);

    // Verify deleted
    let del_diff = diffs.iter().find(|d| d.path.ends_with("deleted.txt")).unwrap();
    assert_eq!(del_diff.change_type, ChangeType::Deleted);
    assert_eq!(del_diff.deletions, 1);
}
