mod app_bundle;
mod config;
mod grid;
mod input;
mod launch_agent;
mod menubar;
mod overlay;
mod platforms;
mod process_control;
mod runtime;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => menubar::run(),
        Some("--install-app") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            match app_bundle::install_app() {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        Some("--uninstall-app") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            match app_bundle::uninstall_app() {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        Some("--start") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            match process_control::start() {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        Some("--stop") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
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
        Some("--restart") => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            match process_control::restart() {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        Some(flag) if flag == process_control::managed_run_flag() => {
            if let Some(extra) = args.next() {
                eprintln!("Unexpected argument: {}", extra);
                print_usage();
                std::process::exit(1);
            }
            process_control::run_managed();
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
        "Keymouse - macOS keyboard mouse control

Usage:
  keymouse                       Start menu bar app
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
