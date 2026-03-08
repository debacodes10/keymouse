use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoopSourceRef};
use core_graphics::display::{CGDirectDisplayID, CGDisplay};
use core_graphics::event::{CGEventField, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType};
use core_graphics::geometry::CGPoint;
use core_graphics::sys::CGEventRef;
use enigo::{Enigo, MouseButton, MouseControllable};

use std::ffi::c_void;
use std::ptr;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread_local;

const KEYCODE_H: i64 = 4;
const KEYCODE_J: i64 = 38;
const KEYCODE_K: i64 = 40;
const KEYCODE_L: i64 = 37;
const KEYCODE_F: i64 = 3;
const KEYCODE_D: i64 = 2;
const KEYCODE_F8: i64 = 100;
const KEYCODE_SEMICOLON: i64 = 41;
const KEYCODE_Q: i64 = 12;
const KEYCODE_W: i64 = 13;
const KEYCODE_E: i64 = 14;
const KEYCODE_A: i64 = 0;
const KEYCODE_S: i64 = 1;
const KEYCODE_Z: i64 = 6;
const KEYCODE_X: i64 = 7;
const KEYCODE_C: i64 = 8;

const KEYBOARD_EVENT_KEYCODE: CGEventField = 9;
const MOVE_STEP: i32 = 20;
const MAX_DISPLAYS: usize = 16;

static MOUSE_MODE: AtomicBool = AtomicBool::new(false);
static GRID_MODE: AtomicBool = AtomicBool::new(false);
thread_local! {
    static ENIGO: RefCell<Enigo> = RefCell::new(Enigo::new());
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

fn with_enigo<F>(f: F)
where
    F: FnOnce(&mut Enigo),
{
    ENIGO.with(|cell| {
        let mut enigo = cell.borrow_mut();
        f(&mut enigo);
    });
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
        _ => None,
    }
}

fn display_for_point(point: CGPoint) -> CGDisplay {
    let mut display_ids = [0_u32; MAX_DISPLAYS];
    let mut display_count = 0_u32;
    let mut display = CGDisplay::main();

    // Use the display currently containing the cursor so 3x3 jump works
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

fn jump_to_grid_cell_on_display(display: CGDisplay, row: i32, col: i32) {
    let bounds = display.bounds();

    let origin_x = bounds.origin.x;
    let origin_y = bounds.origin.y;
    let width = bounds.size.width;
    let height = bounds.size.height;

    let cell_width = width / 3.0;
    let cell_height = height / 3.0;

    let target_x = origin_x + (col as f64) * cell_width + (cell_width / 2.0);
    let target_y = origin_y + (row as f64) * cell_height + (cell_height / 2.0);

    with_enigo(|e| e.mouse_move_to(target_x.round() as i32, target_y.round() as i32));
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

    if keycode == KEYCODE_F8 && matches!(event_type, CGEventType::KeyDown) {
        let enabled = !MOUSE_MODE.load(Ordering::SeqCst);
        MOUSE_MODE.store(enabled, Ordering::SeqCst);
        if !enabled {
            GRID_MODE.store(false, Ordering::SeqCst);
        }
        eprintln!("mouse mode: {}", if enabled { "on" } else { "off" });
        return ptr::null_mut();
    }

    if !MOUSE_MODE.load(Ordering::SeqCst) {
        return event;
    }

    if matches!(event_type, CGEventType::KeyDown) && keycode == KEYCODE_SEMICOLON {
        GRID_MODE.store(true, Ordering::SeqCst);
        return ptr::null_mut();
    }

    if GRID_MODE.load(Ordering::SeqCst) {
        if matches!(event_type, CGEventType::KeyDown) {
            if let Some((row, col)) = grid_cell_for_keycode(keycode) {
                let cursor_point = unsafe { CGEventGetLocation(event) };
                let display = display_for_point(cursor_point);
                jump_to_grid_cell_on_display(display, row, col);
                GRID_MODE.store(false, Ordering::SeqCst);
            }
        }
        return ptr::null_mut();
    }

    if matches!(event_type, CGEventType::KeyDown) {
        match keycode {
            KEYCODE_H => {
                with_enigo(|e| e.mouse_move_relative(-MOVE_STEP, 0));
            }
            KEYCODE_J => {
                with_enigo(|e| e.mouse_move_relative(0, MOVE_STEP));
            }
            KEYCODE_K => {
                with_enigo(|e| e.mouse_move_relative(0, -MOVE_STEP));
            }
            KEYCODE_L => {
                with_enigo(|e| e.mouse_move_relative(MOVE_STEP, 0));
            }
            KEYCODE_F => {
                with_enigo(|e| e.mouse_click(MouseButton::Left));
            }
            KEYCODE_D => {
                with_enigo(|e| e.mouse_click(MouseButton::Right));
            }
            _ => {}
        }
    }

    // Suppress all keydown/keyup events while mouse mode is active.
    ptr::null_mut()
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
