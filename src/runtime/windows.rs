use crate::config::{self, KeyBindings, Modifier};
use crate::grid::bounds::GridBounds;
use crate::grid::recursive::RecursiveGrid;
use crate::input::{
    KEYCODE_ESCAPE, display_index_for_keycode, grid_cell_for_keycode, movement_step_from_modifiers,
    scroll_step_from_modifiers,
};
use crate::platforms::{
    KEY_EVENT_DOWN_TYPES, KEY_EVENT_UP_TYPES, call_next_keyboard_hook, cursor_position,
    display_for_point, install_keyboard_hook, is_keyboard_action, keyboard_message_keycode,
    run_message_loop, uninstall_keyboard_hook,
};
use enigo::{Enigo, MouseButton, MouseControllable};
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::OnceLock;
use std::thread_local;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

const VK_SHIFT: i64 = 0x10;
const VK_MENU: i64 = 0x12;
const VK_LSHIFT: i64 = 0xA0;
const VK_RSHIFT: i64 = 0xA1;
const VK_LMENU: i64 = 0xA4;
const VK_RMENU: i64 = 0xA5;

static KEY_BINDINGS: OnceLock<KeyBindings> = OnceLock::new();
static KEYBOARD_HOOK: OnceLock<isize> = OnceLock::new();
static MOUSE_MODE_LISTENER: OnceLock<fn(bool)> = OnceLock::new();

struct AppState {
    enigo: Enigo,
    mouse_mode: bool,
    drag_active: bool,
    grid: RecursiveGrid,
    held_keys: HashSet<i64>,
}

impl AppState {
    fn new() -> Self {
        Self {
            enigo: Enigo::new(),
            mouse_mode: false,
            drag_active: false,
            grid: RecursiveGrid::new(),
            held_keys: HashSet::new(),
        }
    }

    fn release_drag_if_active(&mut self) {
        if self.drag_active {
            self.enigo.mouse_up(MouseButton::Left);
            self.drag_active = false;
        }
    }
}

thread_local! {
    static APP_STATE: RefCell<AppState> = RefCell::new(AppState::new());
}

fn apply_mouse_mode_state(state: &mut AppState, enabled: bool) {
    let changed = state.mouse_mode != enabled;
    state.mouse_mode = enabled;
    if !enabled {
        state.grid.cancel();
        state.release_drag_if_active();
        state.held_keys.clear();
    }
    if changed && let Some(listener) = MOUSE_MODE_LISTENER.get() {
        listener(enabled);
    }
}

pub fn initialize() {
    if KEY_BINDINGS.get().is_some() {
        return;
    }

    let config = config::load_config();
    let toggle_key_name = config.toggle_key.trim().to_ascii_lowercase();
    let bindings = KeyBindings::from_config(&config);
    let _ = KEY_BINDINGS.set(bindings);
    eprintln!("Toggle key binding: {}", toggle_key_name);

    match install_keyboard_hook(keyboard_callback) {
        Ok(hook) => {
            let _ = KEYBOARD_HOOK.set(hook);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

pub fn run_headless() {
    initialize();
    eprintln!("keymouse running (headless). Press configured toggle key to toggle mouse mode.");
    if let Err(error) = run_message_loop() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

pub fn mouse_mode_enabled() -> bool {
    APP_STATE.with(|cell| cell.borrow().mouse_mode)
}

pub fn set_mouse_mode_listener(listener: fn(bool)) {
    let _ = MOUSE_MODE_LISTENER.set(listener);
}

pub fn set_mouse_mode(enabled: bool) {
    APP_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        apply_mouse_mode_state(&mut state, enabled);
    });
}

pub fn shutdown() {
    set_mouse_mode(false);
    if let Some(hook) = KEYBOARD_HOOK.get() {
        uninstall_keyboard_hook(*hook);
    }
}

unsafe extern "system" fn keyboard_callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if !is_keyboard_action(code) {
        return call_next_keyboard_hook(code, wparam, lparam);
    }

    let message = wparam as u32;
    let is_key_down = KEY_EVENT_DOWN_TYPES.contains(&message);
    let is_key_up = KEY_EVENT_UP_TYPES.contains(&message);
    if !is_key_down && !is_key_up {
        return call_next_keyboard_hook(code, wparam, lparam);
    }

    let Some(keycode) = keyboard_message_keycode(lparam) else {
        return call_next_keyboard_hook(code, wparam, lparam);
    };
    let keycode = i64::from(keycode);

    APP_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        let bindings = *KEY_BINDINGS
            .get()
            .expect("key bindings must be initialized");
        let is_first_keydown = if is_key_down {
            state.held_keys.insert(keycode)
        } else {
            state.held_keys.remove(&keycode);
            false
        };

        if keycode == bindings.toggle_key && is_first_keydown {
            let next = !state.mouse_mode;
            apply_mouse_mode_state(&mut state, next);
            eprintln!("mouse mode: {}", if next { "on" } else { "off" });
            return 1;
        }

        if !state.mouse_mode {
            return call_next_keyboard_hook(code, wparam, lparam);
        }

        if is_first_keydown && keycode == bindings.grid_key {
            if let Some(point) = cursor_position() {
                let display = display_for_point(point).or_else(|| {
                    crate::platforms::active_displays_sorted()
                        .into_iter()
                        .next()
                });
                if let Some(display) = display {
                    let bounds = GridBounds::from_rect(
                        f64::from(display.bounds.left),
                        f64::from(display.bounds.top),
                        f64::from(display.bounds.right - display.bounds.left),
                        f64::from(display.bounds.bottom - display.bounds.top),
                    );
                    state.grid.start(bounds);
                }
            }
            return 1;
        }

        if state.grid.is_active() {
            if is_key_down
                && let Some(display_index) = display_index_for_keycode(keycode)
                && let Some(display) = crate::platforms::active_displays_sorted().get(display_index)
            {
                let bounds = GridBounds::from_rect(
                    f64::from(display.bounds.left),
                    f64::from(display.bounds.top),
                    f64::from(display.bounds.right - display.bounds.left),
                    f64::from(display.bounds.bottom - display.bounds.top),
                );
                state.grid.start(bounds);
                return 1;
            }

            if is_first_keydown {
                if keycode == bindings.confirm_key {
                    if let Some(final_bounds) = state.grid.confirm() {
                        let (target_x, target_y) = final_bounds.center();
                        state.enigo.mouse_move_to(target_x, target_y);
                    }
                    return 1;
                }

                if keycode == KEYCODE_ESCAPE {
                    state.grid.cancel();
                    return 1;
                }

                if let Some((row, col)) = grid_cell_for_keycode(keycode) {
                    state.grid.zoom_into_cell(row, col);
                }
            }
            return 1;
        }

        if is_key_down {
            let fast_active = modifier_active(bindings.fast_modifier, &state.held_keys);
            let slow_active = modifier_active(bindings.slow_modifier, &state.held_keys);
            let move_step = movement_step_from_modifiers(fast_active, slow_active);
            let scroll_step = scroll_step_from_modifiers(fast_active, slow_active);
            match keycode {
                key if key == bindings.movement_left => {
                    state.enigo.mouse_move_relative(-move_step, 0);
                }
                key if key == bindings.movement_down => {
                    state.enigo.mouse_move_relative(0, move_step);
                }
                key if key == bindings.movement_up => {
                    state.enigo.mouse_move_relative(0, -move_step);
                }
                key if key == bindings.movement_right => {
                    state.enigo.mouse_move_relative(move_step, 0);
                }
                key if key == bindings.scroll_up => {
                    state.enigo.mouse_scroll_y(-scroll_step);
                }
                key if key == bindings.scroll_down => {
                    state.enigo.mouse_scroll_y(scroll_step);
                }
                key if key == bindings.scroll_left => {
                    state.enigo.mouse_scroll_x(-scroll_step);
                }
                key if key == bindings.scroll_right => {
                    state.enigo.mouse_scroll_x(scroll_step);
                }
                key if key == bindings.left_click => {
                    if is_first_keydown {
                        state.enigo.mouse_click(MouseButton::Left);
                    }
                }
                key if key == bindings.right_click => {
                    if is_first_keydown {
                        state.enigo.mouse_click(MouseButton::Right);
                    }
                }
                key if key == bindings.drag_toggle => {
                    if is_first_keydown {
                        if state.drag_active {
                            state.enigo.mouse_up(MouseButton::Left);
                        } else {
                            state.enigo.mouse_down(MouseButton::Left);
                        }
                        state.drag_active = !state.drag_active;
                    }
                }
                _ => {}
            }
        }

        1
    })
}

fn modifier_active(modifier: Modifier, held_keys: &HashSet<i64>) -> bool {
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
