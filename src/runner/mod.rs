use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, ExitStatus, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;

pub struct PtyHandle {
    child: Box<dyn Child + Send + Sync>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    captured_output: Vec<u8>,
}

impl PtyHandle {
    /// Reads all remaining output from the child process until EOF.
    pub fn read_all_output(&mut self) -> Result<Vec<u8>> {
        let mut buf = [0u8; 4096];
        loop {
            match self.reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    self.captured_output.extend_from_slice(&buf[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    // On macOS/Linux PTY EOF often produces EIO (Input/output error)
                    if e.raw_os_error() == Some(5) {
                        break;
                    }
                    return Err(e.into());
                }
            }
        }
        Ok(self.captured_output.clone())
    }

    /// Reads currently available output without blocking indefinitely.
    pub fn read_available(&mut self) -> Result<Vec<u8>> {
        let mut buf = [0u8; 4096];
        let mut new_bytes = Vec::new();

        loop {
            match self.reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    new_bytes.extend_from_slice(&buf[..n]);
                    self.captured_output.extend_from_slice(&buf[..n]);
                    if n < buf.len() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    if e.raw_os_error() == Some(5) {
                        break;
                    }
                    return Err(e.into());
                }
            }
        }

        Ok(new_bytes)
    }

    /// Writes raw input bytes to the child process stdin.
    pub fn write_input(&mut self, input: &[u8]) -> Result<()> {
        self.writer.write_all(input)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Waits for the child process to exit and returns the exit status.
    pub fn wait(&mut self) -> Result<ExitStatus> {
        self.child.wait().context("Failed to wait on child process")
    }

    /// Checks if the child process has exited without blocking.
    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.child
            .try_wait()
            .context("Failed to check child process status")
    }

    /// Terminates the child process.
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().context("Failed to kill child process")
    }

    /// Returns a reference to all captured output so far.
    pub fn captured_output(&self) -> &[u8] {
        &self.captured_output
    }
}

pub struct PtyRunner;

impl PtyRunner {
    /// Spawns a command inside a virtual pseudo-terminal (PTY) with given working directory and environment.
    pub fn spawn(command: &[String], cwd: &Path, env: &HashMap<String, String>) -> Result<PtyHandle> {
        if command.is_empty() {
            anyhow::bail!("Command cannot be empty");
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 30,
                cols: 100,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY pair")?;

        let mut cmd = CommandBuilder::new(&command[0]);
        for arg in &command[1..] {
            cmd.arg(arg);
        }

        cmd.cwd(cwd);

        // Apply environment variables
        for (k, v) in env {
            cmd.env(k, v);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn command in PTY")?;

        // Drop the slave end in parent process so EOF works correctly
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY master reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY master writer")?;

        Ok(PtyHandle {
            child,
            reader,
            writer,
            captured_output: Vec::new(),
        })
    }
}
