use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_shadowbox"))
        .arg("--help")
        .output()
        .expect("Failed to execute shadowbox binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.to_lowercase().contains("shadowbox"));
    assert!(stdout.to_lowercase().contains("sandbox"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_shadowbox"))
        .arg("--version")
        .output()
        .expect("Failed to execute shadowbox binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}
