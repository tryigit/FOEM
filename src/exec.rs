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

pub fn run(program: &str, args: &[&str], error_prefix: &str) -> Result<String, String> {
    run_with_timeout(program, args, error_prefix, COMMAND_TIMEOUT)
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
    if path.is_empty() {
        return String::new();
    }
    let candidate = Path::new(path);
    if candidate.exists() {
        if let Ok(canonical) = candidate.canonicalize() {
            return canonical.to_string_lossy().into_owned();
        }
    }
    path.replace(['\\', '/'], std::path::MAIN_SEPARATOR_STR)
}

pub fn normalize_remote_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_normalize_local_path_empty() {
        assert_eq!(normalize_local_path(""), "");
    }

    #[test]
    fn test_normalize_local_path_non_existent() {
        let input = "a\\b/c";
        let expected = input.replace(['\\', '/'], std::path::MAIN_SEPARATOR_STR);
        assert_eq!(normalize_local_path(input), expected);
    }

    #[test]
    fn test_normalize_local_path_existing() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "hello").unwrap();

        // Use string path representation to match behavior
        let input_path = file_path.to_str().unwrap();
        let canonical_expected = file_path.canonicalize().unwrap().to_string_lossy().into_owned();

        assert_eq!(normalize_local_path(input_path), canonical_expected);
    }
}
