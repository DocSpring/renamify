#[cfg(test)]
#[cfg(unix)] // Signal tests only work on Unix-like systems
#[allow(unused_mut)]
mod signal_tests {
    use assert_cmd::Command as AssertCommand;
    use std::fs;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    fn get_renamify_binary() -> PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop(); // Remove test binary name
        if path.ends_with("deps") {
            path.pop();
        }
        path.push("renamify");
        path
    }

    fn create_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let lock_file = temp_dir.path().join(".renamify").join("renamify.lock");
        (temp_dir, lock_file)
    }

    #[test]
    fn test_lock_normal_completion() {
        let (temp_dir, lock_file) = create_test_env();

        let output = AssertCommand::new(get_renamify_binary())
            .args(["test-lock", "--delay", "100"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run command");

        assert!(output.status.success(), "Command should succeed");
        assert!(
            !lock_file.exists(),
            "Lock file should be removed after completion"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Acquiring lock..."));
        assert!(stderr.contains("Lock acquired"));
        assert!(stderr.contains("Sleep complete"));
    }

    #[test]
    fn test_lock_conflict() {
        let (temp_dir, lock_file) = create_test_env();

        // Start first command that will hold lock for 2 seconds
        let mut child1 = Command::new(get_renamify_binary())
            .args(["test-lock", "--delay", "2000"])
            .current_dir(temp_dir.path())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn first command");

        // Wait a bit for first command to acquire lock
        thread::sleep(Duration::from_millis(200));
        assert!(
            lock_file.exists(),
            "Lock file should exist while first command runs"
        );

        // Try second command - should fail due to lock conflict
        let output2 = AssertCommand::new(get_renamify_binary())
            .args(["test-lock", "--delay", "100"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run second command");

        assert!(
            !output2.status.success(),
            "Second command should fail due to lock"
        );

        let stderr2 = String::from_utf8_lossy(&output2.stderr);
        assert!(stderr2.contains("Another renamify process is already running"));

        // Clean up first command
        child1.kill().expect("Failed to kill first command");
        child1.wait().expect("Failed to wait for first command");
    }

    #[test]
    fn test_sigint_cleanup() {
        let (temp_dir, lock_file) = create_test_env();

        let mut child = Command::new(get_renamify_binary())
            .args(["test-lock", "--delay", "5000"]) // Long delay
            .current_dir(temp_dir.path())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        // Wait for lock to be acquired
        thread::sleep(Duration::from_millis(200));
        assert!(lock_file.exists(), "Lock file should exist");

        // Send SIGINT (Ctrl-C)
        unsafe {
            libc::kill(child.id() as i32, libc::SIGINT);
        }

        // Wait for process to exit
        let output = child.wait_with_output().expect("Failed to wait for child");

        // Should exit with code 130 (128 + SIGINT)
        assert_eq!(
            output.status.code(),
            Some(130),
            "Should exit with SIGINT code"
        );

        // Give it a moment for cleanup
        thread::sleep(Duration::from_millis(100));

        // Lock file should be cleaned up
        assert!(
            !lock_file.exists(),
            "Lock file should be removed after SIGINT"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Interrupted"));
    }

    #[test]
    fn test_sigterm_cleanup() {
        let (temp_dir, lock_file) = create_test_env();

        let mut child = Command::new(get_renamify_binary())
            .args(["test-lock", "--delay", "5000"]) // Long delay
            .current_dir(temp_dir.path())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        // Wait for lock to be acquired
        thread::sleep(Duration::from_millis(200));
        assert!(lock_file.exists(), "Lock file should exist");

        // Send SIGTERM
        unsafe {
            libc::kill(child.id() as i32, libc::SIGTERM);
        }

        // Wait for process to exit
        let output = child.wait_with_output().expect("Failed to wait for child");

        // Should exit with code 130 (graceful shutdown)
        assert_eq!(
            output.status.code(),
            Some(130),
            "Should exit with SIGTERM code"
        );

        // Give it a moment for cleanup
        thread::sleep(Duration::from_millis(100));

        // Lock file should be cleaned up
        assert!(
            !lock_file.exists(),
            "Lock file should be removed after SIGTERM"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Interrupted"));
    }

    #[test]
    fn test_sigkill_no_cleanup() {
        let (temp_dir, lock_file) = create_test_env();

        let mut child = Command::new(get_renamify_binary())
            .args(["test-lock", "--delay", "5000"]) // Long delay
            .current_dir(temp_dir.path())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        // Wait for lock to be acquired
        thread::sleep(Duration::from_millis(200));
        assert!(lock_file.exists(), "Lock file should exist");

        // Send SIGKILL (kill -9) - no cleanup possible
        unsafe {
            libc::kill(child.id() as i32, libc::SIGKILL);
        }

        // Wait for process to be killed
        let output = child.wait_with_output().expect("Failed to wait for child");

        // Should be killed by signal (no exit code)
        assert!(output.status.code().is_none(), "Should be killed by signal");

        // Give it a moment, but lock file should still exist since no cleanup
        thread::sleep(Duration::from_millis(100));

        // Lock file should still exist since SIGKILL prevents cleanup
        assert!(
            lock_file.exists(),
            "Lock file should still exist after SIGKILL"
        );

        // Manually clean up for next tests
        let _ = fs::remove_file(&lock_file);
    }

    #[test]
    fn test_stale_lock_cleanup() {
        let (temp_dir, lock_file) = create_test_env();

        // Create a stale lock file manually (simulating killed process)
        fs::create_dir_all(lock_file.parent().unwrap()).expect("Failed to create .renamify dir");
        let fake_pid = 99999; // PID that definitely doesn't exist
        let old_timestamp = 1000000000; // Very old timestamp
        fs::write(&lock_file, format!("{}:{}", fake_pid, old_timestamp))
            .expect("Failed to write stale lock file");

        assert!(lock_file.exists(), "Stale lock file should exist");

        // Try to run test-lock - should clean up stale lock and succeed
        let output = AssertCommand::new(get_renamify_binary())
            .args(["test-lock", "--delay", "100"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run command");

        assert!(
            output.status.success(),
            "Command should succeed after cleaning stale lock"
        );
        assert!(
            !lock_file.exists(),
            "Lock file should be removed after completion"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Lock acquired"));
    }

    #[test]
    fn test_zero_delay() {
        let (temp_dir, lock_file) = create_test_env();

        let start = Instant::now();
        let output = AssertCommand::new(get_renamify_binary())
            .args(["test-lock", "--delay", "0"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run command");

        let elapsed = start.elapsed();

        assert!(output.status.success(), "Command should succeed");
        assert!(
            elapsed < Duration::from_millis(500),
            "Should complete quickly with zero delay"
        );
        assert!(
            !lock_file.exists(),
            "Lock file should be removed after completion"
        );
    }
}
