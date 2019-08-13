use crate::{DrawFn, Polymer};
use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::window::{Window as WinitWindow, WindowBuilder};

pub struct Window<'a> {
    pub window: WinitWindow,
    polymer: Arc<Polymer>,
    draw: &'a DrawFn,
}

impl<'a> Window<'a> {
    pub fn new(
        events_loop: &EventLoop<crate::PolymerWindowEvent>,
        polymer: Arc<Polymer>,
        draw: &'a DrawFn,
    ) -> Window<'a> {
        let window = WindowBuilder::new()
            .with_transparent(true)
            .with_resizable(false)
            .with_decorations(false)
            .build(&events_loop)
            .unwrap();

        Window {
            window,
            polymer,
            draw,
        }
    }

    pub fn refresh(&self) {
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 512, 512).unwrap();
        let cr = cairo::Context::new(&surface);

        (self.draw)(&*self.polymer, &cr, 512., 512.);
    }
}
