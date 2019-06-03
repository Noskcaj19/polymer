use log::{error, trace};
use rlua::{Context, Function, Table, ToLuaMulti, Value};

pub const GLOBAL_SIGNALS: &str = "POLYMER_SIGNALS";

pub fn connect_signal<'lua>(
    lua: Context<'lua>,
    (name, handler): (String, Function<'lua>),
) -> rlua::Result<()> {
    let handlers = lua.named_registry_value::<_, Table>(GLOBAL_SIGNALS)?;

    let count;

    if let Ok(Value::Table(table)) = handlers.get(name.as_str()) {
        count = table.len()?;
        table.set(count + 1, handler)?;
    } else {
        let table = lua.create_table()?;
        table.set(1, handler)?;
        handlers.set(name.as_str(), table)?;
        count = 1;
    }
    trace!("Connecting signal {} ({})", name, count);
    Ok(())
}

pub fn emit_signal_inner<'lua, A: ToLuaMulti<'lua> + Clone>(
    lua: Context<'lua>,
    name: &str,
    args: A,
) -> rlua::Result<()> {
    let handlers = lua.named_registry_value::<_, Table>(GLOBAL_SIGNALS)?;

    if let Ok(Value::Table(table)) = handlers.get::<_, Value>(name.clone()) {
        for entry in table.pairs::<Value, Function>() {
            if let Ok((_, func)) = entry {
                match func.call(args.clone()) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Error while emitting signal {}: {}", name, e);
                    }
                };
            }
        }
    }
    Ok(())
}

pub fn emit_signal<'lua>(
    lua: Context<'lua>,
    (name, args): (String, Value<'lua>),
) -> rlua::Result<()> {
    emit_signal_inner(lua, &name, args)
}
