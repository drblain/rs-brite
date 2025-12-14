// config module is owned by main because we don't want to export it in the library
// its specific to this app
mod config;

use config::AppConfig;

use anyhow::{anyhow, Context, Result};

use rs_brite::{daemon, image_processor};

fn main() -> Result<()> {
    println!("Starting rs-brite...");

    let config = AppConfig::load()
        .context("Failed to load configuration")?;

    println!("[Main] Configuration loaded: Prefix='{}', Hotkey='{}', Exit Key='{}'",
        config.key_prefix, config.hotkey, config.exit_key);

    let hotkey = config.hotkey()
        .map_err(|e| anyhow!("Invalid hotkey code '{}': {:?}", config.hotkey, e))?;

    let exitkey = config.exit_key()
        .map_err(|e| anyhow!("Invalid exit_key code '{}': {:?}", config.exit_key, e))?;

    let modifiers = config.modifiers()
        .context("[Main] Failed to parse key_prefix modifiers")?;

    let brightness_factory = move || {
        let mut camera = image_processor::setup_camera()?;

        Ok(move || {
            if let Err(e) = image_processor::auto_brightness(&mut camera) {
                eprintln!("[Action] Error: {}", e);
            }
        })
    };

    if let Err(e) = daemon::run_daemon(Some(modifiers), exitkey, hotkey, brightness_factory) {
        eprintln!("[Main] Critical Daemon Error: {}", e);
    }

    Ok(())
}