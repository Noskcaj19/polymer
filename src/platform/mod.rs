pub use inner_platform::*;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod inner_platform;
