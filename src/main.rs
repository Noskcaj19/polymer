#![feature(maybe_uninit)]

mod config;
mod platform;

use config::Config;

fn draw(cr: &cairo::Context, config: &Config, width: f64, height: f64) {
    cr.set_line_width(10.0);

    let (r, g, b) = (255. / 255., 0. / 255., 0. / 255.);
    cr.set_source_rgb(r, g, b);
    cr.rectangle(100., 100., width - 200., height - 200.);
    cr.stroke();
}

fn main() {
    let config = match Config::load() {
        Some(config) => config,
        None => {
            eprintln!("Unable to load config file");
            std::process::exit(1);
        }
    };

    let draw_ref: &platform::DrawFn = &(draw as platform::DrawFn);

    {
        let mut events_loop = winit::EventsLoop::new();
        let window = platform::Window::new(&events_loop, &config, draw_ref);

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
        })
    }
}
