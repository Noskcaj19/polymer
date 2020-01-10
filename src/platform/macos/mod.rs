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
use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    platform::macos::{WindowBuilderExtMacOS, WindowExtMacOS},
    window::{Window as WinitWindow, WindowBuilder},
};

use crate::{DrawFn, Polymer};

pub struct Window {
    pub window: WinitWindow,
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

        let context_ref = context as *mut core_graphics::context::CGContextRef;
        (*context_ref).translate(0.0, height);
        (*context_ref).scale(1.0, -1.0);

        let surface = cairo::QuartzSurface::create_for_cg_context(
            context as *mut std::ffi::c_void,
            width as u32,
            height as u32,
        )
        .unwrap();

        let d = (*this.get_ivar::<*const c_void>("drawFn")) as *const DrawFn;
        let polymer = (*this.get_ivar::<*const c_void>("polymer")) as *const Polymer;
        {
            let cr = cairo::Context::new(&surface);
            (*d)(&*polymer, &cr, width, height);
        }
    }
}

impl Window {
    pub fn new(
        event_loop: &EventLoop<crate::PolymerWindowEvent>,
        polymer: Arc<Polymer>,
        draw: &DrawFn,
    ) -> Window {
        let window = WindowBuilder::new()
            .with_transparent(true)
            .with_activation_policy(winit::platform::macos::ActivationPolicy::Accessory)
            // If these aren't added it causes drop shadows and borders
            // we don't want that
            .with_resizable(false)
            .with_title_hidden(true)
            .with_titlebar_buttons_hidden(true)
            .with_titlebar_hidden(true)
            .with_decorations(false)
            .build(&event_loop)
            .unwrap();

        window.set_simple_fullscreen(true);

        let ns_window = window.ns_window() as id;
        make_background(ns_window);

        let render_view_class = make_draw_view_class();

        let ns_view = window.ns_view() as id;

        let render_view = unsafe {
            let view_frame = cocoa::appkit::NSView::frame(ns_view);

            let render_view: id = msg_send![render_view_class, alloc];
            let render_view: id = msg_send![render_view, initWithFrame: view_frame];

            // Make draw function available in the drawing callback
            (*render_view).set_ivar("drawFn", draw as *const _ as *const c_void);
            (*render_view).set_ivar("polymer", &*polymer as *const _ as *const c_void);

            let _: () = msg_send![ns_view, addSubview: render_view];
            render_view
        };

        Window {
            window,
            render_view,
        }
    }

    pub fn refresh(&self) {
        unsafe {
            let _: () = msg_send![self.render_view, setNeedsDisplay: true];
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
        decl.add_ivar::<*mut c_void>("polymer");
        decl.register()
    }
}

pub fn make_background(ns_window: id) {
    use cocoa::appkit::NSWindowCollectionBehavior;

    let ns_window = ns_window as id;

    unsafe {
        let _: () = msg_send![ns_window, setOpaque: false];
        let _: () = msg_send![ns_window, setLevel: -1];

        let behaviors = NSWindowCollectionBehavior::NSWindowCollectionBehaviorTransient
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle;
        let _: () = msg_send![ns_window, setCollectionBehavior: behaviors];

        let _: () = msg_send![ns_window, setRestorable: false];
        let _: () = msg_send![ns_window, disableSnapshotRestoration];
        let _: () = msg_send![ns_window, setDisplaysWhenScreenProfileChanges: true];
        let _: () = msg_send![ns_window, setReleasedWhenClosed: false];
        let _: () = msg_send![ns_window, setIgnoresMouseEvents: true];
    }
}
