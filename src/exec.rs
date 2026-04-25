use std::io;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);

fn spawn_with_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<Output, io::Error> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let start = Instant::now();

    loop {
        if let Some(_status) = child.try_wait()? {
            return child.wait_with_output();
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timed out after {}s", timeout.as_secs()),
            ));
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

pub fn run_with_timeout(
    program: &str,
    args: &[&str],
    error_prefix: &str,
    timeout: Duration,
) -> Result<String, String> {
    let attempt = |binary: &str| -> Result<String, String> {
        let output = spawn_with_timeout(binary, args, timeout)
            .map_err(|e| format!("{error_prefix}: {}", e))?;

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut message = if !stderr.is_empty() { stderr } else { stdout };
        if message.is_empty() {
            message = format!(
                "{error_prefix}: command exited with status {}",
                output.status
            );
        }
        if message.contains("device not found")
            || message.contains("no devices/emulators found")
            || message.contains("offline")
        {
            message.push_str(" (device disconnected or USB debugging not authorized)");
        }
        Err(message)
    };

    match attempt(program) {
        Err(e) if cfg!(windows) && e.contains("No such file") && !program.ends_with(".exe") => {
            attempt(&format!("{program}.exe"))
        }
        result => result,
    }
}

#[cfg(test)]
thread_local! {
    pub static MOCK_RUN_IMPL: std::cell::RefCell<Option<Box<dyn Fn(&str, &[&str], &str) -> Result<String, String>>>> = std::cell::RefCell::new(None);
}

#[cfg(not(test))]
pub fn run(program: &str, args: &[&str], error_prefix: &str) -> Result<String, String> {
    run_with_timeout(program, args, error_prefix, COMMAND_TIMEOUT)
}

#[cfg(test)]
pub fn run(program: &str, args: &[&str], error_prefix: &str) -> Result<String, String> {
    let mut mock_result = None;
    MOCK_RUN_IMPL.with(|mock| {
        if let Some(f) = mock.borrow().as_ref() {
            mock_result = Some(f(program, args, error_prefix));
        }
    });

    if let Some(result) = mock_result {
        result
    } else {
        run_with_timeout(program, args, error_prefix, COMMAND_TIMEOUT)
    }
}

pub fn run_with_serial(
    program: &str,
    serial: &str,
    args: &[&str],
    error_prefix: &str,
) -> Result<String, String> {
    let mut full_args = Vec::with_capacity(args.len() + 2);
    full_args.push("-s");
    full_args.push(serial);
    full_args.extend_from_slice(args);
    run(program, &full_args, error_prefix)
}

pub fn normalize_local_path(path: &str) -> String {
    let expanded = shellexpand::tilde(path).into_owned();
    match std::fs::canonicalize(&expanded) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => expanded,
    }
}

pub fn normalize_remote_path(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_run_with_serial_prepends_args() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(|program, args, error_prefix| {
                assert_eq!(program, "test_prog");
                assert_eq!(args, &["-s", "SERIAL123", "arg1", "arg2"]);
                assert_eq!(error_prefix, "test_error");
                Ok("success".to_string())
            }));
        });

        let result = run_with_serial("test_prog", "SERIAL123", &["arg1", "arg2"], "test_error");
        assert_eq!(result, Ok("success".to_string()));

        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_run_with_serial_empty_args() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(|program, args, _error_prefix| {
                assert_eq!(program, "adb");
                assert_eq!(args, &["-s", "1234"]);
                Ok("".to_string())
            }));
        });

        let result = run_with_serial("adb", "1234", &[], "err");
        assert_eq!(result, Ok("".to_string()));

        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    #[test]
    fn test_run_with_serial_propagates_error() {
        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = Some(Box::new(|_, _, _| Err("mock error".to_string())));
        });

        let result = run_with_serial("adb", "1234", &[], "err");
        assert_eq!(result, Err("mock error".to_string()));

        MOCK_RUN_IMPL.with(|mock| {
            *mock.borrow_mut() = None;
        });
    }

    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_normalize_local_path_symlink_cycle() -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let dir = std::env::temp_dir();
            let link1 = dir.join("link1_cycle");
            let link2 = dir.join("link2_cycle");

            let _ = std::fs::remove_file(&link1);
            let _ = std::fs::remove_file(&link2);

            symlink(&link2, &link1)?;
            symlink(&link1, &link2)?;

            let input_path = link1.to_str().ok_or("Invalid UTF-8 in path")?;
            // Canonicalize fails on symlink loop, so it falls back to the expanded string.
            assert_eq!(normalize_local_path(input_path), input_path);

            let _ = std::fs::remove_file(&link1);
            let _ = std::fs::remove_file(&link2);
        }
        Ok(())
    }

    #[test]
    fn test_normalize_local_path_unc_paths() {
        let unc_path = r"\\server\share\file.txt";
        // Canonicalize will fail on a non-existent UNC path, fallback is just the input string (or expanded)
        assert_eq!(normalize_local_path(unc_path), unc_path);
    }

    #[test]
    fn test_normalize_local_path_tilde() {
        let tilde_path = "~/some_non_existent_file.txt";
        let expected = shellexpand::tilde(tilde_path).into_owned();
        assert_eq!(normalize_local_path(tilde_path), expected);
    }

    #[test]
    fn test_normalize_local_path_empty() {
        assert_eq!(normalize_local_path(""), "");
    }

    #[test]
    fn test_normalize_local_path_non_existent() {
        let input = "a\\b/c";
        let expected = shellexpand::tilde(input).into_owned();
        assert_eq!(normalize_local_path(input), expected);
    }

    #[test]
    fn test_normalize_local_path_existing() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let file_path = dir.join("test_file.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "hello")?;

        // Use string path representation to match behavior
        let input_path = file_path.to_str().ok_or("Invalid UTF-8 in path")?;
        let canonical_expected = file_path.canonicalize()?.to_string_lossy().into_owned();

        assert_eq!(normalize_local_path(input_path), canonical_expected);
        Ok(())
    }

    #[test]
    fn test_normalize_local_path_invalid_chars() {
        // Test with invalid path characters (like null byte)
        let invalid_path = "invalid\0path\x01\x02";
        let expected = shellexpand::tilde(invalid_path).into_owned();
        assert_eq!(normalize_local_path(invalid_path), expected);

        // Test with very long path
        let long_path = "a/b\\c".repeat(1000);
        let expected_long = shellexpand::tilde(&long_path).into_owned();
        assert_eq!(normalize_local_path(&long_path), expected_long);
    }

    #[test]
    fn test_normalize_remote_path() {
        assert_eq!(normalize_remote_path("a\\b\\c"), "a/b/c");
        assert_eq!(normalize_remote_path("a/b\\c"), "a/b/c");
        assert_eq!(normalize_remote_path("a/b/"), "a/b");
        assert_eq!(normalize_remote_path("a/b"), "a/b");
        assert_eq!(normalize_remote_path(""), "");
        assert_eq!(normalize_remote_path("/"), "");
    }
}
