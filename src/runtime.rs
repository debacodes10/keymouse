use crate::config::{self, KeyBindings};
use crate::grid::bounds::GridBounds;
use crate::grid::recursive::RecursiveGrid;
use crate::input::{KEYCODE_ESCAPE, KEYCODE_F8, grid_cell_for_keycode, movement_step, scroll_step};
use crate::overlay::Overlay;
use crate::platform::{
    CFMachPortCreateRunLoopSource, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun,
    CGEventGetFlags, CGEventGetIntegerValueField, CGEventGetLocation, CGEventTapCreate,
    CGEventTapEnable, KEYBOARD_EVENT_KEYCODE, display_for_point, event_mask,
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
use std::ptr;
use std::sync::OnceLock;
use std::thread_local;

static KEY_BINDINGS: OnceLock<KeyBindings> = OnceLock::new();
static EVENT_TAP: OnceLock<usize> = OnceLock::new();

struct AppState {
    enigo: Enigo,
    mouse_mode: bool,
    drag_active: bool,
    grid: RecursiveGrid,
    overlay: Overlay,
    held_keys: HashSet<i64>,
}

impl AppState {
    fn new() -> Self {
        Self {
            enigo: Enigo::new(),
            mouse_mode: false,
            drag_active: false,
            grid: RecursiveGrid::new(),
            overlay: Overlay::new(),
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

pub fn run() {
    let config = config::load_config();
    let bindings = KeyBindings::from_config(&config);
    let _ = KEY_BINDINGS.set(bindings);

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

        eprintln!("keymouse running. Press F8 to toggle mouse mode.");
        CFRunLoopRun();
    }
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
        let is_key_down = matches!(event_type, CGEventType::KeyDown);
        let is_first_keydown = if is_key_down {
            state.held_keys.insert(keycode)
        } else {
            state.held_keys.remove(&keycode);
            false
        };

        if keycode == KEYCODE_F8 && is_first_keydown {
            state.mouse_mode = !state.mouse_mode;
            if !state.mouse_mode {
                state.grid.cancel();
                state.overlay.hide();
                state.release_drag_if_active();
                state.held_keys.clear();
            }
            eprintln!(
                "mouse mode: {}",
                if state.mouse_mode { "on" } else { "off" }
            );
            return ptr::null_mut();
        }

        if !state.mouse_mode {
            return event;
        }

        if is_first_keydown && keycode == bindings.grid_key {
            // SAFETY: event is provided by Quartz for this callback invocation.
            let cursor_point = unsafe { CGEventGetLocation(event) };
            let display = display_for_point(cursor_point);
            let display_bounds = GridBounds::from_display(display);
            state.grid.start(display_bounds);
            state.overlay.calibrate_from_cursor(cursor_point);
            if let Some((bounds, depth)) = state.grid.render_state() {
                state.overlay.show_or_update(bounds, depth);
            }
            return ptr::null_mut();
        }

        if state.grid.is_active() {
            if is_first_keydown {
                if keycode == bindings.confirm_key {
                    if let Some(final_bounds) = state.grid.confirm() {
                        let (target_x, target_y) = final_bounds.center();
                        state.enigo.mouse_move_to(target_x, target_y);
                    }
                    state.overlay.hide();
                    return ptr::null_mut();
                }

                if keycode == KEYCODE_ESCAPE {
                    state.grid.cancel();
                    state.overlay.hide();
                    return ptr::null_mut();
                }

                if let Some((row, col)) = grid_cell_for_keycode(keycode) {
                    state.grid.zoom_into_cell(row, col);
                    if let Some((bounds, depth)) = state.grid.render_state() {
                        state.overlay.show_or_update(bounds, depth);
                    }
                }
            }
            return ptr::null_mut();
        }

        if is_key_down {
            let move_step = movement_step(flags, bindings);
            let scroll_step = scroll_step(flags, bindings);
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

        // Suppress all keydown/keyup events while mouse mode is active.
        ptr::null_mut()
    })
}
