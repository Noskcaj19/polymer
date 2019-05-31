#![feature(maybe_uninit)]

mod platform;

fn draw(cr: &cairo::Context, width: f64, height: f64) {
    cr.set_line_width(10.0);

    let (r, g, b) = (255. / 255., 0. / 255., 0. / 255.);
    cr.set_source_rgb(r, g, b);
    cr.rectangle(100., 100., width - 200., height - 200.);
    cr.stroke();
}

fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let window = platform::Window::new(&events_loop, draw);

    events_loop.run_forever(|event| match event {
        winit::Event::WindowEvent {
            event: winit::WindowEvent::Refresh,
            ..
        } => {
            window.refresh();
            winit::ControlFlow::Continue
        }
        winit::Event::WindowEvent {
            event: winit::WindowEvent::CloseRequested,
            ..
        } => winit::ControlFlow::Break,
        _ => winit::ControlFlow::Continue,
    });
}
