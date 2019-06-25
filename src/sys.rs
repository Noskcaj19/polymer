use rlua::Context;

pub fn lua_info(_lua: Context, msg: String) -> rlua::Result<()> {
    log::info!(target: "polymer::user", "{}", msg);
    Ok(())
}

pub fn lua_warn(_lua: Context, msg: String) -> rlua::Result<()> {
    log::warn!(target: "polymer::user", "{}", msg);
    Ok(())
}

pub fn lua_error(_lua: Context, msg: String) -> rlua::Result<()> {
    log::error!(target: "polymer::user", "{}", msg);
    Ok(())
}
