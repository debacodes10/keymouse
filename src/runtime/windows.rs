use crate::config::{self, KeyBindings};
use crate::grid::bounds::GridBounds;
use crate::grid::recursive::RecursiveGrid;
use crate::input::{
    KEYCODE_ESCAPE, display_index_for_keycode, grid_cell_for_keycode,
    modifier_states_from_held_keys, movement_step_from_modifiers, scroll_step_from_modifiers,
};
use crate::platforms::{
    KEY_EVENT_DOWN_TYPES, KEY_EVENT_UP_TYPES, call_next_keyboard_hook, cursor_position,
    display_for_point, install_keyboard_hook, is_keyboard_action, keyboard_message_keycode,
    run_message_loop, uninstall_keyboard_hook,
};
use enigo::{Enigo, MouseButton, MouseControllable};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::OnceLock;
use std::thread_local;
use std::time::{Duration, Instant, SystemTime};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, DeleteObject, EndPaint, LineTo, MoveToEx, PAINTSTRUCT, PS_SOLID,
    SelectObject, SetBkMode, SetTextColor, TRANSPARENT, TextOutW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, LWA_ALPHA, PostQuitMessage, RegisterClassW,
    SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetLayeredWindowAttributes,
    SetWindowPos, ShowWindow, WM_DESTROY, WM_ERASEBKGND, WM_PAINT, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

const CONFIG_POLL_INTERVAL: Duration = Duration::from_millis(500);

static KEY_BINDINGS: OnceLock<KeyBindings> = OnceLock::new();
static KEYBOARD_HOOK: OnceLock<usize> = OnceLock::new();
static MOUSE_MODE_LISTENER: OnceLock<fn(bool)> = OnceLock::new();
static OVERLAY_CLASS_NAME: OnceLock<Vec<u16>> = OnceLock::new();
static OVERLAY_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

thread_local! {
    static OVERLAY_RENDER: RefCell<OverlayRenderState> = RefCell::new(OverlayRenderState::default());
}

struct AppState {
    enigo: Enigo,
    mouse_mode: bool,
    drag_active: bool,
    grid: RecursiveGrid,
    overlay: Overlay,
    held_keys: HashSet<i64>,
    grid_overlay_settings: config::GridOverlaySettings,
    config_last_modified: Option<SystemTime>,
    last_config_poll: Instant,
    last_reload_error: Option<String>,
}

impl AppState {
    fn new() -> Self {
        let mut overlay = Overlay::new();
        let grid_overlay_settings = config::Config::default().grid_overlay_settings();
        overlay.apply_settings(grid_overlay_settings.clone());

        Self {
            enigo: Enigo::new(),
            mouse_mode: false,
            drag_active: false,
            grid: RecursiveGrid::new(),
            overlay,
            held_keys: HashSet::new(),
            grid_overlay_settings,
            config_last_modified: None,
            last_config_poll: Instant::now(),
            last_reload_error: None,
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
    let initial_modified = current_config_modified_time();
    let bindings = KeyBindings::from_config(&config);
    let _ = KEY_BINDINGS.set(bindings);
    eprintln!("Toggle key binding: {}", toggle_key_name);

    APP_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.grid_overlay_settings = initial_overlay_settings.clone();
        state.overlay.apply_settings(initial_overlay_settings);
        state.config_last_modified = initial_modified;
    });

    match install_keyboard_hook(keyboard_callback) {
        Ok(hook) => {
            let _ = KEYBOARD_HOOK.set(hook as usize);
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

#[allow(dead_code)]
pub fn mouse_mode_enabled() -> bool {
    APP_STATE.with(|cell| cell.borrow().mouse_mode)
}

#[allow(dead_code)]
pub fn set_mouse_mode_listener(listener: fn(bool)) {
    let _ = MOUSE_MODE_LISTENER.set(listener);
}

#[allow(dead_code)]
pub fn set_mouse_mode(enabled: bool) {
    APP_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        apply_mouse_mode_state(&mut state, enabled);
    });
}

#[allow(dead_code)]
pub fn shutdown() {
    set_mouse_mode(false);
    if let Some(hook) = KEYBOARD_HOOK.get() {
        uninstall_keyboard_hook(*hook as *mut c_void);
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
        maybe_reload_grid_overlay_config(&mut state);

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
                    start_grid_on_display(&mut state, display.bounds);
                }
            }
            return 1;
        }

        if state.grid.is_active() {
            if is_key_down
                && let Some(display_index) = display_index_for_keycode(keycode)
                && let Some(display) = crate::platforms::active_displays_sorted().get(display_index)
            {
                start_grid_on_display(&mut state, display.bounds);
                return 1;
            }

            if is_first_keydown {
                if keycode == bindings.confirm_key {
                    if let Some(final_bounds) = state.grid.confirm() {
                        let (target_x, target_y) = final_bounds.center();
                        state.enigo.mouse_move_to(target_x, target_y);
                    }
                    state.overlay.hide();
                    return 1;
                }

                if keycode == KEYCODE_ESCAPE {
                    state.grid.cancel();
                    state.overlay.hide();
                    return 1;
                }

                if let Some((row, col)) = grid_cell_for_keycode(keycode) {
                    state.grid.zoom_into_cell(row, col);
                    if let Some((bounds, depth)) = state.grid.render_state() {
                        state.overlay.show_or_update(bounds, depth);
                    }
                }
            }
            return 1;
        }

        if is_key_down {
            let (fast_active, slow_active) =
                modifier_states_from_held_keys(bindings, &state.held_keys);
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

fn start_grid_on_display(state: &mut AppState, rect: RECT) {
    let bounds = GridBounds::from_rect(
        f64::from(rect.left),
        f64::from(rect.top),
        f64::from(rect.right - rect.left),
        f64::from(rect.bottom - rect.top),
    );
    state.grid.start(bounds);
    if let Some((render_bounds, depth)) = state.grid.render_state() {
        state.overlay.show_or_update(render_bounds, depth);
    }
}

fn maybe_reload_grid_overlay_config(state: &mut AppState) {
    if state.last_config_poll.elapsed() < CONFIG_POLL_INTERVAL {
        return;
    }
    state.last_config_poll = Instant::now();

    let current_modified = current_config_modified_time();
    if current_modified == state.config_last_modified {
        return;
    }
    state.config_last_modified = current_modified;

    match config::load_config_for_reload() {
        Ok(config) => {
            let settings = config.grid_overlay_settings();
            if settings != state.grid_overlay_settings {
                state.grid_overlay_settings = settings.clone();
                state.overlay.apply_settings(settings);
                eprintln!(
                    "Reloaded grid overlay settings from {}",
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

fn current_config_modified_time() -> Option<SystemTime> {
    std::fs::metadata(config::config_path())
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

struct Overlay {
    hwnd: HWND,
    settings: config::GridOverlaySettings,
}

impl Overlay {
    fn new() -> Self {
        Self {
            hwnd: null_mut(),
            settings: config::Config::default().grid_overlay_settings(),
        }
    }

    fn apply_settings(&mut self, settings: config::GridOverlaySettings) {
        self.settings = settings.clone();
        OVERLAY_RENDER.with(|cell| {
            let mut render = cell.borrow_mut();
            render.labels = settings.labels;
        });
        if !self.hwnd.is_null() {
            let alpha = (self.settings.opacity.clamp(0.1, 1.0) * 255.0).round() as u8;
            // SAFETY: hwnd is valid and overlay window uses WS_EX_LAYERED.
            unsafe { SetLayeredWindowAttributes(self.hwnd, 0, alpha, LWA_ALPHA) };
        }
        self.request_repaint();
    }

    fn show_or_update(&mut self, bounds: GridBounds, depth: usize) {
        if self.ensure_window().is_err() {
            return;
        }

        OVERLAY_RENDER.with(|cell| {
            let mut render = cell.borrow_mut();
            render.depth = depth;
            render.visible = true;
        });

        // SAFETY: hwnd is a valid top-level window handle once ensure_window succeeds.
        unsafe {
            SetWindowPos(
                self.hwnd,
                null_mut(),
                bounds.x.round() as i32,
                bounds.y.round() as i32,
                bounds.width.max(1.0).round() as i32,
                bounds.height.max(1.0).round() as i32,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
        self.request_repaint();
    }

    fn hide(&mut self) {
        if self.hwnd.is_null() {
            return;
        }
        OVERLAY_RENDER.with(|cell| {
            cell.borrow_mut().visible = false;
        });
        // SAFETY: hwnd belongs to this process and may be hidden at any time.
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    fn ensure_window(&mut self) -> Result<(), String> {
        if !self.hwnd.is_null() {
            return Ok(());
        }

        register_overlay_class()?;
        let class_name = overlay_class_name();
        let title = to_wide("Keymouse Overlay");

        // SAFETY: creating a top-level popup window with a registered class and null parent/menu.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED
                    | WS_EX_TOOLWINDOW
                    | WS_EX_TOPMOST
                    | WS_EX_TRANSPARENT
                    | WS_EX_NOACTIVATE,
                class_name,
                title.as_ptr(),
                WS_POPUP,
                0,
                0,
                100,
                100,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };

        if hwnd.is_null() {
            return Err("Failed to create Windows overlay window.".to_string());
        }

        self.hwnd = hwnd;
        let alpha = (self.settings.opacity.clamp(0.1, 1.0) * 255.0).round() as u8;
        // SAFETY: hwnd is valid and layered style is enabled.
        unsafe {
            SetLayeredWindowAttributes(self.hwnd, 0, alpha, LWA_ALPHA);
            ShowWindow(self.hwnd, SW_HIDE);
        }
        Ok(())
    }

    fn request_repaint(&self) {
        // No-op for compatibility: SetWindowPos/ShowWindow path triggers repaint.
    }
}

#[derive(Clone)]
struct OverlayRenderState {
    labels: [String; 9],
    depth: usize,
    visible: bool,
}

impl Default for OverlayRenderState {
    fn default() -> Self {
        Self {
            labels: [
                "Q".to_string(),
                "W".to_string(),
                "E".to_string(),
                "A".to_string(),
                "S".to_string(),
                "D".to_string(),
                "Z".to_string(),
                "X".to_string(),
                "C".to_string(),
            ],
            depth: 0,
            visible: false,
        }
    }
}

fn register_overlay_class() -> Result<(), String> {
    if OVERLAY_CLASS_REGISTERED.get().is_some() {
        return Ok(());
    }

    let class_name = overlay_class_name();
    // SAFETY: class structure is fully initialized and callback ABI matches Win32 requirements.
    let atom = unsafe {
        let mut class_def: WNDCLASSW = std::mem::zeroed();
        class_def.lpfnWndProc = Some(overlay_window_proc);
        class_def.lpszClassName = class_name;
        RegisterClassW(&class_def)
    };

    if atom == 0 {
        return Err("Failed to register overlay window class.".to_string());
    }

    let _ = OVERLAY_CLASS_REGISTERED.set(());
    Ok(())
}

fn overlay_class_name() -> *const u16 {
    OVERLAY_CLASS_NAME
        .get_or_init(|| to_wide("KeymouseOverlayWindow"))
        .as_ptr()
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn overlay_window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            paint_overlay(hwnd);
            0
        }
        WM_DESTROY => {
            // SAFETY: posting quit message to this thread is always valid.
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => {
            // SAFETY: forwarding unhandled messages to default window proc.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}

fn paint_overlay(hwnd: HWND) {
    let render = OVERLAY_RENDER.with(|cell| cell.borrow().clone());
    if !render.visible {
        return;
    }

    // SAFETY: painting is scoped to WM_PAINT and uses valid handles returned by BeginPaint.
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        if hdc.is_null() {
            return;
        }

        let mut client: RECT = std::mem::zeroed();
        let got_rect = GetClientRect(hwnd, &mut client);
        if got_rect == 0 {
            EndPaint(hwnd, &ps);
            return;
        }

        let width = (client.right - client.left).max(1);
        let height = (client.bottom - client.top).max(1);
        let cell_w = width / 3;
        let cell_h = height / 3;

        let grid_color: u32 = 0x00CCFF;
        let text_color: u32 = 0xFFFFFF;

        let pen = CreatePen(PS_SOLID, 2, grid_color);
        let old_pen = SelectObject(hdc, pen as *mut c_void);

        MoveToEx(hdc, 0, cell_h, null_mut());
        LineTo(hdc, width, cell_h);
        MoveToEx(hdc, 0, cell_h * 2, null_mut());
        LineTo(hdc, width, cell_h * 2);

        MoveToEx(hdc, cell_w, 0, null_mut());
        LineTo(hdc, cell_w, height);
        MoveToEx(hdc, cell_w * 2, 0, null_mut());
        LineTo(hdc, cell_w * 2, height);

        MoveToEx(hdc, 0, 0, null_mut());
        LineTo(hdc, width, 0);
        LineTo(hdc, width, height);
        LineTo(hdc, 0, height);
        LineTo(hdc, 0, 0);

        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, text_color);

        for row in 0..3 {
            for col in 0..3 {
                let index = (row * 3 + col) as usize;
                let label = format!("{}", render.labels[index]);
                let label_wide = to_wide(&label);
                let x = col * cell_w + (cell_w / 2) - 6;
                let y = row * cell_h + (cell_h / 2) - 8;
                TextOutW(
                    hdc,
                    x,
                    y,
                    label_wide.as_ptr(),
                    (label_wide.len() - 1) as i32,
                );
            }
        }

        let depth = format!("Depth: {}", render.depth);
        let depth_wide = to_wide(&depth);
        TextOutW(
            hdc,
            8,
            8,
            depth_wide.as_ptr(),
            (depth_wide.len() - 1) as i32,
        );

        SelectObject(hdc, old_pen);
        DeleteObject(pen as *mut c_void);
        EndPaint(hwnd, &ps);
    }
}
