use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP_NAME: &str = "Keymouse";
const BUNDLE_ID: &str = "com.debacodes10.keymouse";
const LAUNCHER_NAME: &str = "Keymouse";

pub fn install_app() -> Result<String, String> {
    let app_path = app_bundle_path()?;
    let launcher_target = std::env::current_exe()
        .map_err(|error| format!("Failed to resolve current executable: {error}"))?;

    if app_path.exists() {
        if should_prompt()
            && !confirm(&format!(
                "{APP_NAME}.app already exists at {}.\nReplace it?",
                app_path.display()
            ))?
        {
            return Ok("Cancelled app install.".to_string());
        }
        fs::remove_dir_all(&app_path).map_err(|error| {
            format!("Failed to remove existing {}: {error}", app_path.display())
        })?;
    }

    let contents_dir = app_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");
    fs::create_dir_all(&macos_dir)
        .map_err(|error| format!("Failed to create {}: {error}", macos_dir.display()))?;
    fs::create_dir_all(&resources_dir)
        .map_err(|error| format!("Failed to create {}: {error}", resources_dir.display()))?;

    let plist_path = contents_dir.join("Info.plist");
    fs::write(&plist_path, render_info_plist())
        .map_err(|error| format!("Failed to write {}: {error}", plist_path.display()))?;

    let launcher_path = macos_dir.join(LAUNCHER_NAME);
    fs::write(&launcher_path, render_launcher(&launcher_target))
        .map_err(|error| format!("Failed to write {}: {error}", launcher_path.display()))?;
    set_executable(&launcher_path)?;

    let lsregister_notice = match refresh_launch_services(&app_path) {
        Ok(()) => "Spotlight indexing refresh requested.",
        Err(_) => "Spotlight will discover the app automatically shortly.",
    };

    Ok(format!(
        "Installed {} at {}\n{}\nOpen Spotlight and type \"{}\" to launch.",
        APP_NAME,
        app_path.display(),
        lsregister_notice,
        APP_NAME
    ))
}

pub fn uninstall_app() -> Result<String, String> {
    let app_path = app_bundle_path()?;
    if !app_path.exists() {
        return Ok(format!(
            "{APP_NAME}.app is not installed at {}.",
            app_path.display()
        ));
    }

    if should_prompt() && !confirm(&format!("Remove {} from {}?", APP_NAME, app_path.display()))? {
        return Ok("Cancelled app uninstall.".to_string());
    }

    fs::remove_dir_all(&app_path)
        .map_err(|error| format!("Failed to remove {}: {error}", app_path.display()))?;
    Ok(format!("Removed {} from {}.", APP_NAME, app_path.display()))
}

fn app_bundle_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    Ok(home.join("Applications").join(format!("{APP_NAME}.app")))
}

fn render_info_plist() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>CFBundleName</key>
    <string>{APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>{APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>{BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>{LAUNCHER_NAME}</string>
    <key>LSUIElement</key>
    <true/>
  </dict>
</plist>
"#
    )
}

fn render_launcher(target: &Path) -> String {
    let escaped = shell_single_quote(target.to_string_lossy().as_ref());
    format!("#!/bin/sh\nexec '{}' \"$@\"\n", escaped)
}

fn set_executable(path: &Path) -> Result<(), String> {
    let mut perms = fs::metadata(path)
        .map_err(|error| format!("Failed to read {} metadata: {error}", path.display()))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).map_err(|error| {
        format!(
            "Failed to set executable permissions on {}: {error}",
            path.display()
        )
    })
}

fn refresh_launch_services(app_path: &Path) -> Result<(), String> {
    let tool = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";
    Command::new(tool)
        .arg("-f")
        .arg(app_path)
        .output()
        .map_err(|error| format!("Failed to run lsregister: {error}"))
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err("lsregister exited with non-zero status".to_string())
            }
        })
}

fn confirm(prompt: &str) -> Result<bool, String> {
    print!("{prompt} [y/N]: ");
    io::stdout()
        .flush()
        .map_err(|error| format!("Failed to flush prompt: {error}"))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| format!("Failed to read user input: {error}"))?;

    let normalized = input.trim().to_ascii_lowercase();
    Ok(normalized == "y" || normalized == "yes")
}

fn should_prompt() -> bool {
    // SAFETY: libc::isatty only reads descriptor properties.
    unsafe { libc::isatty(libc::STDIN_FILENO) == 1 && libc::isatty(libc::STDOUT_FILENO) == 1 }
}

fn shell_single_quote(value: &str) -> String {
    value.replace('\'', "'\"'\"'")
}
