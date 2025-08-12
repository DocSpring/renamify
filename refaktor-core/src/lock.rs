use anyhow::{anyhow, Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const LOCK_FILE_NAME: &str = "refaktor.lock";
const STALE_LOCK_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(Debug)]
pub struct LockFile {
    path: PathBuf,
    pid: u32,
    timestamp: u64,
}

impl LockFile {
    /// Acquire a lock for the refaktor operation
    pub fn acquire(refaktor_dir: &Path) -> Result<Self> {
        let lock_path = refaktor_dir.join(LOCK_FILE_NAME);

        // Check if lock file exists
        if lock_path.exists() {
            // Read existing lock file
            let mut content = String::new();
            File::open(&lock_path)
                .context("Failed to read lock file")?
                .read_to_string(&mut content)
                .context("Failed to read lock file content")?;

            // Parse lock file content (format: "pid:timestamp")
            let parts: Vec<&str> = content.trim().split(':').collect();
            if parts.len() == 2 {
                let pid = parts[0].parse::<u32>().unwrap_or(0);
                let timestamp = parts[1].parse::<u64>().unwrap_or(0);

                // Check if the lock is stale
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if current_time - timestamp > STALE_LOCK_TIMEOUT_SECS {
                    // Lock is stale, remove it
                    fs::remove_file(&lock_path).context("Failed to remove stale lock file")?;
                } else if is_process_running(pid) {
                    // Process is still running
                    return Err(anyhow!(
                        "Another refaktor process is already running (PID: {}). \
                        If this is incorrect, remove the lock file at: {}",
                        pid,
                        lock_path.display()
                    ));
                } else {
                    // Process is not running, remove the lock
                    fs::remove_file(&lock_path).context("Failed to remove orphaned lock file")?;
                }
            }
        }

        // Create the lock file
        let pid = process::id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let lock_content = format!("{}:{}", pid, timestamp);

        // Ensure the directory exists
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).context("Failed to create refaktor directory")?;
        }

        // Write lock file atomically
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true) // Fail if file exists (race condition protection)
            .open(&lock_path)
            .context("Failed to create lock file")?;

        file.write_all(lock_content.as_bytes())
            .context("Failed to write lock file")?;

        Ok(Self {
            path: lock_path,
            pid,
            timestamp,
        })
    }

    /// Release the lock
    pub fn release(self) -> Result<()> {
        if self.path.exists() {
            // Verify it's our lock before removing
            let mut content = String::new();
            File::open(&self.path)
                .context("Failed to read lock file")?
                .read_to_string(&mut content)
                .context("Failed to read lock file content")?;

            let expected_content = format!("{}:{}", self.pid, self.timestamp);
            if content.trim() == expected_content {
                fs::remove_file(&self.path).context("Failed to remove lock file")?;
            }
        }
        Ok(())
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        // Best effort cleanup on drop
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Check if a process with the given PID is running
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    // On Unix, we can check if a process exists by sending signal 0
    #[allow(clippy::cast_possible_wrap)]
    unsafe {
        libc::kill(pid as libc::pid_t, 0) == 0
    }
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winnt::PROCESS_QUERY_INFORMATION;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
        if handle.is_null() {
            false
        } else {
            CloseHandle(handle);
            true
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn is_process_running(_pid: u32) -> bool {
    // Fallback: assume process is not running if we can't check
    false
}
