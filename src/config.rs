use core_graphics::event::CGKeyCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub movement_up: String,
    pub movement_down: String,
    pub movement_left: String,
    pub movement_right: String,
    pub scroll_up: String,
    pub scroll_down: String,
    pub scroll_left: String,
    pub scroll_right: String,
    pub grid_key: String,
    pub confirm_key: String,
    pub left_click: String,
    pub right_click: String,
    pub fast_modifier: String,
    pub slow_modifier: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            movement_up: "k".to_string(),
            movement_down: "j".to_string(),
            movement_left: "h".to_string(),
            movement_right: "l".to_string(),
            scroll_up: "u".to_string(),
            scroll_down: "n".to_string(),
            scroll_left: "b".to_string(),
            scroll_right: "m".to_string(),
            grid_key: ";".to_string(),
            confirm_key: "enter".to_string(),
            left_click: "f".to_string(),
            right_click: "d".to_string(),
            fast_modifier: "shift".to_string(),
            slow_modifier: "option".to_string(),
        }
    }
}

impl Config {
    pub fn default_toml() -> &'static str {
        r#"movement_up = "k"
movement_down = "j"
movement_left = "h"
movement_right = "l"

scroll_up = "u"
scroll_down = "n"
scroll_left = "b"
scroll_right = "m"

grid_key = ";"
confirm_key = "enter"

left_click = "f"
right_click = "d"

fast_modifier = "shift"
slow_modifier = "option"
"#
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Shift,
    Option,
}

impl Modifier {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "shift" => Some(Self::Shift),
            "option" | "alt" => Some(Self::Option),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KeyBindings {
    pub movement_up: i64,
    pub movement_down: i64,
    pub movement_left: i64,
    pub movement_right: i64,
    pub scroll_up: i64,
    pub scroll_down: i64,
    pub scroll_left: i64,
    pub scroll_right: i64,
    pub grid_key: i64,
    pub confirm_key: i64,
    pub left_click: i64,
    pub right_click: i64,
    pub fast_modifier: Modifier,
    pub slow_modifier: Modifier,
}

impl KeyBindings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            movement_up: required_keycode(&config.movement_up, "movement_up"),
            movement_down: required_keycode(&config.movement_down, "movement_down"),
            movement_left: required_keycode(&config.movement_left, "movement_left"),
            movement_right: required_keycode(&config.movement_right, "movement_right"),
            scroll_up: required_keycode(&config.scroll_up, "scroll_up"),
            scroll_down: required_keycode(&config.scroll_down, "scroll_down"),
            scroll_left: required_keycode(&config.scroll_left, "scroll_left"),
            scroll_right: required_keycode(&config.scroll_right, "scroll_right"),
            grid_key: required_keycode(&config.grid_key, "grid_key"),
            confirm_key: required_keycode(&config.confirm_key, "confirm_key"),
            left_click: required_keycode(&config.left_click, "left_click"),
            right_click: required_keycode(&config.right_click, "right_click"),
            fast_modifier: Modifier::parse(&config.fast_modifier)
                .expect("validated config must have a valid fast_modifier"),
            slow_modifier: Modifier::parse(&config.slow_modifier)
                .expect("validated config must have a valid slow_modifier"),
        }
    }
}

pub fn key_from_string(key: &str) -> Option<CGKeyCode> {
    match key.trim().to_ascii_lowercase().as_str() {
        "a" => Some(0),
        "s" => Some(1),
        "d" => Some(2),
        "f" => Some(3),
        "g" => Some(5),
        "h" => Some(4),
        "b" => Some(11),
        "j" => Some(38),
        "k" => Some(40),
        "l" => Some(37),
        "m" => Some(46),
        "n" => Some(45),
        "q" => Some(12),
        "u" => Some(32),
        "w" => Some(13),
        "e" => Some(14),
        "z" => Some(6),
        "x" => Some(7),
        "c" => Some(8),
        ";" | "semicolon" => Some(41),
        "enter" | "return" => Some(36),
        "escape" | "esc" => Some(53),
        "f8" => Some(100),
        // Modifiers are represented as flags, but we still accept them here.
        "shift" => Some(56),
        "option" | "alt" => Some(58),
        _ => None,
    }
}

pub fn load_config() -> Config {
    let path = config_path();

    if path.exists() {
        let config = match parse_config_file(&path) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

        let validation_errors = validate_config(&config);
        if !validation_errors.is_empty() {
            eprintln!("Invalid Keymouse configuration in {}:", path.display());
            for error in validation_errors {
                eprintln!("  - {}", error);
            }
            std::process::exit(1);
        }

        eprintln!("Loaded Keymouse config from {}", path.display());
        return config;
    }

    let default = Config::default();
    maybe_write_example_config(&path);
    eprintln!("Using default Keymouse configuration");
    default
}

pub fn check_config() -> Result<String, Vec<String>> {
    let path = config_path();
    if !path.exists() {
        return Ok(format!(
            "No config file found at {}. Defaults are valid.",
            path.display()
        ));
    }

    let config = match parse_config_file(&path) {
        Ok(config) => config,
        Err(error) => return Err(vec![error]),
    };

    let validation_errors = validate_config(&config);
    if validation_errors.is_empty() {
        Ok(format!("Config is valid: {}", path.display()))
    } else {
        Err(validation_errors)
    }
}

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
    path.push("keymouse");
    path.push("config.toml");
    path
}

fn maybe_write_example_config(path: &PathBuf) {
    if path.exists() {
        return;
    }

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, Config::default_toml());
}

fn parse_config_file(path: &PathBuf) -> Result<Config, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read config at {}: {}", path.display(), error))?;

    toml::from_str::<Config>(&raw)
        .map_err(|error| format!("Invalid TOML in {}: {}", path.display(), error))
}

fn required_keycode(value: &str, field_name: &str) -> i64 {
    let code = key_from_string(value).unwrap_or_else(|| {
        panic!(
            "validated config must provide a supported key for {}",
            field_name
        )
    });
    i64::from(code)
}

fn validate_config(config: &Config) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen: HashMap<String, Vec<&str>> = HashMap::new();

    for (field, value) in action_bindings(config) {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            errors.push(format!("`{}` cannot be empty.", field));
            continue;
        }

        if is_modifier_name(&normalized) {
            errors.push(format!(
                "`{}` cannot use modifier key `{}`; choose a regular key.",
                field, value
            ));
        }

        if normalized == "f8" {
            errors.push(format!(
                "`{}` cannot use `f8`; it is reserved for mouse-mode toggle.",
                field
            ));
        }

        if key_from_string(&normalized).is_none() {
            errors.push(format!(
                "`{}` has unsupported key `{}`. Use a supported key name from README.",
                field, value
            ));
        }

        seen.entry(normalized).or_default().push(field);
    }

    for (key, fields) in seen {
        if fields.len() > 1 {
            errors.push(format!(
                "Key `{}` is assigned to multiple actions: {}.",
                key,
                fields.join(", ")
            ));
        }
    }

    let fast = config.fast_modifier.trim().to_ascii_lowercase();
    let slow = config.slow_modifier.trim().to_ascii_lowercase();
    let fast_parsed = Modifier::parse(&fast);
    let slow_parsed = Modifier::parse(&slow);

    if fast_parsed.is_none() {
        errors.push(format!(
            "`fast_modifier` has unsupported value `{}`. Allowed: `shift`, `option`, `alt`.",
            config.fast_modifier
        ));
    }
    if slow_parsed.is_none() {
        errors.push(format!(
            "`slow_modifier` has unsupported value `{}`. Allowed: `shift`, `option`, `alt`.",
            config.slow_modifier
        ));
    }
    if fast_parsed.is_some() && slow_parsed.is_some() && fast_parsed == slow_parsed {
        errors.push(
            "`fast_modifier` and `slow_modifier` cannot be the same; choose distinct modifiers."
                .to_string(),
        );
    }

    errors
}

fn action_bindings(config: &Config) -> [(&'static str, &str); 12] {
    [
        ("movement_up", &config.movement_up),
        ("movement_down", &config.movement_down),
        ("movement_left", &config.movement_left),
        ("movement_right", &config.movement_right),
        ("scroll_up", &config.scroll_up),
        ("scroll_down", &config.scroll_down),
        ("scroll_left", &config.scroll_left),
        ("scroll_right", &config.scroll_right),
        ("grid_key", &config.grid_key),
        ("confirm_key", &config.confirm_key),
        ("left_click", &config.left_click),
        ("right_click", &config.right_click),
    ]
}

fn is_modifier_name(value: &str) -> bool {
    matches!(value, "shift" | "option" | "alt")
}

#[cfg(test)]
mod tests {
    use super::{Config, key_from_string, validate_config};

    #[test]
    fn parses_a_keycode_without_treating_it_as_invalid() {
        assert_eq!(key_from_string("a"), Some(0));
    }

    #[test]
    fn rejects_duplicate_action_bindings() {
        let config = Config {
            movement_up: "k".to_string(),
            movement_down: "k".to_string(),
            ..Config::default()
        };

        let errors = validate_config(&config);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("assigned to multiple actions")),
            "expected duplicate binding error, got: {errors:?}"
        );
    }

    #[test]
    fn rejects_reserved_toggle_key_in_actions() {
        let config = Config {
            left_click: "f8".to_string(),
            ..Config::default()
        };

        let errors = validate_config(&config);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("reserved for mouse-mode toggle")),
            "expected reserved key error, got: {errors:?}"
        );
    }

    #[test]
    fn rejects_invalid_modifier_values_and_duplicates() {
        let config = Config {
            fast_modifier: "hyper".to_string(),
            slow_modifier: "hyper".to_string(),
            ..Config::default()
        };

        let errors = validate_config(&config);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("fast_modifier") && error.contains("unsupported")),
            "expected invalid fast_modifier error, got: {errors:?}"
        );
    }

    #[test]
    fn rejects_same_fast_and_slow_modifiers() {
        let config = Config {
            fast_modifier: "shift".to_string(),
            slow_modifier: "shift".to_string(),
            ..Config::default()
        };

        let errors = validate_config(&config);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cannot be the same")),
            "expected modifier mismatch error, got: {errors:?}"
        );
    }

    #[test]
    fn accepts_default_config() {
        let errors = validate_config(&Config::default());
        assert!(
            errors.is_empty(),
            "expected no validation errors, got: {errors:?}"
        );
    }
}
