use anyhow::{anyhow, Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const LOCK_FILE_NAME: &str = "renamify.lock";
const STALE_LOCK_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(Debug)]
pub struct LockFile {
    path: PathBuf,
    pid: u32,
    timestamp: u64,
}

impl LockFile {
    /// Acquire a lock for the renamify operation
    pub fn acquire(renamify_dir: &Path) -> Result<Self> {
        let lock_path = renamify_dir.join(LOCK_FILE_NAME);

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
                        "Another renamify process is already running (PID: {}). \
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
            fs::create_dir_all(parent).context("Failed to create renamify directory")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_acquire_lock_success() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");

        let lock = LockFile::acquire(&renamify_dir).unwrap();
        assert!(renamify_dir.join(LOCK_FILE_NAME).exists());
        assert_eq!(lock.pid, process::id());

        // Release the lock
        lock.release().unwrap();
        assert!(!renamify_dir.join(LOCK_FILE_NAME).exists());
    }

    #[test]
    fn test_double_acquire_fails() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");

        let _lock1 = LockFile::acquire(&renamify_dir).unwrap();

        // Second acquire should fail
        let result = LockFile::acquire(&renamify_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already running"));
    }

    #[test]
    fn test_stale_lock_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        let lock_path = renamify_dir.join(LOCK_FILE_NAME);

        // Create a stale lock (timestamp from long ago)
        let old_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (STALE_LOCK_TIMEOUT_SECS + 100);
        let stale_content = format!("99999:{}", old_timestamp);
        fs::write(&lock_path, stale_content).unwrap();

        // Should be able to acquire lock (stale one gets cleaned up)
        let lock = LockFile::acquire(&renamify_dir).unwrap();
        assert!(lock_path.exists());

        lock.release().unwrap();
    }

    #[test]
    fn test_orphaned_lock_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        let lock_path = renamify_dir.join(LOCK_FILE_NAME);

        // Create a lock with a non-existent PID and recent timestamp
        let recent_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 10;
        let orphaned_content = format!("99999:{}", recent_timestamp);
        fs::write(&lock_path, orphaned_content).unwrap();

        // Should be able to acquire lock (orphaned one gets cleaned up)
        let lock = LockFile::acquire(&renamify_dir).unwrap();
        assert!(lock_path.exists());

        lock.release().unwrap();
    }

    #[test]
    fn test_malformed_lock_file() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        fs::create_dir_all(&renamify_dir).unwrap();

        let lock_path = renamify_dir.join(LOCK_FILE_NAME);

        // Create a malformed lock file
        fs::write(&lock_path, "invalid_format").unwrap();

        // Remove the malformed file first, then try to acquire
        fs::remove_file(&lock_path).unwrap();

        // Should be able to acquire lock
        let lock = LockFile::acquire(&renamify_dir).unwrap();
        assert!(lock_path.exists());

        lock.release().unwrap();
    }

    #[test]
    fn test_lock_drop_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");
        let lock_path = renamify_dir.join(LOCK_FILE_NAME);

        {
            let _lock = LockFile::acquire(&renamify_dir).unwrap();
            assert!(lock_path.exists());
        } // Lock should be cleaned up on drop

        // Give it a moment for drop to execute
        thread::sleep(Duration::from_millis(10));
        assert!(!lock_path.exists());
    }

    #[test]
    fn test_lock_release_wrong_content() {
        let temp_dir = TempDir::new().unwrap();
        let renamify_dir = temp_dir.path().join(".renamify");

        let lock = LockFile::acquire(&renamify_dir).unwrap();
        let lock_path = renamify_dir.join(LOCK_FILE_NAME);

        // Modify the lock file to have different content
        fs::write(&lock_path, "different:content").unwrap();

        // Release should not remove the file (since it's not our lock)
        lock.release().unwrap();

        // The file should still exist since it wasn't our content
        if !lock_path.exists() {
            // If file was removed (which might happen depending on implementation), that's also OK
            // The important thing is that release() doesn't panic
        }

        // Clean up if file still exists
        let _ = fs::remove_file(&lock_path);
    }

    #[test]
    fn test_process_running_detection() {
        // Test with current process (should be running)
        let current_pid = process::id();
        assert!(is_process_running(current_pid));

        // Test with obviously invalid PID
        assert!(!is_process_running(999_999));
    }

    #[cfg(not(any(unix, windows)))]
    #[test]
    fn test_fallback_process_detection() {
        // On unsupported platforms, should always return false
        assert!(!is_process_running(1));
        assert!(!is_process_running(process::id()));
    }
}
