use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const GRID_LABEL_COUNT: usize = 9;
const GRID_LABEL_MAX_CHARS: usize = 6;
const SUPPORTED_GRID_THEMES: [&str; 4] = ["classic", "midnight", "ocean", "forest"];

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub toggle_key: String,
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
    pub drag_toggle: String,
    pub fast_modifier: String,
    pub slow_modifier: String,
    pub grid_labels: Vec<String>,
    pub grid_theme: String,
    pub grid_opacity: f64,
    pub grid_color: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            toggle_key: "f8".to_string(),
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
            drag_toggle: "v".to_string(),
            fast_modifier: "shift".to_string(),
            slow_modifier: "option".to_string(),
            grid_labels: default_grid_labels(),
            grid_theme: "classic".to_string(),
            grid_opacity: 1.0,
            grid_color: String::new(),
        }
    }
}

impl Config {
    pub fn default_toml() -> &'static str {
        r#"toggle_key = "f8"

movement_up = "k"
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
drag_toggle = "v"

fast_modifier = "shift"
slow_modifier = "option"

# Grid overlay visuals
# Available themes: "classic", "midnight", "ocean", "forest"
grid_theme = "classic"
# Opacity multiplier for the grid overlay: 0.0 to 1.0
grid_opacity = 1.0
# Optional accent color override for the grid in hex format (#RRGGBB).
# Leave empty to use the selected theme colors.
grid_color = ""
# Labels are visual only; key mapping remains Q/W/E A/S/D Z/X/C.
grid_labels = ["Q", "W", "E", "A", "S", "D", "Z", "X", "C"]
"#
    }

    pub fn grid_overlay_settings(&self) -> GridOverlaySettings {
        GridOverlaySettings {
            labels: labels_from_vec(&self.grid_labels).unwrap_or_else(default_grid_label_array),
            theme: self.grid_theme.trim().to_ascii_lowercase(),
            opacity: self.grid_opacity.clamp(0.0, 1.0),
            accent_color: parse_hex_color(&self.grid_color),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct GridOverlaySettings {
    pub labels: [String; GRID_LABEL_COUNT],
    pub theme: String,
    pub opacity: f64,
    pub accent_color: Option<(f64, f64, f64)>,
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
    pub toggle_key: i64,
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
    pub drag_toggle: i64,
    pub fast_modifier: Modifier,
    pub slow_modifier: Modifier,
}

impl KeyBindings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            toggle_key: required_keycode(&config.toggle_key, "toggle_key"),
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
            drag_toggle: required_keycode(&config.drag_toggle, "drag_toggle"),
            fast_modifier: Modifier::parse(&config.fast_modifier)
                .expect("validated config must have a valid fast_modifier"),
            slow_modifier: Modifier::parse(&config.slow_modifier)
                .expect("validated config must have a valid slow_modifier"),
        }
    }
}

pub fn key_from_string(key: &str) -> Option<u16> {
    #[cfg(target_os = "macos")]
    {
        key_from_string_macos(key)
    }

    #[cfg(target_os = "windows")]
    {
        key_from_string_windows(key)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = key;
        None
    }
}

#[cfg(target_os = "macos")]
fn key_from_string_macos(key: &str) -> Option<u16> {
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
        "v" => Some(9),
        "z" => Some(6),
        "x" => Some(7),
        "c" => Some(8),
        ";" | "semicolon" => Some(41),
        "enter" | "return" => Some(36),
        "escape" | "esc" => Some(53),
        "f1" => Some(122),
        "f2" => Some(120),
        "f3" => Some(99),
        "f4" => Some(118),
        "f5" => Some(96),
        "f6" => Some(97),
        "f7" => Some(98),
        "f8" => Some(100),
        "f9" => Some(101),
        "f10" => Some(109),
        "f11" => Some(103),
        "f12" => Some(111),
        // Modifiers are represented as flags, but we still accept them here.
        "shift" => Some(56),
        "option" | "alt" => Some(58),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn key_from_string_windows(key: &str) -> Option<u16> {
    match key.trim().to_ascii_lowercase().as_str() {
        "a" => Some(0x41),
        "s" => Some(0x53),
        "d" => Some(0x44),
        "f" => Some(0x46),
        "g" => Some(0x47),
        "h" => Some(0x48),
        "b" => Some(0x42),
        "j" => Some(0x4A),
        "k" => Some(0x4B),
        "l" => Some(0x4C),
        "m" => Some(0x4D),
        "n" => Some(0x4E),
        "q" => Some(0x51),
        "u" => Some(0x55),
        "w" => Some(0x57),
        "e" => Some(0x45),
        "v" => Some(0x56),
        "z" => Some(0x5A),
        "x" => Some(0x58),
        "c" => Some(0x43),
        ";" | "semicolon" => Some(0xBA),
        "enter" | "return" => Some(0x0D),
        "escape" | "esc" => Some(0x1B),
        "f1" => Some(0x70),
        "f2" => Some(0x71),
        "f3" => Some(0x72),
        "f4" => Some(0x73),
        "f5" => Some(0x74),
        "f6" => Some(0x75),
        "f7" => Some(0x76),
        "f8" => Some(0x77),
        "f9" => Some(0x78),
        "f10" => Some(0x79),
        "f11" => Some(0x7A),
        "f12" => Some(0x7B),
        // Modifiers are represented as flags, but we still accept them here.
        "shift" => Some(0x10),
        "option" | "alt" => Some(0x12),
        _ => None,
    }
}

pub fn load_config() -> Config {
    let path = config_path();

    if path.exists() {
        let config = match load_config_for_reload() {
            Ok(config) => config,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

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

pub fn load_config_for_reload() -> Result<Config, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }

    let config = parse_config_file(&path)?;
    let validation_errors = validate_config(&config);
    if validation_errors.is_empty() {
        Ok(config)
    } else {
        let mut message = format!("Invalid Keymouse configuration in {}:", path.display());
        for error in validation_errors {
            message.push_str("\n  - ");
            message.push_str(&error);
        }
        Err(message)
    }
}

pub fn config_path() -> PathBuf {
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
    let toggle_key = config.toggle_key.trim().to_ascii_lowercase();

    if toggle_key.is_empty() {
        errors.push("`toggle_key` cannot be empty.".to_string());
    } else {
        if is_modifier_name(&toggle_key) {
            errors.push(format!(
                "`toggle_key` cannot use modifier key `{}`; choose a regular key.",
                config.toggle_key
            ));
        }

        if key_from_string(&toggle_key).is_none() {
            errors.push(format!(
                "`toggle_key` has unsupported key `{}`. Use a supported key name from README.",
                config.toggle_key
            ));
        }
    }

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

        if !toggle_key.is_empty() && normalized == toggle_key {
            errors.push(format!(
                "`{}` cannot use `{}`; it is reserved for mouse-mode toggle.",
                field, config.toggle_key
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

    if config.grid_labels.len() != GRID_LABEL_COUNT {
        errors.push(format!(
            "`grid_labels` must contain exactly {} entries.",
            GRID_LABEL_COUNT
        ));
    } else {
        for (index, label) in config.grid_labels.iter().enumerate() {
            let trimmed = label.trim();
            if trimmed.is_empty() {
                errors.push(format!("`grid_labels[{index}]` cannot be empty."));
                continue;
            }
            if trimmed.chars().count() > GRID_LABEL_MAX_CHARS {
                errors.push(format!(
                    "`grid_labels[{index}]` is too long (max {} characters).",
                    GRID_LABEL_MAX_CHARS
                ));
            }
        }
    }

    let theme = config.grid_theme.trim().to_ascii_lowercase();
    if !SUPPORTED_GRID_THEMES.contains(&theme.as_str()) {
        errors.push(format!(
            "`grid_theme` has unsupported value `{}`. Allowed: {}.",
            config.grid_theme,
            SUPPORTED_GRID_THEMES
                .iter()
                .map(|value| format!("`{value}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !config.grid_opacity.is_finite() || !(0.0..=1.0).contains(&config.grid_opacity) {
        errors.push("`grid_opacity` must be a number between 0.0 and 1.0.".to_string());
    }
    if !config.grid_color.trim().is_empty() && parse_hex_color(&config.grid_color).is_none() {
        errors.push("`grid_color` must be a hex RGB color like `#4fd1ff`.".to_string());
    }

    errors
}

fn action_bindings(config: &Config) -> [(&'static str, &str); 13] {
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
        ("drag_toggle", &config.drag_toggle),
    ]
}

fn is_modifier_name(value: &str) -> bool {
    matches!(value, "shift" | "option" | "alt")
}

fn default_grid_labels() -> Vec<String> {
    ["Q", "W", "E", "A", "S", "D", "Z", "X", "C"]
        .iter()
        .map(|value| value.to_string())
        .collect()
}

fn default_grid_label_array() -> [String; GRID_LABEL_COUNT] {
    [
        "Q".to_string(),
        "W".to_string(),
        "E".to_string(),
        "A".to_string(),
        "S".to_string(),
        "D".to_string(),
        "Z".to_string(),
        "X".to_string(),
        "C".to_string(),
    ]
}

fn labels_from_vec(values: &[String]) -> Option<[String; GRID_LABEL_COUNT]> {
    if values.len() != GRID_LABEL_COUNT {
        return None;
    }

    let labels = values.iter().cloned().collect::<Vec<_>>();
    labels.try_into().ok()
}

fn parse_hex_color(value: &str) -> Option<(f64, f64, f64)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if hex.len() != 6 || !hex.chars().all(|value| value.is_ascii_hexdigit()) {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((
        red as f64 / 255.0,
        green as f64 / 255.0,
        blue as f64 / 255.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::{Config, key_from_string, validate_config};

    #[test]
    fn parses_a_keycode_without_treating_it_as_invalid() {
        assert_eq!(key_from_string("a"), Some(0));
    }

    #[test]
    fn supports_function_keys_for_bindings() {
        assert_eq!(key_from_string("f1"), Some(122));
        assert_eq!(key_from_string("f12"), Some(111));
    }

    #[test]
    fn parses_explicit_toggle_key_from_toml() {
        let config: Config = toml::from_str(r#"toggle_key = "f1""#).expect("valid toml");
        assert_eq!(config.toggle_key, "f1");

        let errors = validate_config(&config);
        assert!(
            errors.is_empty(),
            "expected no validation errors, got: {errors:?}"
        );
    }

    #[test]
    fn falls_back_to_default_toggle_key_when_missing_from_toml() {
        let config: Config = toml::from_str(r#"movement_up = "k""#).expect("valid toml");
        assert_eq!(config.toggle_key, "f8");

        let errors = validate_config(&config);
        assert!(
            errors.is_empty(),
            "expected no validation errors, got: {errors:?}"
        );
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
    fn allows_f8_for_actions_when_toggle_key_is_different() {
        let config = Config {
            toggle_key: "g".to_string(),
            left_click: "f8".to_string(),
            ..Config::default()
        };

        let errors = validate_config(&config);
        assert!(
            errors.is_empty(),
            "expected no validation errors, got: {errors:?}"
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

    #[test]
    fn accepts_valid_grid_color_hex() {
        let config = Config {
            grid_color: "#29ccff".to_string(),
            ..Config::default()
        };

        let errors = validate_config(&config);
        assert!(
            errors.is_empty(),
            "expected no validation errors, got: {errors:?}"
        );
    }

    #[test]
    fn rejects_invalid_grid_color() {
        let config = Config {
            grid_color: "teal".to_string(),
            ..Config::default()
        };

        let errors = validate_config(&config);
        assert!(
            errors.iter().any(|error| error.contains("`grid_color`")),
            "expected grid_color validation error, got: {errors:?}"
        );
    }
}
