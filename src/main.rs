mod grid;

use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoopSourceRef};
use core_graphics::display::{CGDirectDisplayID, CGDisplay};
use core_graphics::event::{
    CGEventField, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy,
    CGEventType,
};
use core_graphics::geometry::CGPoint;
use core_graphics::sys::CGEventRef;
use enigo::{Enigo, MouseButton, MouseControllable};
use grid::bounds::GridBounds;
use grid::recursive::RecursiveGrid;

use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr;
use std::thread_local;

const KEYCODE_H: i64 = 4;
const KEYCODE_J: i64 = 38;
const KEYCODE_K: i64 = 40;
const KEYCODE_L: i64 = 37;
const KEYCODE_F: i64 = 3;
const KEYCODE_D: i64 = 2;
const KEYCODE_G: i64 = 5;
const KEYCODE_F8: i64 = 100;
const KEYCODE_SEMICOLON: i64 = 41;
const KEYCODE_ENTER: i64 = 36;
const KEYCODE_ESCAPE: i64 = 53;
const KEYCODE_Q: i64 = 12;
const KEYCODE_W: i64 = 13;
const KEYCODE_E: i64 = 14;
const KEYCODE_A: i64 = 0;
const KEYCODE_S: i64 = 1;
const KEYCODE_Z: i64 = 6;
const KEYCODE_X: i64 = 7;
const KEYCODE_C: i64 = 8;

const KEYBOARD_EVENT_KEYCODE: CGEventField = 9;
const NORMAL_SPEED: i32 = 20;
const FAST_SPEED: i32 = 120;
const SLOW_SPEED: i32 = 5;
const MAX_DISPLAYS: usize = 16;

const EVENT_FLAG_MASK_SHIFT: u64 = 1 << 17;
const EVENT_FLAG_MASK_OPTION: u64 = 1 << 19;

struct AppState {
    enigo: Enigo,
    mouse_mode: bool,
    grid: RecursiveGrid,
}

impl AppState {
    fn new() -> Self {
        Self {
            enigo: Enigo::new(),
            mouse_mode: false,
            grid: RecursiveGrid::new(),
        }
    }
}

thread_local! {
    static APP_STATE: RefCell<AppState> = RefCell::new(AppState::new());
}

unsafe extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGGetActiveDisplayList(
        max_displays: u32,
        active_displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> i32;

    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;

    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: CFRunLoopSourceRef, mode: *const c_void);
    fn CFRunLoopRun();
}

type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

fn event_mask(types: &[CGEventType]) -> u64 {
    types
        .iter()
        .fold(0_u64, |mask, t| mask | (1_u64 << (*t as u64)))
}

fn grid_cell_for_keycode(keycode: i64) -> Option<(i32, i32)> {
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

fn movement_step(flags: u64) -> i32 {
    let shift_pressed = (flags & EVENT_FLAG_MASK_SHIFT) != 0;
    let option_pressed = (flags & EVENT_FLAG_MASK_OPTION) != 0;

    if shift_pressed {
        FAST_SPEED
    } else if option_pressed {
        SLOW_SPEED
    } else {
        NORMAL_SPEED
    }
}

fn display_for_point(point: CGPoint) -> CGDisplay {
    let mut display_ids = [0_u32; MAX_DISPLAYS];
    let mut display_count = 0_u32;
    let mut display = CGDisplay::main();

    // Use the display currently containing the cursor so grid navigation works
    // correctly on multi-monitor setups.
    let result = unsafe {
        CGGetActiveDisplayList(
            MAX_DISPLAYS as u32,
            display_ids.as_mut_ptr(),
            &mut display_count,
        )
    };
    if result == 0 {
        for display_id in display_ids.iter().copied().take(display_count as usize) {
            let candidate = CGDisplay::new(display_id);
            if candidate.bounds().contains(&point) {
                display = candidate;
                break;
            }
        }
    }

    display
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

        if keycode == KEYCODE_F8 && matches!(event_type, CGEventType::KeyDown) {
            state.mouse_mode = !state.mouse_mode;
            if !state.mouse_mode {
                state.grid.cancel();
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

        if matches!(event_type, CGEventType::KeyDown) && keycode == KEYCODE_SEMICOLON {
            // SAFETY: event is provided by Quartz for this callback invocation.
            let cursor_point = unsafe { CGEventGetLocation(event) };
            let display = display_for_point(cursor_point);
            state.grid.start(GridBounds::from_display(display));
            return ptr::null_mut();
        }

        if state.grid.is_active() {
            if matches!(event_type, CGEventType::KeyDown) {
                if keycode == KEYCODE_ENTER {
                    if let Some(final_bounds) = state.grid.confirm() {
                        let (target_x, target_y) = final_bounds.center();
                        state.enigo.mouse_move_to(target_x, target_y);
                    }
                    return ptr::null_mut();
                }

                if keycode == KEYCODE_ESCAPE {
                    state.grid.cancel();
                    return ptr::null_mut();
                }

                if let Some((row, col)) = grid_cell_for_keycode(keycode) {
                    state.grid.zoom_into_cell(row, col);
                }
            }
            return ptr::null_mut();
        }

        if matches!(event_type, CGEventType::KeyDown) {
            let step = movement_step(flags);
            match keycode {
                KEYCODE_H => {
                    state.enigo.mouse_move_relative(-step, 0);
                }
                KEYCODE_J => {
                    state.enigo.mouse_move_relative(0, step);
                }
                KEYCODE_K => {
                    state.enigo.mouse_move_relative(0, -step);
                }
                KEYCODE_L => {
                    state.enigo.mouse_move_relative(step, 0);
                }
                KEYCODE_F => {
                    state.enigo.mouse_click(MouseButton::Left);
                }
                KEYCODE_D => {
                    state.enigo.mouse_click(MouseButton::Right);
                }
                _ => {}
            }
        }

        // Suppress all keydown/keyup events while mouse mode is active.
        ptr::null_mut()
    })
}

fn main() {
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
