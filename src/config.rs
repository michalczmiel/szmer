use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/szmer";
const CONFIG_FILE: &str = "config.json";

/// Configuration for Timewarrior integration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TimewarriorConfig {
    /// Whether Timewarrior integration is enabled
    #[serde(default)]
    pub enabled: bool,
}

/// Main application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Notification sound name (None = system default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_sound: Option<String>,
    /// Whether notifications are paused
    #[serde(default)]
    pub paused: bool,
    /// Break reminder interval in seconds
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    /// Timewarrior integration settings
    #[serde(default)]
    pub timewarrior: TimewarriorConfig,
}

fn default_interval() -> u64 {
    3600 // 1 hour default
}

impl Default for Config {
    fn default() -> Self {
        Self {
            notification_sound: None,
            paused: false,
            interval_seconds: default_interval(),
            timewarrior: TimewarriorConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(config_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(config_path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join(CONFIG_DIR).join(CONFIG_FILE))
    }
}
