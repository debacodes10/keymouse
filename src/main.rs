mod config;
mod grid;
mod input;
mod platforms;
mod runtime;

#[cfg(target_os = "macos")]
mod app_bundle;
#[cfg(target_os = "macos")]
mod launch_agent;
#[cfg(target_os = "macos")]
mod menubar;
#[cfg(target_os = "macos")]
mod overlay;
#[cfg(target_os = "macos")]
mod process_control;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => run_default(),
        Some("--install-app") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            install_app();
        }
        Some("--uninstall-app") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            uninstall_app();
        }
        Some("--start") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            start_managed();
        }
        Some("--stop") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            stop_managed();
        }
        Some("--restart") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            restart_managed();
        }
        Some(flag) if is_managed_run_flag(flag) => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            run_managed();
        }
        Some("--headless") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            runtime::run_headless();
        }
        Some("--check-config") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            match config::check_config() {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(errors) => {
                    eprintln!("Config validation failed:");
                    for error in errors {
                        eprintln!("  - {}", error);
                    }
                    std::process::exit(1);
                }
            }
        }
        Some("--help") | Some("-h") => print_usage(),
        Some(flag) => {
            eprintln!("Unknown option: {}", flag);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "Keymouse - keyboard mouse control

Usage:
  keymouse                       Start app (platform-specific default)
  keymouse [command]

Commands:
  --install-app      Install ~/Applications/Keymouse.app for Spotlight/Finder launch
  --uninstall-app    Remove ~/Applications/Keymouse.app
  --start            Start as a managed background process
  --stop             Stop the managed background process
  --restart          Restart the managed background process
  --check-config     Validate config file and exit
  --headless         Run without menu bar UI
  --help, -h         Show this help

Examples:
  keymouse --install-app
  keymouse --start
  keymouse --check-config"
    );
}

#[cfg(target_os = "macos")]
fn run_default() {
    menubar::run();
}

#[cfg(target_os = "windows")]
fn run_default() {
    runtime::run_headless();
}

#[cfg(target_os = "macos")]
fn install_app() {
    match app_bundle::install_app() {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn install_app() {
    unsupported_flag("--install-app");
}

#[cfg(target_os = "macos")]
fn uninstall_app() {
    match app_bundle::uninstall_app() {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn uninstall_app() {
    unsupported_flag("--uninstall-app");
}

#[cfg(target_os = "macos")]
fn start_managed() {
    match process_control::start() {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn start_managed() {
    unsupported_flag("--start");
}

#[cfg(target_os = "macos")]
fn stop_managed() {
    match process_control::stop() {
        Ok(process_control::StopOutcome::StoppedGracefully) => {
            println!("Stopped Keymouse.");
        }
        Ok(process_control::StopOutcome::StoppedForcefully) => {
            println!("Stopped Keymouse (required force kill).");
        }
        Ok(process_control::StopOutcome::NotRunning) => {
            println!("Keymouse is not running.");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn stop_managed() {
    unsupported_flag("--stop");
}

#[cfg(target_os = "macos")]
fn restart_managed() {
    match process_control::restart() {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn restart_managed() {
    unsupported_flag("--restart");
}

#[cfg(target_os = "macos")]
fn is_managed_run_flag(flag: &str) -> bool {
    flag == process_control::managed_run_flag()
}

#[cfg(target_os = "windows")]
fn is_managed_run_flag(_flag: &str) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn run_managed() {
    process_control::run_managed();
}

#[cfg(target_os = "windows")]
fn run_managed() {
    unsupported_flag("managed run flag");
}

#[cfg(target_os = "windows")]
fn unsupported_flag(flag: &str) {
    eprintln!("{flag} is only supported on macOS.");
    std::process::exit(1);
}
