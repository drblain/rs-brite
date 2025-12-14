use anyhow::{anyhow, Result};
use x11rb::protocol::xproto::ConnectionExt;
use std::{sync::mpsc, thread, process};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{ModMask, GrabMode};
use xkeysym::Keysym;

// Job is a closure that can be called mutably and lives for the whole program
pub trait Job: FnMut() + 'static {}
impl<F: FnMut() + 'static> Job for F {}
pub trait Recipe<J: Job>: FnOnce() -> Result<J> + Send + 'static {}
impl<F, J: Job> Recipe<J> for F where F: FnOnce() -> Result<J> + Send + 'static {}

pub fn run_daemon<J>(modifiers: Option<ModMask>, close_key: Keysym, trigger_key: Keysym, initializer: impl Recipe<J>) -> Result<()>
where
    J: Job
{
    let (xconn, nscreen) = x11rb::connect(None).map_err(|e| {
        anyhow!("[Daemon] Failed to establish connection to X server: {:?}", e)
    })?;

    let xsetup = xconn.setup();
    let root = xsetup.roots[nscreen].root;

    let keycode_min = xsetup.min_keycode;
    let keycode_max = xsetup.max_keycode;
    let keycode_count = keycode_max - keycode_min + 1;

    let mapping_res = xconn
        .get_keyboard_mapping(keycode_min, keycode_count)
        .map_err(|e| anyhow!("[Daemon] Failed to request keymap: {}", e))?
        .reply()
        .map_err(|e| anyhow!("[Daemon] Failed to receive keymap response: {}", e))?;

    let hotkey = resolve_keycode(trigger_key, keycode_min, mapping_res.keysyms_per_keycode, &mapping_res.keysyms)
        .ok_or_else(|| anyhow!("[Daemon] could not find keycode for trigger key: {}", u32::from(trigger_key)))?;
    let exitkey = resolve_keycode(close_key, keycode_min, mapping_res.keysyms_per_keycode, &mapping_res.keysyms)
        .ok_or_else(|| anyhow!("[Daemon] could not find keycode for trigger key: {}", u32::from(close_key)))?;

    // convert Keysym slice to ModMask
    let mods = match modifiers {
        Some(m) => m,
        None => ModMask::ANY
    };

    grab_hotkey_variants(&xconn, root, hotkey, mods)
        .map_err(|e| {
            anyhow!("[Daemon] Mapping error while grabbing key: ({}, {})", hotkey, e)
        })?;

    grab_hotkey_variants(&xconn, root, exitkey, mods)
        .map_err(|e| {
            anyhow!("[Daemon] Mapping error while grabbing key: ({}, {})", exitkey, e)
        })?;

    xconn.flush()?;

    let (transmit, receive) = mpsc::channel::<()>();

    thread::spawn(move || {
        println!("[Daemon] Worker thread starting...");

        let mut action = match initializer() {
            Ok(job) => job,
            Err(e) => {
                eprintln!("[Daemon] Startup failed: {}", e);
                return;
            }
        };

        println!("[Daemon] Worker thread is ready and waiting.");

        // This for loop is syntactic sugar for a match on Ok(()) and Err(e)
        // Err(e) exits the loop
        for _ in receive {
            println!("[Daemon] Worker received hotkey signal! Executing action...");
            action();
        }

        println!("[Daemon] Sender disconnected. Worker shutting down.");
    });

    println!("[Daemon] Starting hotkey listener...");

    while let Ok(event) = xconn.wait_for_event() {
        if let Event::KeyPress(ev) = event {
            if ev.detail == hotkey {
                println!("[Daemon] Hotkey pressed! Signaling worker thread...");

                if let Err(e) = transmit.send(()){
                    eprintln!("[Daemon] Failed to send signal to worker thread with error: {}. Exiting daemon.", e);
                    process::exit(1);
                }
            } else if ev.detail == exitkey {
                println!("[Daemon] Exit hotkey pressed! Shutting down daemon...");
                process::exit(0);
            }
        }
    }

    Ok(())
}

fn resolve_keycode(key_target: Keysym, keycode_min: u8, keysyms_per_keycode: u8, keymapping: &[u32]) -> Option<u8> {
    for (i, &symbol) in keymapping.iter().enumerate() {
        if symbol == u32::from(key_target) {
            let offset_keycode = i / (keysyms_per_keycode as usize);
            return Some(keycode_min + offset_keycode as u8);
        }
    }
    None
}

fn grab_hotkey_variants(xconn: &impl ConnectionExt, root: u32, keycode: u8, base_mods: ModMask) -> Result<()> {
    let ignored_masks = [
        ModMask::from(0u8),
        ModMask::M2,
        ModMask::LOCK,
        ModMask::M2 | ModMask::LOCK
    ];

    for ignored_case in ignored_masks {
        xconn.grab_key(
            false,
            root,
            base_mods | ignored_case,
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC
        )?.check()?;
    }

    Ok(())
}