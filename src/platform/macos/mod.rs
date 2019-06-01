use cocoa::base::id;
use cocoa::foundation::NSRect;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Object, Sel},
    sel, sel_impl,
};
use std::os::raw::c_void;
use winit::os::macos::{WindowBuilderExt, WindowExt};

use super::DrawFn;
use crate::Config;

pub struct Window {
    pub window: winit::Window,
    render_view: *mut Object,
}

extern "C" fn draw_rect(this: &Object, _cmd: Sel, _rect: NSRect) {
    unsafe {
        let window: id = msg_send![this, window];
        let frame = cocoa::appkit::NSWindow::frame(window);
        let size = frame.size;
        let width = size.width;
        let height = size.height;

        let current: id = msg_send![class!(NSGraphicsContext), currentContext];
        let context: id = msg_send![current, CGContext];

        let surface = cairo::QuartzSurface::create_for_cg_context(
            context as *mut std::ffi::c_void,
            width as u32,
            height as u32,
        )
        .unwrap();

        let d = (*this.get_ivar::<*const c_void>("drawFn")) as *const DrawFn;
        let config = (*this.get_ivar::<*const c_void>("config")) as *const Config;
        {
            let cr = cairo::Context::new(&surface);
            (*d)(&cr, &*config, width, height);
        }
    }
}

impl Window {
    pub fn new(events_loop: &winit::EventsLoop, config: &Config, draw: &DrawFn) -> Window {
        let window = winit::WindowBuilder::new()
            .with_transparency(true)
            .with_activation_policy(winit::os::macos::ActivationPolicy::Accessory)
            // If these aren't added it causes drop shadows and borders
            // we don't want that
            .with_resizable(false)
            .with_title_hidden(true)
            .with_titlebar_buttons_hidden(true)
            .with_titlebar_hidden(true)
            .with_decorations(false)
            .build(&events_loop)
            .unwrap();

        window.set_simple_fullscreen(true);

        let ns_window = window.get_nswindow() as id;
        make_background(ns_window);

        let render_view_class = make_draw_view_class();

        let ns_view = window.get_nsview() as id;

        let render_view = unsafe {
            let view_frame = cocoa::appkit::NSView::frame(ns_view);

            let render_view: id = msg_send![render_view_class, alloc];
            let render_view: id = msg_send![render_view, initWithFrame: view_frame];

            // Make draw function available in the drawing callback
            let fn_ptr = draw as *const DrawFn;
            (*render_view).set_ivar("drawFn", fn_ptr as *const c_void);

            (*render_view).set_ivar("config", config as *const _ as *const c_void);

            msg_send![ns_view, addSubview: render_view];
            render_view
        };

        Window {
            window,
            render_view,
        }
    }

    pub fn refresh(&self) {
        unsafe {
            msg_send![self.render_view, setNeedsDisplay: true];
        }
    }
}

fn make_draw_view_class() -> &'static objc::runtime::Class {
    unsafe {
        let mut decl = ClassDecl::new("PolymerDrawView", class!(NSView)).unwrap();
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&Object, Sel, NSRect),
        );
        decl.add_ivar::<*mut c_void>("drawFn");
        decl.add_ivar::<*mut c_void>("config");
        decl.register()
    }
}

pub fn make_background(ns_window: id) {
    use cocoa::appkit::NSWindowCollectionBehavior;

    let ns_window = ns_window as id;

    unsafe {
        msg_send![ns_window, setOpaque: false];
        msg_send![ns_window, setLevel: -1];

        let behaviors = NSWindowCollectionBehavior::NSWindowCollectionBehaviorTransient
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle;
        msg_send![ns_window, setCollectionBehavior: behaviors];

        msg_send![ns_window, setRestorable: false];
        msg_send![ns_window, disableSnapshotRestoration];
        msg_send![ns_window, setDisplaysWhenScreenProfileChanges: true];
        msg_send![ns_window, setReleasedWhenClosed: false];
        msg_send![ns_window, setIgnoresMouseEvents: true];
    }
}
