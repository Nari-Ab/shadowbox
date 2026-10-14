use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SecretShield;

impl SecretShield {
    /// Sanitizes environment variables passed to child processes, removing sensitive credentials.
    pub fn sanitize_env(env: &HashMap<String, String>) -> HashMap<String, String> {
        let blocked_prefixes = [
            "AWS_",
            "AZURE_",
            "GCP_",
            "GOOGLE_",
            "GITHUB_TOKEN",
            "GH_TOKEN",
            "GITLAB_TOKEN",
            "SSH_",
            "STRIPE_",
            "SLACK_",
            "SENDGRID_",
            "DATABASE_URL",
            "REDIS_URL",
            "MONGO_URL",
        ];

        let blocked_substrings = [
            "SECRET",
            "PASSWORD",
            "PRIVATE_KEY",
            "ACCESS_KEY",
            "AUTH_TOKEN",
            "API_KEY",
            "BEARER",
            "DATABASE_URL",
            "DB_PASS",
            "PRIVATE",
        ];

        let mut sanitized = HashMap::new();

        for (key, val) in env {
            let key_upper = key.to_uppercase();

            let matches_prefix = blocked_prefixes.iter().any(|p| key_upper.starts_with(p));
            let matches_substring = blocked_substrings.iter().any(|s| key_upper.contains(s));

            if !matches_prefix && !matches_substring {
                sanitized.insert(key.clone(), val.clone());
            }
        }

        sanitized
    }

    /// Checks if a file path represents a sensitive file that should not be exposed to an AI agent.
    pub fn is_sensitive_file(path: &Path) -> bool {
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        // Whitelisted safe example/template files
        if filename == ".env.example"
            || filename == ".env.sample"
            || filename == ".env.template"
            || filename == ".env.dist"
        {
            return false;
        }

        // Sensitive env files
        if filename == ".env" || filename.starts_with(".env.") {
            return true;
        }

        // SSH and private keys
        if filename.starts_with("id_rsa")
            || filename.starts_with("id_ed25519")
            || filename.starts_with("id_ecdsa")
            || filename.ends_with(".pem")
            || filename.ends_with(".key")
            || filename.ends_with(".p12")
            || filename.ends_with(".pfx")
            || filename.ends_with(".keystore")
        {
            return true;
        }

        false
    }

    /// Cloaks sensitive files in the sandbox workspace by replacing secret values with harmless mocks.
    pub fn cloak_sensitive_files(workspace: &Path) -> Result<Vec<PathBuf>> {
        let mut cloaked = Vec::new();

        for entry in WalkDir::new(workspace).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && Self::is_sensitive_file(path) {
                // Read and sanitize file contents
                if let Ok(content) = fs::read_to_string(path) {
                    let mut cloaked_lines = Vec::new();
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with('#') || trimmed.is_empty() {
                            cloaked_lines.push(line.to_string());
                        } else if let Some((k, _)) = line.split_once('=') {
                            cloaked_lines.push(format!("{}=[SHADOWBOX_REDACTED]", k));
                        } else {
                            cloaked_lines.push("[SHADOWBOX_REDACTED]".to_string());
                        }
                    }
                    fs::write(path, cloaked_lines.join("\n") + "\n")?;
                } else {
                    // Binary or non-UTF8 sensitive file
                    fs::write(path, b"[SHADOWBOX_REDACTED_BINARY_KEY]\n")?;
                }
                cloaked.push(path.to_path_buf());
            }
        }

        Ok(cloaked)
    }
}
