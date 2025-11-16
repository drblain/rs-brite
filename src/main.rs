mod daemon;
mod image_processor;

use rdev::Key;

fn main() -> Result<()> {
    println!("Starting rs-brite...");

    let hotkey = Key::F12;

    let mut camera = match image_processor::setup_camera() {
        Ok(cam) => cam,
        Err(e) => {
            eprintln!("[Main] Critical Error: Camera setup failed: {}", e);
            Err(())
        }
    };

    println!("[Main] Camera initialized. Handing off to daemon.");

    let brightness_action = move || {
        if let Err(e) = image_processor::auto_brightness(&mut camera) {
            eprintln!("[Action] Error during automatic brightness adjustment: {}", e);
        }
    };

    if let Err(e) = daemon::run_daemon(hotkey, brightness_action) {
        eprintln!("[Main] Critical Daemon Error: {}", e);
    }

    Ok(())
}
