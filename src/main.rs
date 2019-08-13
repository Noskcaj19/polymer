use log::{debug, error, trace};
use rlua::{Function, Lua};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

mod bindings;
mod config;
mod platform;
mod signals;
mod timeout;

pub use config::Config;
use std::sync::Arc;

#[derive(Debug)]
pub enum PolymerWindowEvent {
    RedrawRequested,
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

        let polymer_bindings = bindings::create_bindings(&lua)?;
        lua.globals().set("__polymer_sys", polymer_bindings)?;

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

            let get_context_fn: Function =
                lua.named_registry_value(bindings::CONTEXT_FROM_SURFACE_KEY)?;

            let context: rlua::Value =
                get_context_fn.call((rlua::LightUserData(cairo_surface_ptr as *mut _),))?;

            let draw_ctx = lua.create_table()?;
            draw_ctx.set("width", width)?;
            draw_ctx.set("height", height)?;

            // Emit the draw signal
            signals::emit_signal_inner(lua, "redraw_requested", (context, draw_ctx))?;

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

    let polymer = Arc::new(Polymer { lua: Lua::new() });

    {
        let event_loop = EventLoop::new_user_event();
        let window = platform::Window::new(&event_loop, Arc::clone(&polymer), &(draw as DrawFn));

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
                PolymerWindowEvent::RedrawRequested => {
                    window.window.request_redraw();
                }
            },

            _ => {
                *control_flow = ControlFlow::Wait;
            }
        });
    }
}
