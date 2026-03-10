use crate::{launch_agent, runtime};
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyProhibited, NSStatusBar};
use cocoa::base::{NO, YES, id, nil};
use cocoa::foundation::NSString;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::OnceLock;

static STATUS_ITEM: OnceLock<usize> = OnceLock::new();
static TOGGLE_ITEM: OnceLock<usize> = OnceLock::new();
static START_AT_LOGIN_ITEM: OnceLock<usize> = OnceLock::new();
static HANDLER: OnceLock<usize> = OnceLock::new();
static HANDLER_CLASS: OnceLock<usize> = OnceLock::new();
const NS_VARIABLE_STATUS_ITEM_LENGTH: f64 = -1.0;

fn nsstring(value: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(value) }
}

fn status_button_title(mouse_mode: bool) -> &'static str {
    if mouse_mode { "KM*" } else { "KM" }
}

fn refresh_menu_state(mouse_mode: bool) {
    unsafe {
        if let Some(status_item) = STATUS_ITEM.get() {
            let button: id = msg_send![*status_item as id, button];
            if button != nil {
                let _: () = msg_send![button, setTitle: nsstring(status_button_title(mouse_mode))];
            }
        }

        if let Some(toggle_item) = TOGGLE_ITEM.get() {
            let title = if mouse_mode {
                "Turn Mouse Mode Off"
            } else {
                "Turn Mouse Mode On"
            };
            let _: () = msg_send![*toggle_item as id, setTitle: nsstring(title)];
            let _: () = msg_send![*toggle_item as id, setEnabled: YES];
        }

        if let Some(start_item) = START_AT_LOGIN_ITEM.get() {
            let state = if launch_agent::is_enabled() {
                1_i64
            } else {
                0_i64
            };
            let _: () = msg_send![*start_item as id, setState: state];
        }
    }
}

extern "C" fn toggle_mouse_mode(_: &Object, _: Sel, _: id) {
    let enabled = runtime::mouse_mode_enabled();
    let next = !enabled;
    runtime::set_mouse_mode(next);
    refresh_menu_state(next);
}

extern "C" fn quit_app(_: &Object, _: Sel, _: id) {
    runtime::shutdown();
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, terminate: nil];
    }
}

extern "C" fn toggle_start_at_login(_: &Object, _: Sel, _: id) {
    match launch_agent::toggle() {
        Ok(_) => refresh_menu_state(runtime::mouse_mode_enabled()),
        Err(error) => eprintln!("Failed to toggle start at login: {error}"),
    }
}

fn handler_class() -> *const Class {
    if let Some(existing) = HANDLER_CLASS.get() {
        return *existing as *const Class;
    }

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("KeymouseMenuHandler", superclass)
        .expect("failed to register KeymouseMenuHandler class");
    unsafe {
        decl.add_method(
            sel!(toggleMouseMode:),
            toggle_mouse_mode as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(toggleStartAtLogin:),
            toggle_start_at_login as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(sel!(quitApp:), quit_app as extern "C" fn(&Object, Sel, id));
    }

    let class = decl.register();
    let _ = HANDLER_CLASS.set(class as *const Class as usize);
    class
}

unsafe fn add_action_item(menu: id, handler: id, title: &str, action: Sel) -> id {
    let item: id = msg_send![class!(NSMenuItem), alloc];
    let item: id =
        msg_send![item, initWithTitle: nsstring(title) action: action keyEquivalent: nsstring("")];
    let _: () = msg_send![item, setTarget: handler];
    let _: () = msg_send![menu, addItem: item];
    item
}

unsafe fn add_hint_item(menu: id, title: &str) {
    let item: id = msg_send![class!(NSMenuItem), alloc];
    let item: id =
        msg_send![item, initWithTitle: nsstring(title) action: nil keyEquivalent: nsstring("")];
    let _: () = msg_send![item, setEnabled: NO];
    let _: () = msg_send![menu, addItem: item];
}

unsafe fn add_separator(menu: id) {
    let separator: id = msg_send![class!(NSMenuItem), separatorItem];
    let _: () = msg_send![menu, addItem: separator];
}

pub fn run() {
    runtime::initialize();
    runtime::set_mouse_mode_listener(refresh_menu_state);

    unsafe {
        let app = NSApplication::sharedApplication(nil);
        let _: () = msg_send![app, setActivationPolicy: NSApplicationActivationPolicyProhibited];

        let status_bar = NSStatusBar::systemStatusBar(nil);
        let status_item = status_bar.statusItemWithLength_(NS_VARIABLE_STATUS_ITEM_LENGTH);
        let _ = STATUS_ITEM.set(status_item as usize);

        let button: id = msg_send![status_item, button];
        if button != nil {
            let _: () = msg_send![button, setTitle: nsstring(status_button_title(runtime::mouse_mode_enabled()))];
        }

        let menu: id = msg_send![class!(NSMenu), alloc];
        let menu: id = msg_send![menu, initWithTitle: nsstring("Keymouse")];

        let handler_class = handler_class();
        let handler: id = msg_send![handler_class, new];
        let _ = HANDLER.set(handler as usize);

        let toggle_item =
            add_action_item(menu, handler, "Turn Mouse Mode On", sel!(toggleMouseMode:));
        let _ = TOGGLE_ITEM.set(toggle_item as usize);
        refresh_menu_state(runtime::mouse_mode_enabled());

        add_separator(menu);
        add_hint_item(menu, "Hints");
        add_hint_item(menu, "F8: keyboard toggle");
        add_hint_item(menu, "Hold H/J/K/L: move cursor");
        add_hint_item(menu, "V: toggle drag hold");
        add_hint_item(menu, "; then Q..C: grid jump");
        add_hint_item(menu, "Enter: confirm grid, Esc: cancel");

        add_separator(menu);
        let start_at_login_item =
            add_action_item(menu, handler, "Start at Login", sel!(toggleStartAtLogin:));
        let _ = START_AT_LOGIN_ITEM.set(start_at_login_item as usize);

        add_separator(menu);
        let _ = add_action_item(menu, handler, "Quit", sel!(quitApp:));

        let _: () = msg_send![status_item, setMenu: menu];
        let _: () = msg_send![app, run];
    }
}
