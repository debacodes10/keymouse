use crate::grid::bounds::GridBounds;
use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyProhibited, NSBackingStoreBuffered,
    NSWindowStyleMask,
};
use cocoa::base::{NO, YES, id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use core_graphics::display::{CGDirectDisplayID, CGDisplay};
use objc::{class, msg_send, sel, sel_impl};

const LABELS: [&str; 9] = ["Q", "W", "E", "A", "S", "D", "Z", "X", "C"];
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
        }
    }

    unsafe fn build_depth_label(content: id) -> id {
        let label_frame = NSRect::new(NSPoint::new(10.0, 10.0), NSSize::new(140.0, 26.0));
        let label: id = msg_send![class!(NSTextField), alloc];
        let label: id = msg_send![label, initWithFrame: label_frame];
        let white: id = msg_send![class!(NSColor), whiteColor];
        let font: id = msg_send![class!(NSFont), boldSystemFontOfSize: 16.0_f64];
        let text = unsafe { NSString::alloc(nil).init_str("Depth: 0") };

        let _: () = msg_send![label, setBezeled: NO];
        let _: () = msg_send![label, setDrawsBackground: NO];
        let _: () = msg_send![label, setEditable: NO];
        let _: () = msg_send![label, setSelectable: NO];
        let _: () = msg_send![label, setAlignment: ALIGN_LEFT];
        let _: () = msg_send![label, setTextColor: white];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setStringValue: text];

        let _: () = msg_send![content, addSubview: label];
        label
    }

    unsafe fn build_cells(content: id) -> ([id; 9], [id; 9]) {
        let mut cell_views = [nil; 9];
        let mut cell_labels = [nil; 9];

        for (index, text) in LABELS.iter().enumerate() {
            let view_frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(10.0, 10.0));
            let view: id = msg_send![class!(NSView), alloc];
            let view: id = msg_send![view, initWithFrame: view_frame];
            let _: () = msg_send![view, setWantsLayer: YES];
            let layer: id = msg_send![view, layer];

            let bg: id = msg_send![class!(NSColor), colorWithCalibratedRed: 0.15_f64 green: 0.15_f64 blue: 0.15_f64 alpha: 0.16_f64];
            let border: id = msg_send![class!(NSColor), colorWithCalibratedRed: 1.0_f64 green: 1.0_f64 blue: 1.0_f64 alpha: 0.45_f64];
            let bg_color: id = msg_send![bg, CGColor];
            let border_color: id = msg_send![border, CGColor];
            let _: () = msg_send![layer, setBackgroundColor: bg_color];
            let _: () = msg_send![layer, setBorderColor: border_color];
            let _: () = msg_send![layer, setBorderWidth: 1.5_f64];
            let _: () = msg_send![content, addSubview: view];

            let label: id = msg_send![class!(NSTextField), alloc];
            let label: id = msg_send![label, initWithFrame: view_frame];
            let white: id = msg_send![class!(NSColor), whiteColor];
            let font: id = msg_send![class!(NSFont), boldSystemFontOfSize: 36.0_f64];
            let ns_text = unsafe { NSString::alloc(nil).init_str(text) };

            let _: () = msg_send![label, setBezeled: NO];
            let _: () = msg_send![label, setDrawsBackground: NO];
            let _: () = msg_send![label, setEditable: NO];
            let _: () = msg_send![label, setSelectable: NO];
            let _: () = msg_send![label, setAlignment: ALIGN_CENTER];
            let _: () = msg_send![label, setTextColor: white];
            let _: () = msg_send![label, setFont: font];
            let _: () = msg_send![label, setStringValue: ns_text];
            let _: () = msg_send![content, addSubview: label];

            cell_views[index] = view;
            cell_labels[index] = label;
        }

        (cell_views, cell_labels)
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

            for row in 0..3 {
                for col in 0..3 {
                    let index = (row * 3 + col) as usize;
                    let x = col as f64 * cell_width;
                    let y = height - ((row as f64 + 1.0) * cell_height);
                    let cell_frame =
                        NSRect::new(NSPoint::new(x, y), NSSize::new(cell_width, cell_height));
                    let label_frame = NSRect::new(
                        NSPoint::new(x, y + (cell_height * 0.34)),
                        NSSize::new(cell_width, 44.0),
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
