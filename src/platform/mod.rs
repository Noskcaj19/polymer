pub use inner_platform::*;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod inner_platform;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod inner_platform;
