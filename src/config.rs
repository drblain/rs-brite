use serde::Deserialize;
use config::{Config, File};
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,

    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    #[serde(default = "default_exit_key")]
    pub exit_key: String
}

fn default_key_prefix() -> String { "Control+Shift".to_string() }
fn default_hotkey() -> String { "F12".to_string() }
fn default_exit_key() -> String { "Escape".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            key_prefix: default_key_prefix(),
            hotkey: default_hotkey(),
            exit_key: default_exit_key()
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let builder = Config::builder()
            .add_source(File::with_name("/etc/rs-brite/config.toml").required(false))
            .add_source(File::with_name("rs-brite.toml").required(false))
            .add_source(config::Environment::with_prefix("RS_BRITE"));

        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }
}