// config module is owned by main because we don't want to export it in the library
// its specific to this app
mod config;

use config::AppConfig;

use anyhow::{anyhow, Context, Result};
use global_hotkey::hotkey::{Code, Modifiers};

use rs_brite::{daemon, image_processor};

fn main() -> Result<()> {
    println!("Starting rs-brite...");

    let config = AppConfig::load()
        .context("Failed to load configuration")?;

    println!("[Main] Configuration loaded: Prefix='{}', Hotkey='{}', Exit Key='{}'",
        config.key_prefix, config.hotkey, config.exit_key);

    let hotkey: Code = config.hotkey.parse()
        .map_err(|e| anyhow!("Invalid hotkey code '{}': {:?}", config.hotkey, e))?;

    let exitkey: Code = config.exit_key.parse()
        .map_err(|e| anyhow!("Invalid exit_key code '{}': {:?}", config.exit_key, e))?;

    let modifiers = parse_modifiers(&config.key_prefix)
        .context("[Main] Failed to parse key_prefix modifiers")?;

    let brightness_factory = move || {
        let mut camera = image_processor::setup_camera()?;

        Ok(move || {
            if let Err(e) = image_processor::auto_brightness(&mut camera) {
                eprintln!("[Action] Error: {}", e);
            }
        })
    };

    if let Err(e) = daemon::run_daemon(modifiers, exitkey, hotkey, brightness_factory) {
        eprintln!("[Main] Critical Daemon Error: {}", e);
    }

    Ok(())
}

fn parse_modifiers(str_modifiers: &str) -> Result<Option<Modifiers>> {
    if str_modifiers.trim().is_empty() {
        return Ok(None);
    }

    let mut mods = Modifiers::empty();

    for part in str_modifiers.split('+') {
        match part.trim().to_lowercase().as_str() {
            "control" | "ctrl" | "lctrl" | "left_control" | "rctrl" | "right_control" => {
                mods |= Modifiers::CONTROL;
            },
            "shift" | "lshift" | "rshift" | "left_shift" | "right_shift" => {
                mods |= Modifiers::SHIFT;
            },
            "alt" | "option" | "lalt" | "ralt" | "left_alt" | "right_alt" => {
                mods |= Modifiers::ALT;
            },
            "super" | "cmd" | "command" | "win" | "meta" => {
                mods |= Modifiers::SUPER;
            },
            "" => continue,
            _ => return Err(anyhow!("Unknown modifier: {}", part)),
        }
    }

    if mods.is_empty() {
        Ok(None)
    } else {
        Ok(Some(mods))
    }
}