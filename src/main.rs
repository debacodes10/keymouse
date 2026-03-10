mod config;
mod grid;
mod input;
mod overlay;
mod platform;
mod runtime;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => runtime::run(),
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
    eprintln!("Usage: keymouse [--check-config]");
}
