use crate::{DrawFn, Polymer};

pub struct Window<'a> {
    pub window: winit::Window,
    polymer: &'a Polymer,
    draw: &'a DrawFn,
}

impl<'a> Window<'a> {
    pub fn new(
        events_loop: &winit::EventsLoop,
        polymer: &'a Polymer,
        draw: &'a DrawFn,
    ) -> Window<'a> {
        let window = winit::WindowBuilder::new()
            .with_transparency(true)
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

        (self.draw)(self.polymer, &cr, 512., 512.);
    }
}
