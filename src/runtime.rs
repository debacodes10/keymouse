use crate::config::{self, KeyBindings};
use crate::grid::bounds::GridBounds;
use crate::grid::recursive::RecursiveGrid;
use crate::input::{
    KEYCODE_ESCAPE, KEYCODE_F8, grid_cell_for_keycode, movement_step, scroll_step,
};
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
use std::ffi::c_void;
use std::ptr;
use std::sync::OnceLock;
use std::thread_local;

static KEY_BINDINGS: OnceLock<KeyBindings> = OnceLock::new();

struct AppState {
    enigo: Enigo,
    mouse_mode: bool,
    grid: RecursiveGrid,
    overlay: Overlay,
}

impl AppState {
    fn new() -> Self {
        Self {
            enigo: Enigo::new(),
            mouse_mode: false,
            grid: RecursiveGrid::new(),
            overlay: Overlay::new(),
        }
    }
}

thread_local! {
    static APP_STATE: RefCell<AppState> = RefCell::new(AppState::new());
}

pub fn run() {
    let config = config::load_config();
    let _ = KEY_BINDINGS.set(KeyBindings::from_config(&config));

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

        if keycode == KEYCODE_F8 && matches!(event_type, CGEventType::KeyDown) {
            state.mouse_mode = !state.mouse_mode;
            if !state.mouse_mode {
                state.grid.cancel();
                state.overlay.hide();
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

        if matches!(event_type, CGEventType::KeyDown) && keycode == bindings.grid_key {
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
            if matches!(event_type, CGEventType::KeyDown) {
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

        if matches!(event_type, CGEventType::KeyDown) {
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
                    state.enigo.mouse_click(MouseButton::Left);
                }
                key if key == bindings.right_click => {
                    state.enigo.mouse_click(MouseButton::Right);
                }
                _ => {}
            }
        }

        // Suppress all keydown/keyup events while mouse mode is active.
        ptr::null_mut()
    })
}
