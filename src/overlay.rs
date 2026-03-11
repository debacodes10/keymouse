use crate::config::GridOverlaySettings;
use crate::grid::bounds::GridBounds;
use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyProhibited, NSBackingStoreBuffered,
    NSWindowStyleMask,
};
use cocoa::base::{NO, YES, id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use core_graphics::display::{CGDirectDisplayID, CGDisplay};
use objc::{class, msg_send, sel, sel_impl};

const ALIGN_LEFT: i64 = 0;
const ALIGN_CENTER: i64 = 2;
const MAX_DISPLAYS: usize = 16;

unsafe extern "C" {
    fn CGGetActiveDisplayList(
        max_displays: u32,
        active_displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> i32;
}

pub struct Overlay {
    window: id,
    depth_label: id,
    cell_views: [id; 9],
    cell_labels: [id; 9],
    settings: GridOverlaySettings,
}

impl Overlay {
    pub fn new() -> Self {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let app = NSApp();
            if app == nil {
                let app = NSApplication::sharedApplication(nil);
                let _: () =
                    msg_send![app, setActivationPolicy: NSApplicationActivationPolicyProhibited];
            }
        }

        Self {
            window: nil,
            depth_label: nil,
            cell_views: [nil; 9],
            cell_labels: [nil; 9],
            settings: default_overlay_settings(),
        }
    }

    pub fn calibrate_from_cursor(&mut self, _cursor_quartz: core_graphics::geometry::CGPoint) {
        // No-op: keep API stable for call sites. Overlay conversion is
        // deterministic via desktop-bounds Y flip in appkit_frame_y().
    }

    pub fn show_or_update(&mut self, bounds: GridBounds, depth: usize) {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            self.ensure_window();
            self.update_frame(bounds);
            self.layout_cells();
            self.update_depth(depth);
            let _: () = msg_send![self.window, orderFrontRegardless];
        }
    }

    pub fn hide(&mut self) {
        if self.window == nil {
            return;
        }

        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let _: () = msg_send![self.window, orderOut: nil];
        }
    }

    fn ensure_window(&mut self) {
        if self.window != nil {
            return;
        }

        unsafe {
            let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(100.0, 100.0));
            let window: id = msg_send![class!(NSWindow), alloc];
            let window: id = msg_send![
                window,
                initWithContentRect: frame
                styleMask: NSWindowStyleMask::NSBorderlessWindowMask
                backing: NSBackingStoreBuffered
                defer: NO
            ];
            let clear_color: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![window, setOpaque: NO];
            let _: () = msg_send![window, setBackgroundColor: clear_color];
            let _: () = msg_send![window, setHasShadow: NO];
            let _: () = msg_send![window, setIgnoresMouseEvents: YES];
            let _: () = msg_send![window, setLevel: 5000_i64];
            let _: () = msg_send![window, setReleasedWhenClosed: NO];

            let content: id = msg_send![window, contentView];
            let depth = Self::build_depth_label(content);
            let (cell_views, cell_labels) = Self::build_cells(content);

            self.window = window;
            self.depth_label = depth;
            self.cell_views = cell_views;
            self.cell_labels = cell_labels;
            self.apply_visuals();
        }
    }

    pub fn apply_settings(&mut self, settings: GridOverlaySettings) {
        self.settings = settings;
        if self.window == nil {
            return;
        }

        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            self.apply_visuals();
            self.layout_cells();
        }
    }

    unsafe fn build_depth_label(content: id) -> id {
        let label_frame = NSRect::new(NSPoint::new(10.0, 10.0), NSSize::new(140.0, 26.0));
        let label: id = msg_send![class!(NSTextField), alloc];
        let label: id = msg_send![label, initWithFrame: label_frame];
        let font: id = msg_send![class!(NSFont), boldSystemFontOfSize: 16.0_f64];
        let text = unsafe { NSString::alloc(nil).init_str("Depth: 0") };

        let _: () = msg_send![label, setBezeled: NO];
        let _: () = msg_send![label, setDrawsBackground: NO];
        let _: () = msg_send![label, setEditable: NO];
        let _: () = msg_send![label, setSelectable: NO];
        let _: () = msg_send![label, setAlignment: ALIGN_LEFT];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setStringValue: text];

        let _: () = msg_send![content, addSubview: label];
        label
    }

    unsafe fn build_cells(content: id) -> ([id; 9], [id; 9]) {
        let mut cell_views = [nil; 9];
        let mut cell_labels = [nil; 9];

        for index in 0..9 {
            let view_frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(10.0, 10.0));
            let view: id = msg_send![class!(NSView), alloc];
            let view: id = msg_send![view, initWithFrame: view_frame];
            let _: () = msg_send![view, setWantsLayer: YES];
            let _: () = msg_send![content, addSubview: view];

            let label: id = msg_send![class!(NSTextField), alloc];
            let label: id = msg_send![label, initWithFrame: view_frame];
            let font: id = msg_send![class!(NSFont), boldSystemFontOfSize: 36.0_f64];

            let _: () = msg_send![label, setBezeled: NO];
            let _: () = msg_send![label, setDrawsBackground: NO];
            let _: () = msg_send![label, setEditable: NO];
            let _: () = msg_send![label, setSelectable: NO];
            let _: () = msg_send![label, setAlignment: ALIGN_CENTER];
            let _: () = msg_send![label, setFont: font];
            let _: () = msg_send![content, addSubview: label];

            cell_views[index] = view;
            cell_labels[index] = label;
        }

        (cell_views, cell_labels)
    }

    fn apply_visuals(&self) {
        unsafe {
            let palette = palette_for_theme(&self.settings.theme, self.settings.accent_color);
            for index in 0..9 {
                let layer: id = msg_send![self.cell_views[index], layer];

                let bg = ns_color_from_rgba(
                    palette.cell_bg.0,
                    palette.cell_bg.1,
                    palette.cell_bg.2,
                    palette.cell_bg.3 * self.settings.opacity,
                );
                let border = ns_color_from_rgba(
                    palette.cell_border.0,
                    palette.cell_border.1,
                    palette.cell_border.2,
                    palette.cell_border.3 * self.settings.opacity,
                );
                let text = ns_color_from_rgba(
                    palette.cell_text.0,
                    palette.cell_text.1,
                    palette.cell_text.2,
                    palette.cell_text.3 * self.settings.opacity,
                );
                let bg_color: id = msg_send![bg, CGColor];
                let border_color: id = msg_send![border, CGColor];
                let _: () = msg_send![layer, setBackgroundColor: bg_color];
                let _: () = msg_send![layer, setBorderColor: border_color];
                let _: () = msg_send![layer, setBorderWidth: 2.0_f64];
                let _: () = msg_send![layer, setCornerRadius: 12.0_f64];
                let _: () = msg_send![layer, setShadowOpacity: 0.28_f64 * self.settings.opacity];
                let _: () = msg_send![layer, setShadowRadius: 8.0_f64];
                let _: () = msg_send![layer, setShadowOffset: NSSize::new(0.0, -1.0)];
                let _: () = msg_send![layer, setMasksToBounds: NO];

                let label_text = NSString::alloc(nil).init_str(&self.settings.labels[index]);
                let _: () = msg_send![self.cell_labels[index], setTextColor: text];
                let _: () = msg_send![self.cell_labels[index], setStringValue: label_text];
            }

            let depth_color = ns_color_from_rgba(
                palette.depth_text.0,
                palette.depth_text.1,
                palette.depth_text.2,
                palette.depth_text.3 * self.settings.opacity,
            );
            let _: () = msg_send![self.depth_label, setTextColor: depth_color];
        }
    }

    fn update_frame(&self, bounds: GridBounds) {
        unsafe {
            let appkit_y = self.appkit_frame_y(bounds);
            let frame = NSRect::new(
                NSPoint::new(bounds.x, appkit_y),
                NSSize::new(bounds.width, bounds.height),
            );
            let _: () = msg_send![self.window, setFrame: frame display: YES];
        }
    }

    fn appkit_frame_y(&self, bounds: GridBounds) -> f64 {
        if let Some(desktop) = desktop_bounds() {
            let base = desktop.y + desktop.height;
            base - (bounds.y + bounds.height)
        } else {
            bounds.y
        }
    }

    fn layout_cells(&self) {
        unsafe {
            let frame: NSRect = msg_send![self.window, frame];
            let width = frame.size.width;
            let height = frame.size.height;
            let cell_width = width / 3.0;
            let cell_height = height / 3.0;
            let inset = 7.0;

            for row in 0..3 {
                for col in 0..3 {
                    let index = (row * 3 + col) as usize;
                    let x = col as f64 * cell_width;
                    let y = height - ((row as f64 + 1.0) * cell_height);
                    let cell_frame = NSRect::new(
                        NSPoint::new(x + inset, y + inset),
                        NSSize::new(cell_width - (inset * 2.0), cell_height - (inset * 2.0)),
                    );
                    let label_frame = NSRect::new(
                        NSPoint::new(x + inset, y + (cell_height * 0.33)),
                        NSSize::new(cell_width - (inset * 2.0), 44.0),
                    );

                    let _: () = msg_send![self.cell_views[index], setFrame: cell_frame];
                    let _: () = msg_send![self.cell_labels[index], setFrame: label_frame];
                }
            }

            let depth_frame =
                NSRect::new(NSPoint::new(10.0, height - 34.0), NSSize::new(160.0, 24.0));
            let _: () = msg_send![self.depth_label, setFrame: depth_frame];
        }
    }

    fn update_depth(&self, depth: usize) {
        unsafe {
            let text = format!("Depth: {}", depth);
            let ns_text = NSString::alloc(nil).init_str(&text);
            let _: () = msg_send![self.depth_label, setStringValue: ns_text];
        }
    }
}

#[derive(Clone, Copy)]
struct ThemePalette {
    cell_bg: (f64, f64, f64, f64),
    cell_border: (f64, f64, f64, f64),
    cell_text: (f64, f64, f64, f64),
    depth_text: (f64, f64, f64, f64),
}

fn palette_for_theme(theme: &str, accent_color: Option<(f64, f64, f64)>) -> ThemePalette {
    let mut palette = match theme {
        "midnight" => ThemePalette {
            cell_bg: (0.04, 0.07, 0.15, 0.62),
            cell_border: (0.43, 0.60, 0.94, 0.96),
            cell_text: (0.93, 0.96, 1.00, 1.00),
            depth_text: (0.74, 0.85, 1.00, 1.00),
        },
        "ocean" => ThemePalette {
            cell_bg: (0.02, 0.20, 0.26, 0.60),
            cell_border: (0.24, 0.83, 0.91, 0.98),
            cell_text: (0.90, 1.00, 0.99, 1.00),
            depth_text: (0.66, 0.97, 0.94, 1.00),
        },
        "forest" => ThemePalette {
            cell_bg: (0.09, 0.19, 0.11, 0.58),
            cell_border: (0.52, 0.90, 0.55, 0.98),
            cell_text: (0.95, 1.00, 0.95, 1.00),
            depth_text: (0.78, 1.00, 0.80, 1.00),
        },
        _ => ThemePalette {
            cell_bg: (0.13, 0.14, 0.16, 0.35),
            cell_border: (0.92, 0.95, 1.00, 0.72),
            cell_text: (1.00, 1.00, 1.00, 1.00),
            depth_text: (0.94, 0.97, 1.00, 1.00),
        },
    };

    if let Some(accent) = accent_color {
        palette.cell_bg = blend_rgba(palette.cell_bg, (accent.0, accent.1, accent.2, 0.62), 0.18);
        palette.cell_border = (accent.0, accent.1, accent.2, 0.98);
        palette.depth_text = (accent.0, accent.1, accent.2, 1.00);
    }

    palette
}

fn blend_rgba(
    base: (f64, f64, f64, f64),
    overlay: (f64, f64, f64, f64),
    ratio: f64,
) -> (f64, f64, f64, f64) {
    let r = ratio.clamp(0.0, 1.0);
    (
        base.0 + (overlay.0 - base.0) * r,
        base.1 + (overlay.1 - base.1) * r,
        base.2 + (overlay.2 - base.2) * r,
        base.3 + (overlay.3 - base.3) * r,
    )
}

fn ns_color_from_rgba(red: f64, green: f64, blue: f64, alpha: f64) -> id {
    let clamped = alpha.clamp(0.0, 1.0);
    unsafe {
        msg_send![
            class!(NSColor),
            colorWithCalibratedRed: red
            green: green
            blue: blue
            alpha: clamped
        ]
    }
}

fn default_overlay_settings() -> GridOverlaySettings {
    GridOverlaySettings {
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
        theme: "classic".to_string(),
        opacity: 1.0,
        accent_color: None,
    }
}

fn desktop_bounds() -> Option<GridBounds> {
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
        return None;
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for display_id in display_ids.iter().copied().take(display_count as usize) {
        let bounds = CGDisplay::new(display_id).bounds();
        min_x = min_x.min(bounds.origin.x);
        min_y = min_y.min(bounds.origin.y);
        max_x = max_x.max(bounds.origin.x + bounds.size.width);
        max_y = max_y.max(bounds.origin.y + bounds.size.height);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some(GridBounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}

impl Drop for Overlay {
    fn drop(&mut self) {
        if self.window == nil {
            return;
        }

        unsafe {
            let _: () = msg_send![self.window, orderOut: nil];
            let _: () = msg_send![self.window, close];
        }
    }
}
