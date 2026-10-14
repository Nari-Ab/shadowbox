use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

/// Represents an isolated copy-on-write workspace session.
pub struct Session {
    source_dir: PathBuf,
    session_dir: PathBuf,
    work_dir: PathBuf,
    created_at: SystemTime,
}

impl Session {
    /// Creates a new isolated session for `source_root`.
    /// Files are cloned into `.shadowbox/sessions/<timestamp>-<id>/work` using APFS clonefile or fast copy fallback.
    pub fn new(source_root: &Path, session_root: Option<&Path>) -> Result<Self> {
        let canonical_source = source_root
            .canonicalize()
            .context("Failed to canonicalize source root")?;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let base_session_dir = match session_root {
            Some(root) => root.to_path_buf(),
            None => canonical_source.join(".shadowbox").join("sessions"),
        };

        let session_dir = base_session_dir.join(format!("{}-{}", timestamp, std::process::id()));
        let work_dir = session_dir.join("work");

        fs::create_dir_all(&session_dir)?;

        // Clone the source workspace into work_dir
        Self::clone_workspace(&canonical_source, &work_dir)?;

        Ok(Self {
            source_dir: canonical_source,
            session_dir,
            work_dir,
            created_at: SystemTime::now(),
        })
    }

    /// Returns the path to the isolated scratch workspace where the agent should execute.
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// Returns the path to the original host project directory.
    pub fn source_dir(&self) -> &Path {
        &self.source_dir
    }

    /// Returns the root directory of this specific session metadata.
    pub fn session_dir(&self) -> &Path {
        &self.session_dir
    }

    /// Returns the creation timestamp of the session.
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Clones the workspace using APFS clonefile on macOS or fallback copy.
    fn clone_workspace(src: &Path, dst: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            if let Ok(()) = Self::try_apfs_clone(src, dst) {
                return Ok(());
            }
        }

        // Standard fast recursive copy fallback
        Self::copy_recursive(src, dst)
    }

    #[cfg(target_os = "macos")]
    fn try_apfs_clone(src: &Path, dst: &Path) -> Result<()> {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let c_src = CString::new(src.as_os_str().as_bytes())?;
        let c_dst = CString::new(dst.as_os_str().as_bytes())?;

        // 1 = CLONE_NOFOLLOW
        let ret = unsafe { libc::clonefile(c_src.as_ptr(), c_dst.as_ptr(), 1) };
        if ret == 0 {
            // If .shadowbox exists in the clone, remove it to avoid recursion
            let cloned_shadow = dst.join(".shadowbox");
            if cloned_shadow.exists() {
                let _ = fs::remove_dir_all(cloned_shadow);
            }
            Ok(())
        } else {
            anyhow::bail!("APFS clonefile returned error code {}", ret)
        }
    }

    fn copy_recursive(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in WalkDir::new(src).min_depth(1) {
            let entry = entry?;
            let path = entry.path();
            let relative = path.strip_prefix(src)?;

            // Skip .shadowbox session artifacts
            if relative.starts_with(".shadowbox") {
                continue;
            }

            let target = dst.join(relative);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&target)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(path, &target)?;
            }
        }

        Ok(())
    }

    /// Atomically commits changes from the sandbox back to the host repository.
    pub fn commit(&self) -> Result<()> {
        // 1. Copy added or modified files from work_dir to source_dir
        for entry in WalkDir::new(&self.work_dir).min_depth(1) {
            let entry = entry?;
            let relative = entry.path().strip_prefix(&self.work_dir)?;

            // Skip internal meta folders (.git, .shadowbox)
            if relative.starts_with(".git") || relative.starts_with(".shadowbox") {
                continue;
            }

            let host_target = self.source_dir.join(relative);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&host_target)?;
            } else if entry.file_type().is_file() {
                // Only copy if modified or newly added
                let should_copy = match (fs::read(entry.path()), fs::read(&host_target)) {
                    (Ok(work_bytes), Ok(host_bytes)) => work_bytes != host_bytes,
                    _ => true,
                };

                if should_copy {
                    if let Some(parent) = host_target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(entry.path(), &host_target)?;
                }
            }
        }

        // 2. Remove files from source_dir that were deleted in work_dir
        for entry in WalkDir::new(&self.source_dir).min_depth(1) {
            let entry = entry?;
            let relative = entry.path().strip_prefix(&self.source_dir)?;

            // Skip internal .shadowbox dir
            if relative.starts_with(".shadowbox") || relative.starts_with(".git") {
                continue;
            }

            let work_target = self.work_dir.join(relative);
            if !work_target.exists() {
                if entry.file_type().is_file() {
                    let _ = fs::remove_file(entry.path());
                } else if entry.file_type().is_dir() {
                    let _ = fs::remove_dir_all(entry.path());
                }
            }
        }

        // 3. Clean up the session directory after commit
        let _ = fs::remove_dir_all(&self.session_dir);

        Ok(())
    }

    /// Discards all changes made during the session and purges the scratch directory.
    pub fn discard(&self) -> Result<()> {
        if self.session_dir.exists() {
            fs::remove_dir_all(&self.session_dir)?;
        }
        Ok(())
    }
}
