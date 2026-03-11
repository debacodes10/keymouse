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
    let launcher_target = resolve_launcher_target()?;

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

    let bundled_executable_path = macos_dir.join(LAUNCHER_NAME);
    fs::copy(&launcher_target, &bundled_executable_path).map_err(|error| {
        format!(
            "Failed to copy executable to {}: {error}",
            bundled_executable_path.display()
        )
    })?;
    set_executable(&bundled_executable_path)?;

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

fn resolve_launcher_target() -> Result<PathBuf, String> {
    let current = std::env::current_exe()
        .map_err(|error| format!("Failed to resolve current executable: {error}"))?;

    if !is_arm64_hardware() {
        return Ok(current);
    }

    if binary_supports_arm64(&current) {
        return Ok(current);
    }

    let cargo_bin = cargo_bin_keymouse_path()?;
    if binary_supports_arm64(&cargo_bin) {
        return Ok(cargo_bin);
    }

    Err(
        "This Mac is Apple Silicon, but the current keymouse binary is not arm64.\n\
Run keymouse from a native (non-Rosetta) terminal and reinstall it:\n\
  cargo install keymouse\n\
Then run:\n\
  keymouse --install-app"
            .to_string(),
    )
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

fn cargo_bin_keymouse_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    Ok(home.join(".cargo").join("bin").join("keymouse"))
}

fn is_arm64_hardware() -> bool {
    let output = Command::new("sysctl")
        .args(["-n", "hw.optional.arm64"])
        .output();

    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout).trim() == "1"
}

fn binary_supports_arm64(path: &Path) -> bool {
    let output = Command::new("lipo").args(["-archs"]).arg(path).output();

    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .any(|arch| arch == "arm64")
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
