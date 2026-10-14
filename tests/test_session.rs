use shadowbox::session::Session;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cow_isolation_and_commit() {
    let host_dir = tempdir().unwrap();
    let original_file = host_dir.path().join("code.txt");
    let untouched_file = host_dir.path().join("safe.txt");
    fs::write(&original_file, "original content").unwrap();
    fs::write(&untouched_file, "leave me alone").unwrap();

    let session = Session::new(host_dir.path(), None).unwrap();
    let sandbox_file = session.work_dir().join("code.txt");
    let sandbox_new_file = session.work_dir().join("new_feature.txt");

    // 1. Mutate existing file inside sandbox
    fs::write(&sandbox_file, "mutated content by agent").unwrap();
    // 2. Add new file inside sandbox
    fs::write(&sandbox_new_file, "fn new_feature() {}").unwrap();

    // Verify host files are STILL 100% UNTOUCHED while session is in progress
    assert_eq!(fs::read_to_string(&original_file).unwrap(), "original content");
    assert!(!host_dir.path().join("new_feature.txt").exists());

    // 3. Commit session
    session.commit().unwrap();

    // Verify host files now reflect the approved changes
    assert_eq!(fs::read_to_string(&original_file).unwrap(), "mutated content by agent");
    assert_eq!(fs::read_to_string(&untouched_file).unwrap(), "leave me alone");
    assert_eq!(
        fs::read_to_string(host_dir.path().join("new_feature.txt")).unwrap(),
        "fn new_feature() {}"
    );
}

#[test]
fn test_discard_cleans_up_and_leaves_host_intact() {
    let host_dir = tempdir().unwrap();
    let file = host_dir.path().join("important.txt");
    fs::write(&file, "original important text").unwrap();

    let session = Session::new(host_dir.path(), None).unwrap();
    let sandbox_file = session.work_dir().join("important.txt");

    // Agent accidentally truncates file
    fs::write(&sandbox_file, "").unwrap();

    // Discard session
    session.discard().unwrap();

    // Verify host file is completely preserved
    assert_eq!(fs::read_to_string(&file).unwrap(), "original important text");
    // Verify session directory was deleted
    assert!(!session.work_dir().exists());
}
