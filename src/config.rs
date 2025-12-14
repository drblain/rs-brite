use serde::Deserialize;
use config::{Config, File};
use anyhow::{anyhow, Result};
use xkeysym::Keysym;
use x11rb::protocol::xproto::ModMask;

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

    pub fn hotkey(&self) -> Result<Keysym> {
        parse_keysym(&self.hotkey).map_err(|e| {
            anyhow!("Invalid 'hotkey' in configuration: ({}, {})", self.hotkey, e)
        })
    }

    pub fn exit_key(&self) -> Result<Keysym> {
        parse_keysym(&self.exit_key).map_err(|e| {
            anyhow!("Invalid 'exit_key' in configuration: ({}, {})", self.exit_key, e)
        })
    }

    pub fn modifiers(&self) -> Result<ModMask> {
        parse_modifiers(&self.key_prefix).map_err(|e| {
            anyhow!("Invalid 'key_prefix' in configuration: ({}, {})", self.exit_key, e)
        })
    }
}

fn parse_keysym(key_name: &str) -> Result<Keysym> {
    match key_name.trim().to_lowercase().as_str() {
        "escape" | "esc" => Ok(Keysym::Escape),
        "return" | "enter" => Ok(Keysym::Return),
        "space" => Ok(Keysym::space),
        "tab" => Ok(Keysym::Tab),
        "backspace" => Ok(Keysym::BackSpace),
        "f1" => Ok(Keysym::F1),
        "f2" => Ok(Keysym::F2),
        "f3" => Ok(Keysym::F3),
        "f4" => Ok(Keysym::F4),
        "f5" => Ok(Keysym::F5),
        "f6" => Ok(Keysym::F6),
        "f7" => Ok(Keysym::F7),
        "f8" => Ok(Keysym::F8),
        "f9" => Ok(Keysym::F9),
        "f10" => Ok(Keysym::F10),
        "f11" => Ok(Keysym::F11),
        "f12" => Ok(Keysym::F12),
        // Add more keys as needed
        _ => Err(anyhow!("Unknown or unsupported key: '{}'", key_name)),
    }
}

fn parse_modifiers(str_modifiers: &str) -> Result<ModMask> {
    if str_modifiers.trim().is_empty() {
        return Ok(ModMask::ANY);
    }

    let mut mods = ModMask::default();

    for part in str_modifiers.split('+') {
        match part.trim().to_lowercase().as_str() {
            "control" | "ctrl" | "lctrl" | "rctrl" => mods |= ModMask::CONTROL,
            "shift" | "lshift" | "rshift" => mods |= ModMask::SHIFT,
            "alt" | "lalt" | "ralt" | "option" => mods |= ModMask::M1,
            "super" | "cmd" | "win" | "meta" => mods |= ModMask::M4,
            "" => continue,
            _ => return Err(anyhow!("Unknown modifier: '{}'", part)),
        }
    }

    Ok(mods)
}