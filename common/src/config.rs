use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub ankiweb: AnkiWebConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub language: String,
    pub collection_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnkiWebConfig {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub auto_sync: bool,
    #[serde(default)]
    pub sync_on_exit: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                language: "en-GB".into(),
                collection_path: "/mnt/ext1/applications/pbanki/collection".into(),
            },
            ankiweb: AnkiWebConfig {
                username: String::new(),
                password: String::new(),
                token: None,
                auto_sync: false,
                sync_on_exit: false,
            },
        }
    }
}

const DEFAULT_CONFIG_WITH_COMMENTS: &str = r#"# pbAnki Configuration File
# Generated automatically - edit with care

[general]
# Language code for Anki i18n (e.g., "en-GB", "de", "fr", "ja", "es", "pt", "ru", "zh", "ko")
language = "en-GB"

# Collection path (relative to app directory or absolute)
collection_path = "/mnt/ext1/applications/pbanki/collection"

[ankiweb]
# AnkiWeb synchronization settings
# Leave empty to disable sync

# AnkiWeb username (email)
username = ""

# Password (plain text - will be encrypted in future versions)
# WARNING: Do not share this file if password is filled
password = ""

# Session token (populated after successful login)
# This avoids storing password long-term
token = ""

# Sync automatically on app start
auto_sync = false

# Sync automatically after session ends
sync_on_exit = false
"#;

impl Config {
    pub fn load_or_create(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(path);

        if path.exists() {
            let contents = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save_with_comments(path.to_str().unwrap())?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = "/mnt/ext1/applications/pbanki/config.toml";
        let toml_string = toml::to_string_pretty(self)?;

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, toml_string)?;
        Ok(())
    }

    fn save_with_comments(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, DEFAULT_CONFIG_WITH_COMMENTS)?;
        Ok(())
    }

    pub fn update_and_save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.save()
    }
}
