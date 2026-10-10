use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;

use crate::api::events::emit_config_change_event;

/// Default configuration, written to the user's config directory on first run.
const DEFAULT_CONFIG: &str = include_str!("../config.yaml");
const CONFIG_FILE_NAME: &str = "config.yaml";

/// Session name that matches every audio session not explicitly listed in the config.
pub const OTHER_SESSION: &str = "other";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KeybindConfig {
    pub key: String,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MixerConfig {
    pub enabled: bool,
    pub hotkey: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SystemConfig {
    pub autostart: bool,
    pub show_console: bool,
    pub theme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SessionConfig {
    pub name: String,
    pub keybinds: Option<Vec<KeybindConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub sessions: Vec<SessionConfig>,
    pub mixer: MixerConfig,
    pub system: SystemConfig,
}

impl Default for Config {
    fn default() -> Self {
        serde_yaml_ng::from_str(DEFAULT_CONFIG).expect("bundled default config is valid")
    }
}

impl Config {
    /// Lowercased names of every configured session except the `other` catch-all.
    pub fn defined_session_names(&self) -> Vec<String> {
        self.sessions
            .iter()
            .map(|session| session.name.to_lowercase())
            .filter(|name| name != OTHER_SESSION)
            .collect()
    }
}

struct ConfigStore {
    path: PathBuf,
    config: Config,
}

static STORE: OnceLock<Mutex<ConfigStore>> = OnceLock::new();

/// Loads the config from `config_dir`, creating it on first run. Must be called before any other function in this module.
pub fn init(config_dir: &Path) {
    let path = config_dir.join(CONFIG_FILE_NAME);

    if !path.exists()
        && let Err(e) = create_config_file(&path)
    {
        log::error!("Failed to create config file at {}: {}", path.display(), e);
    }

    let config = match load_config(&path) {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load config from {}, using defaults: {}", path.display(), e);
            Config::default()
        }
    };

    log::info!("Loaded config from {}", path.display());

    if STORE.set(Mutex::new(ConfigStore { path, config })).is_err() {
        log::warn!("Config already initialized");
    }
}

fn store() -> &'static Mutex<ConfigStore> {
    STORE.get().expect("config::init must be called before using the config")
}

pub fn get_config() -> Config {
    store().lock().unwrap().config.clone()
}

pub fn get_defined_session_names() -> Vec<String> {
    get_config().defined_session_names()
}

pub fn set_config(config: Config, app_handle: &AppHandle) -> Result<(), String> {
    {
        let mut store = store().lock().unwrap();
        save_config(&store.path, &config).map_err(|e| format!("Failed to save config: {e}"))?;
        store.config = config.clone();
        log::info!("Saved config to {}", store.path.display());
    }

    emit_config_change_event(&config, app_handle);
    Ok(())
}

/// Seeds a new config file, carrying over the config that older versions kept next to the executable.
fn create_config_file(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let legacy_path = std::env::current_exe()?.with_file_name(CONFIG_FILE_NAME);
    if legacy_path.exists() && load_config(&legacy_path).is_ok() {
        log::info!("Migrating config from {}", legacy_path.display());
        fs::copy(&legacy_path, path)?;
    } else {
        fs::write(path, DEFAULT_CONFIG)?;
    }

    Ok(())
}

fn load_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    Ok(serde_yaml_ng::from_str(&contents)?)
}

fn save_config(path: &Path, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_yaml_ng::to_string(config)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_parses() {
        let config = Config::default();
        assert!(config.mixer.enabled);
        assert_eq!(config.mixer.hotkey.as_deref(), Some("Ctrl + Shift + M"));
        assert_eq!(config.sessions.first().map(|s| s.name.as_str()), Some("master"));
    }

    #[test]
    fn defined_session_names_excludes_other() {
        let mut config = Config::default();
        config.sessions.push(SessionConfig {
            name: "Spotify".into(),
            keybinds: None,
        });

        let names = config.defined_session_names();
        assert!(names.contains(&"master".to_string()));
        assert!(names.contains(&"spotify".to_string()));
        assert!(!names.contains(&OTHER_SESSION.to_string()));
    }

    #[test]
    fn config_round_trips_through_file() {
        let dir = std::env::temp_dir().join(format!("maestro-config-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CONFIG_FILE_NAME);

        let mut config = Config::default();
        config.system.theme = Some("light".into());
        config.mixer.hotkey = None;

        save_config(&path, &config).unwrap();
        assert_eq!(load_config(&path).unwrap(), config);

        fs::remove_dir_all(&dir).unwrap();
    }
}
