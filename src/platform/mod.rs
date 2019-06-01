pub use inner_platform::*;

pub type DrawFn = fn(cr: &cairo::Context, width: f64, height: f64);

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod inner_platform;
