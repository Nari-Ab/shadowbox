use shadowbox::runner::PtyRunner;
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_pty_runner_executes_command() {
    let dir = tempdir().unwrap();
    let mut env = HashMap::new();
    env.insert("CUSTOM_VAR".to_string(), "shadow_val".to_string());
    let cmd = vec!["echo".to_string(), "hello shadowbox".to_string()];

    let mut handle = PtyRunner::spawn(&cmd, dir.path(), &env).unwrap();
    // Read output produced by the echo command
    let output = handle.read_all_output().unwrap();
    let status = handle.wait().unwrap();
    assert!(status.success());
    let output_str = String::from_utf8_lossy(&output);
    assert!(output_str.contains("hello shadowbox"));
}
