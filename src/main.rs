#![feature(maybe_uninit)]

use rlua::{Function, Lua};

mod config;
mod platform;

pub use config::Config;

pub type DrawFn = fn(poly: &crate::Polymer, cr: &cairo::Context, width: f64, height: f64);

pub struct Polymer {
    lua: Lua,
}

fn draw(polymer: &Polymer, cr: &cairo::Context, width: f64, height: f64) {
    polymer
        .lua
        .context(|lua| -> rlua::Result<()> {
            // It seems cairo.Context cannot be constructed from a userdata pointer, but a surface can
            // so get the cairo surface attached to the context and then create the context with lua

            let surface = cr.get_target();

            let cairo_surface_ptr = surface.to_raw_none();

            let get_context_fn: Function = lua
                .load(
                    r#"
                function(surface)
                    local cairo = require('lgi').cairo
                    return cairo.Context(cairo.Surface(surface))
                end"#,
                )
                .eval()?;
            let context: rlua::Value =
                get_context_fn.call((rlua::LightUserData(cairo_surface_ptr as *mut _),))?;

            let lua_polymer = lua.create_table()?;
            lua_polymer.set("width", width)?;
            lua_polymer.set("height", height)?;

            let globals = lua.globals();
            globals.set("polymer", lua_polymer)?;

            // Get the global draw function and call it with the lgi cairo context
            let draw_fn: Function = globals.get("draw")?;
            draw_fn.call::<_, ()>(context)?;

            Ok(())
        })
        .expect("Unable to setup cairo context in lua");
}

fn main() {
    let config = match Config::read() {
        Some(config) => config,
        None => {
            eprintln!("Unable to load config file");
            std::process::exit(1);
        }
    };

    let polymer = Polymer { lua: Lua::new() };

    if let Err(e) = polymer.lua.context(|lua| lua.load(&config).exec()) {
        eprintln!("Error loading user config file:\n");
        eprintln!("{}", e);
        std::process::exit(2);
    }

    {
        let mut events_loop = winit::EventsLoop::new();
        let window = platform::Window::new(&events_loop, &polymer, &(draw as DrawFn));

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
