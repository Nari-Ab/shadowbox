use shadowbox::shield::SecretShield;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_sanitize_env_removes_secrets() {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    env.insert("USER".to_string(), "developer".to_string());
    env.insert("AWS_SECRET_ACCESS_KEY".to_string(), "AKIAIOSFODNN7EXAMPLE".to_string());
    env.insert("AWS_ACCESS_KEY_ID".to_string(), "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string());
    env.insert("GITHUB_TOKEN".to_string(), "ghp_1234567890abcdef".to_string());
    env.insert("SSH_AUTH_SOCK".to_string(), "/private/tmp/com.apple.launchd.xxx/Listeners".to_string());
    env.insert("STRIPE_SECRET_KEY".to_string(), "sk_live_12345".to_string());
    env.insert("MY_DATABASE_URL".to_string(), "postgres://user:password@localhost:5432/db".to_string());

    let sanitized = SecretShield::sanitize_env(&env);

    assert!(sanitized.contains_key("PATH"));
    assert!(sanitized.contains_key("USER"));
    assert!(!sanitized.contains_key("AWS_SECRET_ACCESS_KEY"));
    assert!(!sanitized.contains_key("AWS_ACCESS_KEY_ID"));
    assert!(!sanitized.contains_key("GITHUB_TOKEN"));
    assert!(!sanitized.contains_key("SSH_AUTH_SOCK"));
    assert!(!sanitized.contains_key("STRIPE_SECRET_KEY"));
    assert!(!sanitized.contains_key("MY_DATABASE_URL"));
}

#[test]
fn test_sensitive_file_detection() {
    assert!(SecretShield::is_sensitive_file(Path::new(".env")));
    assert!(SecretShield::is_sensitive_file(Path::new(".env.local")));
    assert!(SecretShield::is_sensitive_file(Path::new(".env.production")));
    assert!(SecretShield::is_sensitive_file(Path::new("id_rsa")));
    assert!(SecretShield::is_sensitive_file(Path::new("server.pem")));
    assert!(SecretShield::is_sensitive_file(Path::new("key.p12")));

    assert!(!SecretShield::is_sensitive_file(Path::new("Cargo.toml")));
    assert!(!SecretShield::is_sensitive_file(Path::new("src/main.rs")));
    assert!(!SecretShield::is_sensitive_file(Path::new("README.md")));
    assert!(!SecretShield::is_sensitive_file(Path::new(".env.example")));
}

#[test]
fn test_cloak_sensitive_files() {
    let dir = tempdir().unwrap();
    let env_file = dir.path().join(".env");
    let normal_file = dir.path().join("index.js");

    fs::write(&env_file, "SECRET_TOKEN=super_secret_production_key").unwrap();
    fs::write(&normal_file, "console.log('hello');").unwrap();

    let cloaked = SecretShield::cloak_sensitive_files(dir.path()).unwrap();
    assert_eq!(cloaked.len(), 1);
    assert_eq!(cloaked[0], env_file);

    // Verify .env has been cloaked / sanitized with mock data
    let content = fs::read_to_string(&env_file).unwrap();
    assert!(!content.contains("super_secret_production_key"));
    assert!(content.contains("[SHADOWBOX_REDACTED]"));

    // Verify normal file untouched
    assert_eq!(fs::read_to_string(&normal_file).unwrap(), "console.log('hello');");
}
