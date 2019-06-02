use std::fs::DirBuilder;
use std::path::PathBuf;

pub struct Config;

impl Config {
    pub fn read() -> Option<String> {
        let config_path = Config::config_path()?;

        if !config_path.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(config_path.parent()?)
                .ok()?;

            std::fs::write(&config_path, DEFAULT_CONFIG).ok()?;
            Some(DEFAULT_CONFIG.to_owned())
        } else {
            std::fs::read_to_string(config_path).ok()
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        Config::data_root().map(|h| h.join("polymer.lua"))
    }

    #[cfg(target_os = "macos")]
    pub fn data_root() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".config/polymer/"))
    }

    #[cfg(not(target_os = "macos"))]
    pub fn data_root() -> Option<PathBuf> {
        dirs::config_dir().map(|h| h.join("polymer/"))
    }
}

const DEFAULT_CONFIG: &'static str = r#"

"#;
