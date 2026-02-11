use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled_device_ids: HashSet<String>,
}

impl AppConfig {
    fn config_path(app: &AppHandle) -> PathBuf {
        app.path()
            .app_config_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("config.json")
    }

    pub fn load(app: &AppHandle) -> Self {
        let path = Self::config_path(app);
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path(app);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
