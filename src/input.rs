use crate::config::{KeyBindings, Modifier};

pub const KEYCODE_D: i64 = 2;
pub const KEYCODE_F: i64 = 3;
pub const KEYCODE_G: i64 = 5;
pub const KEYCODE_ESCAPE: i64 = 53;
pub const KEYCODE_Q: i64 = 12;
pub const KEYCODE_W: i64 = 13;
pub const KEYCODE_E: i64 = 14;
pub const KEYCODE_A: i64 = 0;
pub const KEYCODE_S: i64 = 1;
pub const KEYCODE_Z: i64 = 6;
pub const KEYCODE_X: i64 = 7;
pub const KEYCODE_C: i64 = 8;

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

fn modifier_active(flags: u64, modifier: Modifier) -> bool {
    match modifier {
        Modifier::Shift => (flags & EVENT_FLAG_MASK_SHIFT) != 0,
        Modifier::Option => (flags & EVENT_FLAG_MASK_OPTION) != 0,
    }
}
