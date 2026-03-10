use core_graphics::event::CGKeyCode;
use serde::Deserialize;
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

#[derive(Clone, Copy)]
pub enum Modifier {
    Shift,
    Option,
}

impl Modifier {
    fn from_string(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "option" | "alt" => Self::Option,
            _ => Self::Shift,
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
        let defaults = Config::default();

        Self {
            movement_up: keycode_i64(&config.movement_up, &defaults.movement_up),
            movement_down: keycode_i64(&config.movement_down, &defaults.movement_down),
            movement_left: keycode_i64(&config.movement_left, &defaults.movement_left),
            movement_right: keycode_i64(&config.movement_right, &defaults.movement_right),
            scroll_up: keycode_i64(&config.scroll_up, &defaults.scroll_up),
            scroll_down: keycode_i64(&config.scroll_down, &defaults.scroll_down),
            scroll_left: keycode_i64(&config.scroll_left, &defaults.scroll_left),
            scroll_right: keycode_i64(&config.scroll_right, &defaults.scroll_right),
            grid_key: keycode_i64(&config.grid_key, &defaults.grid_key),
            confirm_key: keycode_i64(&config.confirm_key, &defaults.confirm_key),
            left_click: keycode_i64(&config.left_click, &defaults.left_click),
            right_click: keycode_i64(&config.right_click, &defaults.right_click),
            fast_modifier: Modifier::from_string(&config.fast_modifier),
            slow_modifier: Modifier::from_string(&config.slow_modifier),
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

    if path.exists()
        && let Ok(raw) = fs::read_to_string(&path)
        && let Ok(config) = toml::from_str::<Config>(&raw)
    {
        eprintln!("Loaded Keymouse config from ~/.config/keymouse/config.toml");
        return config;
    }

    let default = Config::default();
    maybe_write_example_config(&path);
    eprintln!("Using default Keymouse configuration");
    default
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

fn keycode_i64(value: &str, fallback: &str) -> i64 {
    if let Some(code) = key_from_string(value) {
        return i64::from(code);
    }

    i64::from(key_from_string(fallback).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::{key_from_string, keycode_i64};

    #[test]
    fn parses_a_keycode_without_treating_it_as_invalid() {
        assert_eq!(key_from_string("a"), Some(0));
    }

    #[test]
    fn falls_back_for_unknown_key_names() {
        assert_eq!(keycode_i64("unknown", "k"), 40);
    }
}
