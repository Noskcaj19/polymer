use std::fs::DirBuilder;

pub struct Config {}

impl Config {
    pub fn load() -> Option<Config> {
        let config_path = Config::config_path()?;

        if !config_path.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(config_path.parent()?)
                .ok()?;

            std::fs::write(&config_path, DEFAULT_CONFIG).ok()?;
        }

        let config_str = std::fs::read_to_string(config_path).ok()?;

        Some(Config {})
    }

    pub fn config_path() -> Option<std::path::PathBuf> {
        Config::data_root().map(|h| h.join("config.toml"))
    }

    #[cfg(target_os = "macos")]
    pub fn data_root() -> Option<std::path::PathBuf> {
        dirs::home_dir().map(|h| h.join(".config/polymer/"))
    }

    #[cfg(not(target_os = "macos"))]
    pub fn data_root() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|h| h.join("polymer/"))
    }
}

const DEFAULT_CONFIG: &'static str = r#"

"#;
