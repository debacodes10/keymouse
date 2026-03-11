use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const PID_FILENAME: &str = "keymouse.pid";
const START_FLAG: &str = "--managed-run";
const STOP_TIMEOUT: Duration = Duration::from_secs(5);
const KILL_TIMEOUT: Duration = Duration::from_secs(2);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub enum StopOutcome {
    StoppedGracefully,
    StoppedForcefully,
    NotRunning,
}

pub fn start() -> Result<String, String> {
    if let Some(pid) = read_running_pid()? {
        return Ok(format!("Keymouse is already running (pid {pid})."));
    }

    cleanup_stale_pid_file()?;

    let exe = std::env::current_exe()
        .map_err(|error| format!("Failed to resolve current executable: {error}"))?;

    let child = spawn_detached(&exe)?;
    write_pid_file(child.id() as i32)?;

    Ok(format!("Started Keymouse (pid {}).", child.id()))
}

pub fn stop() -> Result<StopOutcome, String> {
    let Some(pid) = read_pid_file()? else {
        return Ok(StopOutcome::NotRunning);
    };

    if !is_process_running(pid) || !is_expected_managed_process(pid) {
        cleanup_stale_pid_file()?;
        return Ok(StopOutcome::NotRunning);
    }

    send_signal(pid, libc::SIGTERM)?;
    if wait_for_exit(pid, STOP_TIMEOUT) {
        remove_pid_file_if_exists()?;
        return Ok(StopOutcome::StoppedGracefully);
    }

    send_signal(pid, libc::SIGKILL)?;
    if wait_for_exit(pid, KILL_TIMEOUT) {
        remove_pid_file_if_exists()?;
        return Ok(StopOutcome::StoppedForcefully);
    }

    Err(format!("Failed to stop Keymouse process {pid}."))
}

pub fn restart() -> Result<String, String> {
    let stop_message = match stop()? {
        StopOutcome::StoppedGracefully => "Stopped existing Keymouse process gracefully.",
        StopOutcome::StoppedForcefully => {
            "Stopped existing Keymouse process (required force kill)."
        }
        StopOutcome::NotRunning => "Keymouse was not running.",
    };

    let start_message = start()?;
    Ok(format!("{stop_message} {start_message}"))
}

pub fn run_managed() {
    crate::menubar::run();
}

pub fn managed_run_flag() -> &'static str {
    START_FLAG
}

fn spawn_detached(executable: &std::path::Path) -> Result<std::process::Child, String> {
    use std::os::unix::process::CommandExt;

    let mut command = Command::new(executable);
    command
        .arg(START_FLAG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // SAFETY: pre_exec runs in the child process right before exec; calling setsid
    // here detaches the managed instance from the invoking terminal session.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }

    command
        .spawn()
        .map_err(|error| format!("Failed to start Keymouse: {error}"))
}

fn send_signal(pid: i32, signal: i32) -> Result<(), String> {
    // SAFETY: libc::kill is called with a pid read from our pid file.
    let result = unsafe { libc::kill(pid, signal) };
    if result == 0 {
        return Ok(());
    }

    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        return Ok(());
    }

    Err(format!("Failed to signal process {pid}: {error}"))
}

fn wait_for_exit(pid: i32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !is_process_running(pid) {
            return true;
        }
        thread::sleep(POLL_INTERVAL);
    }
    !is_process_running(pid)
}

fn read_running_pid() -> Result<Option<i32>, String> {
    let Some(pid) = read_pid_file()? else {
        return Ok(None);
    };

    if is_process_running(pid) && is_expected_managed_process(pid) {
        Ok(Some(pid))
    } else {
        cleanup_stale_pid_file()?;
        Ok(None)
    }
}

fn is_process_running(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }

    // SAFETY: signal 0 does not affect process state; it checks process existence/permissions.
    let result = unsafe { libc::kill(pid, 0) };
    if result == 0 {
        return true;
    }

    io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

fn is_expected_managed_process(pid: i32) -> bool {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output();

    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    let command = String::from_utf8_lossy(&output.stdout);
    command.contains(START_FLAG)
}

fn write_pid_file(pid: i32) -> Result<(), String> {
    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&path, pid.to_string())
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))
}

fn read_pid_file() -> Result<Option<i32>, String> {
    let path = pid_file_path();
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    let value = raw.trim();
    if value.is_empty() {
        return Ok(None);
    }

    value
        .parse::<i32>()
        .map(Some)
        .map_err(|error| format!("Invalid pid in {}: {error}", path.display()))
}

fn cleanup_stale_pid_file() -> Result<(), String> {
    let Some(pid) = read_pid_file()? else {
        return Ok(());
    };
    if !is_process_running(pid) {
        remove_pid_file_if_exists()?;
    }
    Ok(())
}

fn remove_pid_file_if_exists() -> Result<(), String> {
    let path = pid_file_path();
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("Failed to remove {}: {error}", path.display()))?;
    }
    Ok(())
}

fn pid_file_path() -> PathBuf {
    if let Ok(path) = std::env::var("KEYMOUSE_PID_FILE") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    let mut path = std::env::temp_dir();
    path.push("keymouse");
    path.push(PID_FILENAME);
    path
}
