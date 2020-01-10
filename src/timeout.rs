use crate::PolymerWindowEvent;
use rlua::{Context, Function, Table};
use std::time::Duration;
use winit::event_loop::EventLoopProxy;

pub const TIMEOUTS: &str = "POLYMER_TIMEOUTS";
// TODO: Just clone this value around?
pub const TIMEOUTS_EVENT_PROXY: &str = "POLYMER_EVENT_LOOP_PROXY";

pub fn add_timer<'lua>(
    lua: Context<'lua>,
    (duration, cb): (u64, Function<'lua>),
) -> rlua::Result<()> {
    let timers: Table = lua.named_registry_value(TIMEOUTS)?;
    let index = timers.len()? + 1;
    timers.raw_set(index, cb)?;

    let rlua::LightUserData(proxy) = lua.named_registry_value(TIMEOUTS_EVENT_PROXY)?;
    let proxy = unsafe { &*(proxy as *mut EventLoopProxy<PolymerWindowEvent>) }.clone();

    // TODO: Currently one thread per timer, implement scheduling
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(duration));
        let _ = proxy.send_event(PolymerWindowEvent::Timer(index));
    });

    Ok(())
}
