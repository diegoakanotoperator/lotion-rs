use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User-facing configuration persisted to disk as TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotionConfig {
    pub active_theme: String,
    pub custom_css_path: Option<PathBuf>,
    pub restore_tabs: bool,
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub maximized: bool,
}

impl Default for LotionConfig {
    fn default() -> Self {
        Self {
            active_theme: "dracula".to_string(),
            custom_css_path: None,
            restore_tabs: true,
            window: WindowConfig {
                width: 1200.0,
                height: 800.0,
                x: None,
                y: None,
                maximized: false,
            },
        }
    }
}

impl LotionConfig {
    /// Returns the config directory path (~/.config/lotion/)
    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lotion")
    }

    /// Returns the config file path (~/.config/lotion/config.toml)
    fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Load config from disk, or create default if not found.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str::<LotionConfig>(&contents) {
                    Ok(config) => {
                        log::info!("Config loaded from {}", path.display());
                        return config;
                    }
                    Err(e) => {
                        log::warn!("Failed to parse config, using defaults: {}", e);
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read config file, using defaults: {}", e);
                }
            }
        } else {
            log::info!(
                "No config file found, creating default at {}",
                path.display()
            );
        }

        let config = Self::default();
        let _ = config.save(); // Best-effort save of defaults
        config
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;

        let contents = toml::to_string_pretty(self)?;
        fs::write(Self::config_path(), contents)?;

        log::info!("Config saved to {}", Self::config_path().display());
        Ok(())
    }
}
