mod daemon;
mod image_processor;

use anyhow::Result;
use global_hotkey::hotkey::{Code, Modifiers};

fn main() -> Result<()> {
    println!("Starting rs-brite...");

    let hotkey = Code::F12;
    let exitkey = Code::Escape;
    let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;

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
