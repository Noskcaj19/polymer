use rlua::{Function, Lua};

mod config;
mod platform;
mod signals;

pub use config::Config;

pub type DrawFn = fn(poly: &crate::Polymer, cr: &cairo::Context, width: f64, height: f64);

pub struct Polymer {
    lua: Lua,
}

fn init_lua(polymer: &Polymer) -> rlua::Result<()> {
    polymer.lua.context(|lua| {
        lua.set_named_registry_value(signals::GLOBAL_SIGNALS, lua.create_table()?)?;

        let connect_signal = lua.create_function(signals::connect_signal)?;
        let emit_signal = lua.create_function(signals::emit_signal)?;

        let polymer_table = lua.create_table()?;

        polymer_table.set("connect_signal", connect_signal)?;
        polymer_table.set("emit_signal", emit_signal)?;

        lua.globals().set("polymer", polymer_table)?;

        // Append the config dir to the lua require search path
        let package: rlua::Table = lua.globals().get("package")?;
        let path: String = package.get("path")?;

        let config_dir = Config::data_root().unwrap();

        package.set(
            "path",
            format!("{};{}", config_dir.join("?.lua").to_str().unwrap(), path),
        )?;

        Ok(())
    })
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

            let draw_ctx = lua.create_table()?;
            draw_ctx.set("width", width)?;
            draw_ctx.set("height", height)?;

            // Emit the draw signal
            signals::emit_signal_inner(lua, "draw", (context, draw_ctx))?;

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

    init_lua(&polymer).unwrap();

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
