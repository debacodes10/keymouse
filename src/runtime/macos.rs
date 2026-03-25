use crate::config::{self, KeyBindings};
use crate::grid::bounds::GridBounds;
use crate::grid::recursive::RecursiveGrid;
use crate::input::{
    KEYCODE_ESCAPE, display_index_for_keycode, grid_cell_for_keycode,
    modifier_states_from_event_flags, movement_step_from_modifiers, scroll_step_from_modifiers,
};
use crate::overlay::Overlay;
use crate::platforms::{
    CFMachPortCreateRunLoopSource, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun,
    CGEventGetFlags, CGEventGetIntegerValueField, CGEventGetLocation, CGEventTapCreate,
    CGEventTapEnable, KEYBOARD_EVENT_KEYCODE, active_displays_sorted, display_for_point,
    event_mask,
};
use core_foundation::runloop::kCFRunLoopCommonModes;
use core_graphics::event::{
    CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use core_graphics::sys::CGEventRef;
use enigo::{Enigo, MouseButton, MouseControllable};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::fs;
use std::ptr;
use std::sync::OnceLock;
use std::thread_local;
use std::time::{Duration, Instant, SystemTime};

static KEY_BINDINGS: OnceLock<KeyBindings> = OnceLock::new();
static EVENT_TAP: OnceLock<usize> = OnceLock::new();
static MOUSE_MODE_LISTENER: OnceLock<fn(bool)> = OnceLock::new();
const CONFIG_POLL_INTERVAL: Duration = Duration::from_millis(500);
const HUD_ACTION_DURATION: Duration = Duration::from_millis(1400);
const HUD_UNKNOWN_KEY_HINT_COOLDOWN: Duration = Duration::from_millis(900);

unsafe extern "C" {
    fn NSBeep();
}

struct AppState {
    enigo: Enigo,
    mouse_mode: bool,
    drag_active: bool,
    grid: RecursiveGrid,
    overlay: Overlay,
    held_keys: HashSet<i64>,
    grid_overlay_settings: config::GridOverlaySettings,
    hud_settings: config::HudSettings,
    config_last_modified: Option<SystemTime>,
    last_config_poll: Instant,
    last_reload_error: Option<String>,
    last_action_message: Option<String>,
    last_action_until: Option<Instant>,
    last_unknown_hint_at: Option<Instant>,
}

impl AppState {
    fn new() -> Self {
        let mut overlay = Overlay::new();
        let grid_overlay_settings = config::Config::default().grid_overlay_settings();
        overlay.apply_settings(grid_overlay_settings.clone());
        let hud_settings = config::Config::default().hud_settings();
        overlay.apply_hud_settings(hud_settings);

        Self {
            enigo: Enigo::new(),
            mouse_mode: false,
            drag_active: false,
            grid: RecursiveGrid::new(),
            overlay,
            held_keys: HashSet::new(),
            grid_overlay_settings,
            hud_settings,
            config_last_modified: None,
            last_config_poll: Instant::now(),
            last_reload_error: None,
            last_action_message: None,
            last_action_until: None,
            last_unknown_hint_at: None,
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
        state.overlay.hide();
        state.release_drag_if_active();
        state.held_keys.clear();
        state.last_action_message = None;
        state.last_action_until = None;
        state.last_unknown_hint_at = None;
    } else {
        update_hud_overlay(state);
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
    let initial_overlay_settings = config.grid_overlay_settings();
    let initial_hud_settings = config.hud_settings();
    let initial_modified = current_config_modified_time();
    let bindings = KeyBindings::from_config(&config);
    let _ = KEY_BINDINGS.set(bindings);
    eprintln!("Toggle key binding: {}", toggle_key_name);
    APP_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.grid_overlay_settings = initial_overlay_settings.clone();
        state.overlay.apply_settings(initial_overlay_settings);
        state.hud_settings = initial_hud_settings;
        state.overlay.apply_hud_settings(initial_hud_settings);
        state.config_last_modified = initial_modified;
    });

    let mask = event_mask(&[CGEventType::KeyDown, CGEventType::KeyUp]);

    // SAFETY: CoreGraphics/CoreFoundation APIs are called with valid arguments.
    unsafe {
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            mask,
            keyboard_callback,
            ptr::null_mut(),
        );

        if tap.is_null() {
            eprintln!(
                "Failed to create event tap. Enable Accessibility for this binary in System Settings -> Privacy & Security -> Accessibility."
            );
            std::process::exit(1);
        }
        let _ = EVENT_TAP.set(tap as usize);

        let run_loop_source = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
        if run_loop_source.is_null() {
            eprintln!("Failed to create run loop source for event tap.");
            std::process::exit(1);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(
            current_loop,
            run_loop_source,
            kCFRunLoopCommonModes as *const c_void,
        );
        CGEventTapEnable(tap, true);
    }
}

pub fn run_headless() {
    initialize();
    eprintln!("keymouse running (headless). Press configured toggle key to toggle mouse mode.");
    // SAFETY: Run loop is initialized and ready to process event-tap callbacks.
    unsafe { CFRunLoopRun() }
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
}

unsafe extern "C" fn keyboard_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    if matches!(
        event_type,
        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
    ) {
        // Event-tap disable can drop key-up events; clear held state so key
        // presses keep working after re-enable in both debug and release builds.
        APP_STATE.with(|cell| cell.borrow_mut().held_keys.clear());
        if let Some(tap) = EVENT_TAP.get() {
            // SAFETY: tap comes from successful CGEventTapCreate and remains valid
            // for the lifetime of the process run loop.
            unsafe { CGEventTapEnable(*tap as *mut _, true) };
        }
        return event;
    }

    match event_type {
        CGEventType::KeyDown | CGEventType::KeyUp => {}
        _ => return event,
    }

    // SAFETY: event is provided by Quartz for this callback invocation.
    let keycode = unsafe { CGEventGetIntegerValueField(event, KEYBOARD_EVENT_KEYCODE) };
    // SAFETY: event is provided by Quartz for this callback invocation.
    let flags = unsafe { CGEventGetFlags(event) };

    APP_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        let bindings = *KEY_BINDINGS
            .get()
            .expect("key bindings must be initialized");
        maybe_reload_grid_overlay_config(&mut state);
        if state.mouse_mode {
            update_hud_overlay(&mut state);
        }
        let is_key_down = matches!(event_type, CGEventType::KeyDown);
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
            return ptr::null_mut();
        }

        if !state.mouse_mode {
            return event;
        }

        if is_first_keydown && keycode == bindings.grid_key {
            // SAFETY: event is provided by Quartz for this callback invocation.
            let cursor_point = unsafe { CGEventGetLocation(event) };
            let display = display_for_point(cursor_point);
            start_grid_on_display(&mut state, display, cursor_point);
            record_hud_action(&mut state, "Grid opened");
            update_hud_overlay(&mut state);
            return ptr::null_mut();
        }

        if state.grid.is_active() {
            if is_key_down && let Some(display_index) = display_index_for_keycode(keycode) {
                let displays = active_displays_sorted();
                if let Some(display) = displays.get(display_index) {
                    // SAFETY: event is provided by Quartz for this callback invocation.
                    let cursor_point = unsafe { CGEventGetLocation(event) };
                    start_grid_on_display(&mut state, *display, cursor_point);
                }
                return ptr::null_mut();
            }

            if is_first_keydown {
                if keycode == bindings.confirm_key {
                    if let Some(final_bounds) = state.grid.confirm() {
                        let (target_x, target_y) = final_bounds.center();
                        state.enigo.mouse_move_to(target_x, target_y);
                        record_hud_action(&mut state, "Grid confirmed");
                    }
                    state.overlay.hide_grid();
                    update_hud_overlay(&mut state);
                    return ptr::null_mut();
                }

                if keycode == KEYCODE_ESCAPE {
                    state.grid.cancel();
                    state.overlay.hide_grid();
                    record_hud_action(&mut state, "Grid cancelled");
                    update_hud_overlay(&mut state);
                    return ptr::null_mut();
                }

                if let Some((row, col)) = grid_cell_for_keycode(keycode, &bindings) {
                    state.grid.zoom_into_cell(row, col);
                    if let Some((bounds, depth)) = state.grid.render_state() {
                        state.overlay.show_or_update(bounds, depth);
                    }
                    record_hud_action(&mut state, "Grid zoom");
                } else if is_first_keydown {
                    maybe_record_unknown_key_hint(&mut state, keycode, bindings);
                }
            }
            update_hud_overlay(&mut state);
            return ptr::null_mut();
        }

        if is_key_down {
            let (fast_active, slow_active) = modifier_states_from_event_flags(flags, bindings);
            let move_step = movement_step_from_modifiers(fast_active, slow_active);
            let scroll_step = scroll_step_from_modifiers(fast_active, slow_active);
            let mut recognized_action = false;
            match keycode {
                key if key == bindings.movement_left => {
                    state.enigo.mouse_move_relative(-move_step, 0);
                    recognized_action = true;
                    record_hud_action(&mut state, "Move left");
                }
                key if key == bindings.movement_down => {
                    state.enigo.mouse_move_relative(0, move_step);
                    recognized_action = true;
                    record_hud_action(&mut state, "Move down");
                }
                key if key == bindings.movement_up => {
                    state.enigo.mouse_move_relative(0, -move_step);
                    recognized_action = true;
                    record_hud_action(&mut state, "Move up");
                }
                key if key == bindings.movement_right => {
                    state.enigo.mouse_move_relative(move_step, 0);
                    recognized_action = true;
                    record_hud_action(&mut state, "Move right");
                }
                key if key == bindings.scroll_up => {
                    state.enigo.mouse_scroll_y(-scroll_step);
                    recognized_action = true;
                    record_hud_action(&mut state, "Scroll up");
                }
                key if key == bindings.scroll_down => {
                    state.enigo.mouse_scroll_y(scroll_step);
                    recognized_action = true;
                    record_hud_action(&mut state, "Scroll down");
                }
                key if key == bindings.scroll_left => {
                    state.enigo.mouse_scroll_x(-scroll_step);
                    recognized_action = true;
                    record_hud_action(&mut state, "Scroll left");
                }
                key if key == bindings.scroll_right => {
                    state.enigo.mouse_scroll_x(scroll_step);
                    recognized_action = true;
                    record_hud_action(&mut state, "Scroll right");
                }
                key if key == bindings.left_click => {
                    if is_first_keydown {
                        state.enigo.mouse_click(MouseButton::Left);
                        recognized_action = true;
                        record_hud_action(&mut state, "Left click");
                    }
                }
                key if key == bindings.right_click => {
                    if is_first_keydown {
                        state.enigo.mouse_click(MouseButton::Right);
                        recognized_action = true;
                        record_hud_action(&mut state, "Right click");
                    }
                }
                key if key == bindings.drag_toggle => {
                    if is_first_keydown {
                        if state.drag_active {
                            state.enigo.mouse_up(MouseButton::Left);
                            record_hud_action(&mut state, "Drag released");
                        } else {
                            state.enigo.mouse_down(MouseButton::Left);
                            record_hud_action(&mut state, "Drag engaged");
                        }
                        recognized_action = true;
                        state.drag_active = !state.drag_active;
                    }
                }
                _ => {}
            }
            if is_first_keydown && !recognized_action {
                maybe_record_unknown_key_hint(&mut state, keycode, bindings);
            }
        }

        update_hud_overlay(&mut state);

        // Suppress all keydown/keyup events while mouse mode is active.
        ptr::null_mut()
    })
}

fn maybe_reload_grid_overlay_config(state: &mut AppState) {
    let current_modified = current_config_modified_time();
    if !should_reload_config(
        state.last_config_poll.elapsed(),
        current_modified,
        state.config_last_modified,
    ) {
        return;
    }

    state.last_config_poll = Instant::now();
    state.config_last_modified = current_modified;

    match config::load_config_for_reload() {
        Ok(config) => {
            let settings = config.grid_overlay_settings();
            let hud_settings = config.hud_settings();
            let mut reloaded_any = false;
            if settings != state.grid_overlay_settings {
                state.grid_overlay_settings = settings.clone();
                state.overlay.apply_settings(settings);
                reloaded_any = true;
            }
            if hud_settings != state.hud_settings {
                state.hud_settings = hud_settings;
                state.overlay.apply_hud_settings(hud_settings);
                reloaded_any = true;
            }
            if reloaded_any {
                eprintln!(
                    "Reloaded overlay settings from {}",
                    config::config_path().display()
                );
            }
            state.last_reload_error = None;
        }
        Err(error) => {
            if state.last_reload_error.as_deref() != Some(error.as_str()) {
                eprintln!("{}", error);
            }
            state.last_reload_error = Some(error);
        }
    }
}

fn hud_mode_label(state: &AppState) -> &'static str {
    if state.grid.is_active() {
        "Grid mode"
    } else if state.drag_active {
        "Mouse mode (drag)"
    } else {
        "Mouse mode"
    }
}

fn current_hud_action(state: &AppState) -> Option<&str> {
    let until = state.last_action_until?;
    if Instant::now() > until {
        return None;
    }
    state.last_action_message.as_deref()
}

fn record_hud_action(state: &mut AppState, action: &str) {
    state.last_action_message = Some(action.to_string());
    state.last_action_until = Some(Instant::now() + HUD_ACTION_DURATION);
}

fn update_hud_overlay(state: &mut AppState) {
    let mode = hud_mode_label(state).to_string();
    let action = current_hud_action(state).map(str::to_string);
    state.overlay.show_hud(&mode, action.as_deref());
}

fn maybe_record_unknown_key_hint(state: &mut AppState, keycode: i64, bindings: KeyBindings) {
    if !state.hud_settings.show_unknown_key_hint {
        return;
    }
    if keycode == bindings.toggle_key {
        return;
    }
    if let Some(last) = state.last_unknown_hint_at
        && last.elapsed() < HUD_UNKNOWN_KEY_HINT_COOLDOWN
    {
        return;
    }
    state.last_unknown_hint_at = Some(Instant::now());
    record_hud_action(state, "Ignored key");
    play_unknown_key_sound();
}

fn play_unknown_key_sound() {
    // SAFETY: NSBeep is an AppKit call with no arguments and no retained objects.
    unsafe {
        NSBeep();
    }
}

fn start_grid_on_display(
    state: &mut AppState,
    display: core_graphics::display::CGDisplay,
    cursor_point: core_graphics::geometry::CGPoint,
) {
    let display_bounds = GridBounds::from_display(display);
    state.grid.start(display_bounds);
    state.overlay.calibrate_from_cursor(cursor_point);
    if let Some((bounds, depth)) = state.grid.render_state() {
        state.overlay.show_or_update(bounds, depth);
    }
}

fn current_config_modified_time() -> Option<SystemTime> {
    fs::metadata(config::config_path())
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

fn should_reload_config(
    elapsed_since_last_poll: Duration,
    current_modified: Option<SystemTime>,
    previous_modified: Option<SystemTime>,
) -> bool {
    elapsed_since_last_poll >= CONFIG_POLL_INTERVAL && current_modified != previous_modified
}

#[cfg(test)]
mod tests {
    use super::should_reload_config;
    use std::time::{Duration, SystemTime};

    #[test]
    fn skips_reload_before_poll_interval() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2);
        let previous = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
        assert!(!should_reload_config(
            Duration::from_millis(499),
            Some(now),
            Some(previous)
        ));
    }

    #[test]
    fn skips_reload_when_timestamp_has_not_changed() {
        let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(2);
        assert!(!should_reload_config(
            Duration::from_millis(500),
            Some(timestamp),
            Some(timestamp),
        ));
    }

    #[test]
    fn reloads_when_poll_interval_elapsed_and_timestamp_changed() {
        let current = SystemTime::UNIX_EPOCH + Duration::from_secs(2);
        let previous = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
        assert!(should_reload_config(
            Duration::from_millis(500),
            Some(current),
            Some(previous),
        ));
    }
}
