use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::CFRunLoopSourceRef;
use core_graphics::display::{CGDirectDisplayID, CGDisplay};
use core_graphics::event::{
    CGEventField, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy,
    CGEventType,
};
use core_graphics::geometry::CGPoint;
use core_graphics::sys::CGEventRef;
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
