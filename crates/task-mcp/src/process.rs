//! Process management helpers.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use crate::error::{Error, Result};
use crate::state::TaskInfo;

/// Check if a process with the given PID is still alive.
///
/// Uses signal 0 which checks process existence without sending a signal.
pub fn is_alive(pid: u32) -> bool {
    let pid = Pid::from_raw(i32::try_from(pid).unwrap_or(0));
    kill(pid, None).is_ok()
}

/// Create temp file paths for stdout/stderr logs.
fn create_log_paths(name: &str) -> (PathBuf, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let stdout_path = temp_dir.join(format!("task-mcp-{name}-stdout.log"));
    let stderr_path = temp_dir.join(format!("task-mcp-{name}-stderr.log"));
    (stdout_path, stderr_path)
}

/// Spawn a process with output redirected to log files.
pub fn spawn_task(name: &str, command: &str, cwd: Option<&Path>) -> Result<TaskInfo> {
    let (stdout_path, stderr_path) = create_log_paths(name);

    // Create/truncate log files
    let stdout_file = File::create(&stdout_path)?;
    let stderr_file = File::create(&stderr_path)?;

    // Build command - use shell for complex commands
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    // Redirect stdout/stderr to files
    cmd.stdout(Stdio::from(stdout_file));
    cmd.stderr(Stdio::from(stderr_file));

    let child = cmd.spawn().map_err(|e| Error::SpawnFailed {
        name: name.to_string(),
        reason: e.to_string(),
    })?;

    let pid = child.id();

    Ok(TaskInfo {
        name: name.to_string(),
        pid,
        command: command.to_string(),
        cwd: cwd.map(Path::to_path_buf),
        stdout_path,
        stderr_path,
        started_at: Instant::now(),
    })
}

/// Send SIGTERM to a process.
pub fn terminate(pid: u32) -> Result<()> {
    let pid = Pid::from_raw(i32::try_from(pid).unwrap_or(0));
    kill(pid, Signal::SIGTERM).map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
}

/// Read the tail of a log file.
pub async fn read_log_tail(path: &Path, lines: usize) -> Result<String> {
    let content = tokio::fs::read_to_string(path).await?;
    let all_lines: Vec<&str> = content.lines().collect();
    let start = all_lines.len().saturating_sub(lines);
    Ok(all_lines[start..].join("\n"))
}

/// Remove log files for a task.
pub async fn cleanup_logs(stdout_path: &Path, stderr_path: &Path) {
    let _ = tokio::fs::remove_file(stdout_path).await;
    let _ = tokio::fs::remove_file(stderr_path).await;
}
