use crate::config::{KeyBindings, Modifier};
#[cfg(target_os = "windows")]
use std::collections::HashSet;

#[cfg(target_os = "macos")]
pub const KEYCODE_ESCAPE: i64 = 53;
#[cfg(target_os = "windows")]
pub const KEYCODE_ESCAPE: i64 = 0x1B;
#[cfg(target_os = "macos")]
pub const KEYCODE_1: i64 = 18;
#[cfg(target_os = "windows")]
pub const KEYCODE_1: i64 = 0x31;
#[cfg(target_os = "macos")]
pub const KEYCODE_2: i64 = 19;
#[cfg(target_os = "windows")]
pub const KEYCODE_2: i64 = 0x32;
#[cfg(target_os = "macos")]
pub const KEYCODE_3: i64 = 20;
#[cfg(target_os = "windows")]
pub const KEYCODE_3: i64 = 0x33;
#[cfg(target_os = "macos")]
pub const KEYCODE_4: i64 = 21;
#[cfg(target_os = "windows")]
pub const KEYCODE_4: i64 = 0x34;
#[cfg(target_os = "macos")]
pub const KEYCODE_5: i64 = 23;
#[cfg(target_os = "windows")]
pub const KEYCODE_5: i64 = 0x35;
#[cfg(target_os = "macos")]
pub const KEYCODE_6: i64 = 22;
#[cfg(target_os = "windows")]
pub const KEYCODE_6: i64 = 0x36;
#[cfg(target_os = "macos")]
pub const KEYCODE_7: i64 = 26;
#[cfg(target_os = "windows")]
pub const KEYCODE_7: i64 = 0x37;
#[cfg(target_os = "macos")]
pub const KEYCODE_8: i64 = 28;
#[cfg(target_os = "windows")]
pub const KEYCODE_8: i64 = 0x38;
#[cfg(target_os = "macos")]
pub const KEYCODE_9: i64 = 25;
#[cfg(target_os = "windows")]
pub const KEYCODE_9: i64 = 0x39;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_1: i64 = 83;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_1: i64 = 0x61;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_2: i64 = 84;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_2: i64 = 0x62;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_3: i64 = 85;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_3: i64 = 0x63;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_4: i64 = 86;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_4: i64 = 0x64;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_5: i64 = 87;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_5: i64 = 0x65;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_6: i64 = 88;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_6: i64 = 0x66;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_7: i64 = 89;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_7: i64 = 0x67;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_8: i64 = 91;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_8: i64 = 0x68;
#[cfg(target_os = "macos")]
pub const KEYCODE_NUMPAD_9: i64 = 92;
#[cfg(target_os = "windows")]
pub const KEYCODE_NUMPAD_9: i64 = 0x69;
const NORMAL_SPEED: i32 = 20;
const FAST_SPEED: i32 = 120;
const SLOW_SPEED: i32 = 5;
const NORMAL_SCROLL: i32 = 8;
const FAST_SCROLL: i32 = 24;
const SLOW_SCROLL: i32 = 1;

pub fn grid_cell_for_keycode(keycode: i64, bindings: &KeyBindings) -> Option<(i32, i32)> {
    bindings
        .grid_selection_keys
        .iter()
        .position(|configured_keycode| *configured_keycode == keycode)
        .map(|index| ((index / 3) as i32, (index % 3) as i32))
}

#[cfg(test)]
pub fn keybindings_with_grid_selection_keys(keys: [i64; 9]) -> KeyBindings {
    KeyBindings {
        toggle_key: 0,
        movement_up: 0,
        movement_down: 0,
        movement_left: 0,
        movement_right: 0,
        scroll_up: 0,
        scroll_down: 0,
        scroll_left: 0,
        scroll_right: 0,
        grid_key: 0,
        grid_selection_keys: keys,
        confirm_key: 0,
        left_click: 0,
        right_click: 0,
        drag_toggle: 0,
        fast_modifier: Modifier::Shift,
        slow_modifier: Modifier::Option,
    }
}

pub fn display_index_for_keycode(keycode: i64) -> Option<usize> {
    match keycode {
        KEYCODE_1 => Some(0),
        KEYCODE_NUMPAD_1 => Some(0),
        KEYCODE_2 => Some(1),
        KEYCODE_NUMPAD_2 => Some(1),
        KEYCODE_3 => Some(2),
        KEYCODE_NUMPAD_3 => Some(2),
        KEYCODE_4 => Some(3),
        KEYCODE_NUMPAD_4 => Some(3),
        KEYCODE_5 => Some(4),
        KEYCODE_NUMPAD_5 => Some(4),
        KEYCODE_6 => Some(5),
        KEYCODE_NUMPAD_6 => Some(5),
        KEYCODE_7 => Some(6),
        KEYCODE_NUMPAD_7 => Some(6),
        KEYCODE_8 => Some(7),
        KEYCODE_NUMPAD_8 => Some(7),
        KEYCODE_9 => Some(8),
        KEYCODE_NUMPAD_9 => Some(8),
        _ => None,
    }
}

pub fn movement_step_from_modifiers(fast_active: bool, slow_active: bool) -> i32 {
    if fast_active {
        FAST_SPEED
    } else if slow_active {
        SLOW_SPEED
    } else {
        NORMAL_SPEED
    }
}

pub fn scroll_step_from_modifiers(fast_active: bool, slow_active: bool) -> i32 {
    if fast_active {
        FAST_SCROLL
    } else if slow_active {
        SLOW_SCROLL
    } else {
        NORMAL_SCROLL
    }
}

#[cfg(target_os = "macos")]
const EVENT_FLAG_MASK_SHIFT: u64 = 1 << 17;
#[cfg(target_os = "macos")]
const EVENT_FLAG_MASK_OPTION: u64 = 1 << 19;

#[cfg(target_os = "windows")]
const VK_SHIFT: i64 = 0x10;
#[cfg(target_os = "windows")]
const VK_MENU: i64 = 0x12;
#[cfg(target_os = "windows")]
const VK_LSHIFT: i64 = 0xA0;
#[cfg(target_os = "windows")]
const VK_RSHIFT: i64 = 0xA1;
#[cfg(target_os = "windows")]
const VK_LMENU: i64 = 0xA4;
#[cfg(target_os = "windows")]
const VK_RMENU: i64 = 0xA5;

#[cfg(target_os = "macos")]
pub fn modifier_states_from_event_flags(flags: u64, bindings: KeyBindings) -> (bool, bool) {
    (
        event_flag_modifier_active(flags, bindings.fast_modifier),
        event_flag_modifier_active(flags, bindings.slow_modifier),
    )
}

#[cfg(target_os = "windows")]
pub fn modifier_states_from_held_keys(
    bindings: KeyBindings,
    held_keys: &HashSet<i64>,
) -> (bool, bool) {
    (
        held_key_modifier_active(bindings.fast_modifier, held_keys),
        held_key_modifier_active(bindings.slow_modifier, held_keys),
    )
}

#[cfg(target_os = "macos")]
fn event_flag_modifier_active(flags: u64, modifier: Modifier) -> bool {
    match modifier {
        Modifier::Shift => (flags & EVENT_FLAG_MASK_SHIFT) != 0,
        Modifier::Option => (flags & EVENT_FLAG_MASK_OPTION) != 0,
    }
}

#[cfg(target_os = "windows")]
fn held_key_modifier_active(modifier: Modifier, held_keys: &HashSet<i64>) -> bool {
    match modifier {
        Modifier::Shift => {
            held_keys.contains(&VK_SHIFT)
                || held_keys.contains(&VK_LSHIFT)
                || held_keys.contains(&VK_RSHIFT)
        }
        Modifier::Option => {
            held_keys.contains(&VK_MENU)
                || held_keys.contains(&VK_LMENU)
                || held_keys.contains(&VK_RMENU)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        grid_cell_for_keycode, keybindings_with_grid_selection_keys, movement_step_from_modifiers,
        scroll_step_from_modifiers,
    };

    #[test]
    fn movement_step_prioritizes_fast_modifier() {
        assert_eq!(movement_step_from_modifiers(true, true), 120);
        assert_eq!(movement_step_from_modifiers(true, false), 120);
        assert_eq!(movement_step_from_modifiers(false, true), 5);
        assert_eq!(movement_step_from_modifiers(false, false), 20);
    }

    #[test]
    fn scroll_step_prioritizes_fast_modifier() {
        assert_eq!(scroll_step_from_modifiers(true, true), 24);
        assert_eq!(scroll_step_from_modifiers(true, false), 24);
        assert_eq!(scroll_step_from_modifiers(false, true), 1);
        assert_eq!(scroll_step_from_modifiers(false, false), 8);
    }

    #[test]
    fn grid_cell_lookup_uses_configured_selection_keys() {
        let bindings = keybindings_with_grid_selection_keys([10, 11, 12, 13, 14, 15, 16, 17, 18]);

        assert_eq!(grid_cell_for_keycode(10, &bindings), Some((0, 0)));
        assert_eq!(grid_cell_for_keycode(15, &bindings), Some((1, 2)));
        assert_eq!(grid_cell_for_keycode(18, &bindings), Some((2, 2)));
        assert_eq!(grid_cell_for_keycode(999, &bindings), None);
    }
}
