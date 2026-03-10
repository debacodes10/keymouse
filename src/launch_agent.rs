use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LABEL: &str = "com.debacodes10.keymouse";

pub fn is_enabled() -> bool {
    plist_path().exists()
}

pub fn set_enabled(enabled: bool) -> Result<bool, String> {
    if enabled {
        install()?;
        Ok(true)
    } else {
        uninstall()?;
        Ok(false)
    }
}

pub fn toggle() -> Result<bool, String> {
    set_enabled(!is_enabled())
}

fn install() -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|error| format!("Failed to resolve current executable: {error}"))?;

    let path = plist_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }

    let content = render_plist(&exe);
    fs::write(&path, content)
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))?;

    // Reload in current GUI session so it takes effect without logout.
    let domain = gui_domain();
    let _ = launchctl(&["bootout", &domain, path_as_str(&path)?]);
    launchctl(&["bootstrap", &domain, path_as_str(&path)?])
        .map_err(|error| format!("LaunchAgent installed but failed to load: {error}"))?;

    Ok(())
}

fn uninstall() -> Result<(), String> {
    let path = plist_path();
    let domain = gui_domain();

    if path.exists() {
        let _ = launchctl(&["bootout", &domain, path_as_str(&path)?]);
        fs::remove_file(&path)
            .map_err(|error| format!("Failed to remove {}: {error}", path.display()))?;
    }

    Ok(())
}

fn plist_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Library");
    path.push("LaunchAgents");
    path.push(format!("{LABEL}.plist"));
    path
}

fn gui_domain() -> String {
    let uid = std::env::var("UID")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or_else(|| {
            // Fallback for non-shell launch contexts.
            unsafe { libc::getuid() }
        });
    format!("gui/{uid}")
}

fn launchctl(args: &[&str]) -> Result<(), String> {
    let output = Command::new("launchctl")
        .args(args)
        .output()
        .map_err(|error| format!("failed to execute launchctl: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err(format!("launchctl {:?} failed", args))
    } else {
        Err(stderr)
    }
}

fn path_as_str(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("Path is not valid UTF-8: {}", path.display()))
}

fn render_plist(executable: &Path) -> String {
    let exe = escape_xml(executable.to_string_lossy().as_ref());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
      <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
  </dict>
</plist>
"#
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
