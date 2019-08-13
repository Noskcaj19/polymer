use crate::{signals, timeout, PolymerWindowEvent};
use rlua::{Context, Function, Table};
use winit::event_loop::EventLoopProxy;

pub const CONTEXT_FROM_SURFACE_KEY: &str = "POLYMER_CONTEXT_FROM_SURFACE";

pub fn create_bindings<'lua>(lua: &Context<'lua>) -> rlua::Result<Table<'lua>> {
    let polymer_table = lua.create_table()?;
    let context_from_surface: Function = lua
        .load(
            r#"
                function(surface)
                    local cairo = require('lgi').cairo
                    return cairo.Context(cairo.Surface(surface))
                end"#,
        )
        .eval()?;

    lua.set_named_registry_value(CONTEXT_FROM_SURFACE_KEY, context_from_surface)?;

    polymer_table.set(
        "request_redraw",
        lua.create_function(|lua, ()| {
            let rlua::LightUserData(event_proxy) =
                lua.named_registry_value(crate::timeout::TIMEOUTS_EVENT_PROXY)?;
            let event_proxy = unsafe { &*(event_proxy as *mut EventLoopProxy<PolymerWindowEvent>) };
            let _ = event_proxy.send_event(PolymerWindowEvent::RedrawRequested);
            Ok(())
        })?,
    )?;

    polymer_table.set("add_timer", lua.create_function(timeout::add_timer)?)?;
    polymer_table.set(
        "connect_signal",
        lua.create_function(signals::connect_signal)?,
    )?;
    polymer_table.set("emit_signal", lua.create_function(signals::emit_signal)?)?;
    polymer_table.set(
        "trace",
        lua.create_function(|_, msg: String| {
            log::trace!(target: "polymer::user", "{}", msg);
            Ok(())
        })?,
    )?;
    polymer_table.set(
        "info",
        lua.create_function(|_, msg: String| {
            log::info!(target: "polymer::user", "{}", msg);
            Ok(())
        })?,
    )?;
    polymer_table.set(
        "warn",
        lua.create_function(|_, msg: String| {
            log::warn!(target: "polymer::user", "{}", msg);
            Ok(())
        })?,
    )?;
    polymer_table.set(
        "error",
        lua.create_function(|_, msg: String| {
            log::error!(target: "polymer::user", "{}", msg);
            Ok(())
        })?,
    )?;

    Ok(polymer_table)
}
