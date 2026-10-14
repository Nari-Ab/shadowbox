use anyhow::Result;
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub unified_diff: String,
    pub additions: usize,
    pub deletions: usize,
}

pub struct DiffEngine;

impl DiffEngine {
    /// Compares two directories and produces a list of unified file diffs.
    pub fn diff_dirs(base_dir: &Path, current_dir: &Path) -> Result<Vec<FileDiff>> {
        let mut diffs = Vec::new();
        let mut current_files = HashSet::new();

        // 1. Scan current_dir for added and modified files
        for entry in WalkDir::new(current_dir).min_depth(1) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let relative = path.strip_prefix(current_dir)?;

            // Skip internal meta folders
            if relative.starts_with(".shadowbox") || relative.starts_with(".git") {
                continue;
            }

            current_files.insert(relative.to_path_buf());
            let base_file = base_dir.join(relative);

            if !base_file.exists() {
                // File was added
                let current_content = fs::read_to_string(path).unwrap_or_default();
                let additions = current_content.lines().count();
                diffs.push(FileDiff {
                    path: relative.to_path_buf(),
                    change_type: ChangeType::Added,
                    unified_diff: format!("+++ b/{}\n@@ -0,0 +1,{} @@\n{}", relative.display(), additions, current_content),
                    additions,
                    deletions: 0,
                });
            } else {
                // File exists in both, check if content changed
                let base_content = fs::read_to_string(&base_file).unwrap_or_default();
                let current_content = fs::read_to_string(path).unwrap_or_default();

                if base_content != current_content {
                    let text_diff = TextDiff::from_lines(&base_content, &current_content);
                    let mut unified_diff = String::new();
                    let mut additions = 0;
                    let mut deletions = 0;

                    for change in text_diff.iter_all_changes() {
                        match change.tag() {
                            ChangeTag::Insert => {
                                additions += 1;
                                unified_diff.push('+');
                                unified_diff.push_str(change.value());
                            }
                            ChangeTag::Delete => {
                                deletions += 1;
                                unified_diff.push('-');
                                unified_diff.push_str(change.value());
                            }
                            ChangeTag::Equal => {
                                unified_diff.push(' ');
                                unified_diff.push_str(change.value());
                            }
                        }
                    }

                    diffs.push(FileDiff {
                        path: relative.to_path_buf(),
                        change_type: ChangeType::Modified,
                        unified_diff,
                        additions,
                        deletions,
                    });
                }
            }
        }

        // 2. Scan base_dir for deleted files
        for entry in WalkDir::new(base_dir).min_depth(1) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let relative = path.strip_prefix(base_dir)?;

            if relative.starts_with(".shadowbox") || relative.starts_with(".git") {
                continue;
            }

            if !current_files.contains(relative) {
                let base_content = fs::read_to_string(path).unwrap_or_default();
                let deletions = base_content.lines().count();
                diffs.push(FileDiff {
                    path: relative.to_path_buf(),
                    change_type: ChangeType::Deleted,
                    unified_diff: format!("--- a/{}\n@@ -1,{} +0,0 @@\n{}", relative.display(), deletions, base_content),
                    additions: 0,
                    deletions,
                });
            }
        }

        // Sort diffs by path for deterministic output
        diffs.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(diffs)
    }
}
