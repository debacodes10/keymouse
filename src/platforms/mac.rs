use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::CFRunLoopSourceRef;
use core_graphics::display::{CGDirectDisplayID, CGDisplay};
use core_graphics::event::{
    CGEventField, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy,
    CGEventType,
};
use core_graphics::geometry::CGPoint;
use core_graphics::sys::CGEventRef;
use std::cmp::Ordering;
use std::ffi::c_void;

pub const MAX_DISPLAYS: usize = 16;
pub const KEYBOARD_EVENT_KEYCODE: CGEventField = 9;

pub type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

unsafe extern "C" {
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    pub fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
    pub fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    pub fn CGEventGetFlags(event: CGEventRef) -> u64;
    pub fn CGGetActiveDisplayList(
        max_displays: u32,
        active_displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> i32;

    pub fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;

    pub fn CFRunLoopGetCurrent() -> *mut c_void;
    pub fn CFRunLoopAddSource(rl: *mut c_void, source: CFRunLoopSourceRef, mode: *const c_void);
    pub fn CFRunLoopRun();
}

pub fn event_mask(types: &[CGEventType]) -> u64 {
    types
        .iter()
        .fold(0_u64, |mask, t| mask | (1_u64 << (*t as u64)))
}

pub fn display_for_point(point: CGPoint) -> CGDisplay {
    let mut display = CGDisplay::main();

    // Use the display currently containing the cursor so grid navigation works
    // correctly on multi-monitor setups.
    for candidate in active_displays_sorted() {
        if candidate.bounds().contains(&point) {
            display = candidate;
            break;
        }
    }

    display
}

pub fn active_displays_sorted() -> Vec<CGDisplay> {
    let mut display_ids = [0_u32; MAX_DISPLAYS];
    let mut display_count = 0_u32;
    let result = unsafe {
        CGGetActiveDisplayList(
            MAX_DISPLAYS as u32,
            display_ids.as_mut_ptr(),
            &mut display_count,
        )
    };
    if result != 0 || display_count == 0 {
        return vec![CGDisplay::main()];
    }

    let mut displays = display_ids
        .iter()
        .copied()
        .take(display_count as usize)
        .map(|id| (id, CGDisplay::new(id)))
        .collect::<Vec<_>>();

    // Keep monitor numbers deterministic across runs/layout changes.
    displays.sort_by(|a, b| {
        let a_bounds = a.1.bounds();
        let b_bounds = b.1.bounds();
        let order_x = a_bounds.origin.x.total_cmp(&b_bounds.origin.x);
        if order_x != Ordering::Equal {
            return order_x;
        }
        let order_y = a_bounds.origin.y.total_cmp(&b_bounds.origin.y);
        if order_y != Ordering::Equal {
            return order_y;
        }
        a.0.cmp(&b.0)
    });

    displays.into_iter().map(|(_, display)| display).collect()
}
