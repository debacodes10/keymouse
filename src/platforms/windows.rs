use std::cmp::Ordering;
use std::mem::size_of;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{BOOL, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetCursorPos, GetMessageW, HC_ACTION, HHOOK,
    KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

pub const MAX_DISPLAYS: usize = 16;
pub const KEYBOARD_EVENT_KEYCODE: u32 = 0;

pub const KEY_EVENT_DOWN_TYPES: [u32; 2] = [WM_KEYDOWN, WM_SYSKEYDOWN];
pub const KEY_EVENT_UP_TYPES: [u32; 2] = [WM_KEYUP, WM_SYSKEYUP];

pub type KeyboardHookCallback = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;

#[derive(Clone, Copy, Debug)]
pub struct Display {
    pub monitor: HMONITOR,
    pub bounds: RECT,
}

impl Display {
    pub fn contains(&self, point: POINT) -> bool {
        point.x >= self.bounds.left
            && point.x < self.bounds.right
            && point.y >= self.bounds.top
            && point.y < self.bounds.bottom
    }
}

pub fn install_keyboard_hook(callback: KeyboardHookCallback) -> Result<HHOOK, String> {
    // SAFETY: WH_KEYBOARD_LL callback follows required ABI and lifetime.
    let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(callback), 0, 0) };
    if hook == 0 {
        return Err("Failed to install low-level keyboard hook.".to_string());
    }
    Ok(hook)
}

pub fn uninstall_keyboard_hook(hook: HHOOK) {
    if hook == 0 {
        return;
    }
    // SAFETY: hook was returned by SetWindowsHookExW.
    unsafe {
        UnhookWindowsHookEx(hook);
    }
}

pub fn call_next_keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: forwarding event chain per Windows hook contract.
    unsafe { CallNextHookEx(0, code, wparam, lparam) }
}

pub fn keyboard_message_keycode(lparam: LPARAM) -> Option<u32> {
    let event_ptr = lparam as *const KBDLLHOOKSTRUCT;
    if event_ptr.is_null() {
        return None;
    }
    // SAFETY: low-level keyboard hook provides a valid KBDLLHOOKSTRUCT for HC_ACTION.
    Some(unsafe { (*event_ptr).vkCode })
}

pub fn is_keyboard_action(code: i32) -> bool {
    code == HC_ACTION
}

pub fn run_message_loop() -> Result<(), String> {
    // SAFETY: MSG is plain old data and zeroed initialization is valid.
    let mut message: MSG = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: message pointer is valid for writes.
        let status = unsafe { GetMessageW(&mut message, 0, 0, 0) };
        match status.cmp(&0) {
            Ordering::Greater => {
                // SAFETY: valid message from GetMessageW.
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            Ordering::Equal => return Ok(()),
            Ordering::Less => return Err("Windows message loop failed.".to_string()),
        }
    }
}

pub fn cursor_position() -> Option<POINT> {
    let mut point = POINT { x: 0, y: 0 };
    // SAFETY: point is a valid out-parameter.
    let ok = unsafe { GetCursorPos(&mut point) };
    if ok == 0 { None } else { Some(point) }
}

pub fn display_for_point(point: POINT) -> Option<Display> {
    let displays = active_displays_sorted();
    displays.into_iter().find(|display| display.contains(point))
}

pub fn active_displays_sorted() -> Vec<Display> {
    let mut monitors = Vec::<Display>::with_capacity(MAX_DISPLAYS);
    // SAFETY: callback writes only through LPARAM-provided pointer to monitors vector.
    unsafe {
        EnumDisplayMonitors(
            0 as HDC,
            null_mut(),
            Some(enum_monitor_callback),
            (&mut monitors as *mut Vec<Display>) as isize,
        );
    }

    monitors.sort_by(|a, b| {
        let x_order = a.bounds.left.cmp(&b.bounds.left);
        if x_order != Ordering::Equal {
            return x_order;
        }
        let y_order = a.bounds.top.cmp(&b.bounds.top);
        if y_order != Ordering::Equal {
            return y_order;
        }
        (a.monitor as usize).cmp(&(b.monitor as usize))
    });

    monitors
}

unsafe extern "system" fn enum_monitor_callback(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    if lparam == 0 {
        return 0;
    }

    let displays = &mut *(lparam as *mut Vec<Display>);
    let mut info = MONITORINFOEXW {
        monitorInfo: windows_sys::Win32::Graphics::Gdi::MONITORINFO {
            cbSize: size_of::<MONITORINFOEXW>() as u32,
            rcMonitor: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            rcWork: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            dwFlags: 0,
        },
        szDevice: [0; 32],
    };

    if GetMonitorInfoW(
        monitor,
        &mut info as *mut MONITORINFOEXW as *mut windows_sys::Win32::Graphics::Gdi::MONITORINFO,
    ) == 0
    {
        return 1;
    }

    displays.push(Display {
        monitor,
        bounds: info.monitorInfo.rcMonitor,
    });
    1
}
