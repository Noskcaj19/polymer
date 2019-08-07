use log::{debug, error, trace};
use rlua::{Function, Lua, Table};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

mod config;
mod platform;
mod signals;
mod sys;
mod timeout;

pub use config::Config;

#[derive(Debug)]
pub enum PolymerWindowEvent {
    Timer(i64),
}

pub type DrawFn = fn(poly: &crate::Polymer, cr: &cairo::Context, width: f64, height: f64);

pub struct Polymer {
    lua: Lua,
}

fn init_lua(polymer: &Polymer, proxy: EventLoopProxy<PolymerWindowEvent>) -> rlua::Result<()> {
    polymer.lua.context(|lua| {
        // TODO: Clean up proxy passing
        let proxy = Box::new(proxy);
        let proxy_ref = Box::leak(proxy);
        lua.set_named_registry_value(
            timeout::TIMEOUTS_EVENT_PROXY,
            rlua::LightUserData(proxy_ref as *mut _ as *mut _),
        )?;
        lua.set_named_registry_value(timeout::TIMEOUTS, lua.create_table()?)?;
        lua.set_named_registry_value(signals::GLOBAL_SIGNALS, lua.create_table()?)?;

        let add_timer = lua.create_function(timeout::add_timer)?;
        let connect_signal = lua.create_function(signals::connect_signal)?;
        let emit_signal = lua.create_function(signals::emit_signal)?;
        let context_from_surface: Function = lua
            .load(
                r#"
                function(surface)
                    local cairo = require('lgi').cairo
                    return cairo.Context(cairo.Surface(surface))
                end"#,
            )
            .eval()?;
        let lua_info = lua.create_function(sys::lua_info)?;
        let lua_warn = lua.create_function(sys::lua_warn)?;
        let lua_error = lua.create_function(sys::lua_error)?;

        let polymer_table = lua.create_table()?;

        polymer_table.set("add_timer", add_timer)?;
        polymer_table.set("connect_signal", connect_signal)?;
        polymer_table.set("emit_signal", emit_signal)?;
        polymer_table.set("context_from_surface", context_from_surface)?;
        polymer_table.set("info", lua_info)?;
        polymer_table.set("warn", lua_warn)?;
        polymer_table.set("error", lua_error)?;

        lua.globals().set("__polymer_sys", polymer_table)?;

        // Append the config dir to the lua require search path
        let package: rlua::Table = lua.globals().get("package")?;
        let path: String = package.get("path")?;

        let config_dir = Config::data_root().unwrap();
        let config_path = config_dir.join("?.lua").to_str().unwrap().to_owned();

        // TODO: Temporary - move libs to config dir?
        let poly_libs = concat!(env!("CARGO_MANIFEST_DIR"), "/lib/?.lua");

        debug!("[init] extended lua path: {:#?}", config_path);
        debug!("[init] extended lua path: {:#?}", poly_libs);

        let package_path = format!("{};{};{}", config_path, poly_libs, path);

        package.set("path", package_path)?;

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
                .globals()
                .get::<_, Table>("__polymer_sys")?
                .get("context_from_surface")?;

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
    pretty_env_logger::init();

    let config = match Config::read() {
        Some(config) => config,
        None => {
            eprintln!("Unable to load config file");
            std::process::exit(1);
        }
    };

    let polymer = Polymer { lua: Lua::new() };

    {
        let event_loop = EventLoop::new_user_event();
        let window = platform::Window::new(&event_loop, &polymer, &(draw as DrawFn));

        init_lua(&polymer, event_loop.create_proxy()).unwrap();

        if let Err(e) = polymer.lua.context(|lua| lua.load(&config).exec()) {
            error!("[config] Error loading config");
            eprintln!("Error loading user config file:\n");
            eprintln!("{}", e);
            std::process::exit(2);
        }

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                trace!("[events] Redrawing");
                window.refresh();
            }
            Event::UserEvent(event) => match event {
                PolymerWindowEvent::Timer(index) => {
                    polymer.lua.context(|lua| {
                        let timeouts = lua
                            .named_registry_value::<_, rlua::Table>(timeout::TIMEOUTS)
                            .unwrap();
                        let cb = timeouts.raw_get::<_, Function>(index).unwrap();
                        let () = cb.call(()).unwrap();
                    });
                }
            },

            _ => {
                *control_flow = ControlFlow::Wait;
            }
        });
    }
}
