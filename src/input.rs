use crate::config::{KeyBindings, Modifier};

#[cfg(target_os = "macos")]
pub const KEYCODE_D: i64 = 2;
#[cfg(target_os = "windows")]
pub const KEYCODE_D: i64 = 0x44;
#[cfg(target_os = "macos")]
pub const KEYCODE_F: i64 = 3;
#[cfg(target_os = "windows")]
pub const KEYCODE_F: i64 = 0x46;
#[cfg(target_os = "macos")]
pub const KEYCODE_G: i64 = 5;
#[cfg(target_os = "windows")]
pub const KEYCODE_G: i64 = 0x47;
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
#[cfg(target_os = "macos")]
pub const KEYCODE_Q: i64 = 12;
#[cfg(target_os = "windows")]
pub const KEYCODE_Q: i64 = 0x51;
#[cfg(target_os = "macos")]
pub const KEYCODE_W: i64 = 13;
#[cfg(target_os = "windows")]
pub const KEYCODE_W: i64 = 0x57;
#[cfg(target_os = "macos")]
pub const KEYCODE_E: i64 = 14;
#[cfg(target_os = "windows")]
pub const KEYCODE_E: i64 = 0x45;
#[cfg(target_os = "macos")]
pub const KEYCODE_A: i64 = 0;
#[cfg(target_os = "windows")]
pub const KEYCODE_A: i64 = 0x41;
#[cfg(target_os = "macos")]
pub const KEYCODE_S: i64 = 1;
#[cfg(target_os = "windows")]
pub const KEYCODE_S: i64 = 0x53;
#[cfg(target_os = "macos")]
pub const KEYCODE_Z: i64 = 6;
#[cfg(target_os = "windows")]
pub const KEYCODE_Z: i64 = 0x5A;
#[cfg(target_os = "macos")]
pub const KEYCODE_X: i64 = 7;
#[cfg(target_os = "windows")]
pub const KEYCODE_X: i64 = 0x58;
#[cfg(target_os = "macos")]
pub const KEYCODE_C: i64 = 8;
#[cfg(target_os = "windows")]
pub const KEYCODE_C: i64 = 0x43;

const EVENT_FLAG_MASK_SHIFT: u64 = 1 << 17;
const EVENT_FLAG_MASK_OPTION: u64 = 1 << 19;
const NORMAL_SPEED: i32 = 20;
const FAST_SPEED: i32 = 120;
const SLOW_SPEED: i32 = 5;
const NORMAL_SCROLL: i32 = 8;
const FAST_SCROLL: i32 = 24;
const SLOW_SCROLL: i32 = 1;

pub fn grid_cell_for_keycode(keycode: i64) -> Option<(i32, i32)> {
    match keycode {
        KEYCODE_Q => Some((0, 0)),
        KEYCODE_W => Some((0, 1)),
        KEYCODE_E => Some((0, 2)),
        KEYCODE_A => Some((1, 0)),
        KEYCODE_S => Some((1, 1)),
        KEYCODE_D => Some((1, 2)),
        KEYCODE_Z => Some((2, 0)),
        KEYCODE_X => Some((2, 1)),
        KEYCODE_C => Some((2, 2)),
        // Alternate bindings to support home-row recursive selection examples.
        KEYCODE_F => Some((1, 0)),
        KEYCODE_G => Some((1, 1)),
        _ => None,
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

pub fn movement_step(flags: u64, bindings: KeyBindings) -> i32 {
    if modifier_active(flags, bindings.fast_modifier) {
        FAST_SPEED
    } else if modifier_active(flags, bindings.slow_modifier) {
        SLOW_SPEED
    } else {
        NORMAL_SPEED
    }
}

pub fn scroll_step(flags: u64, bindings: KeyBindings) -> i32 {
    if modifier_active(flags, bindings.fast_modifier) {
        FAST_SCROLL
    } else if modifier_active(flags, bindings.slow_modifier) {
        SLOW_SCROLL
    } else {
        NORMAL_SCROLL
    }
}

#[cfg(target_os = "windows")]
pub fn movement_step_from_modifiers(fast_active: bool, slow_active: bool) -> i32 {
    if fast_active {
        FAST_SPEED
    } else if slow_active {
        SLOW_SPEED
    } else {
        NORMAL_SPEED
    }
}

#[cfg(target_os = "windows")]
pub fn scroll_step_from_modifiers(fast_active: bool, slow_active: bool) -> i32 {
    if fast_active {
        FAST_SCROLL
    } else if slow_active {
        SLOW_SCROLL
    } else {
        NORMAL_SCROLL
    }
}

fn modifier_active(flags: u64, modifier: Modifier) -> bool {
    match modifier {
        Modifier::Shift => (flags & EVENT_FLAG_MASK_SHIFT) != 0,
        Modifier::Option => (flags & EVENT_FLAG_MASK_OPTION) != 0,
    }
}
