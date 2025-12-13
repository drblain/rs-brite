use anyhow::{anyhow, Result};
use std::{sync::mpsc, thread, process};

// Job is a closure that can be called mutably and lives for the whole program
pub trait Job: FnMut() + 'static {}
impl<F: FnMut() + 'static> Job for F {}
pub trait Recipe<J: Job>: FnOnce() -> Result<J> + Send + 'static {}
impl<F, J: Job> Recipe<J> for F where F: FnOnce() -> Result<J> + Send + 'static {}

pub fn run_daemon<J>(modifiers: Option<Modifiers>, close_key: Code, trigger_key: Code, initializer: impl Recipe<J>) -> Result<()>
where
    J: Job
{
    let manager = GlobalHotKeyManager::new().map_err(|e| {
        anyhow!("[Daemon] Failed to create hotkey manager: {:?}", e)
    })?;

    let hotkey = HotKey::new(modifiers, trigger_key);
    let exitkey = HotKey::new(modifiers, close_key);

    manager.register(hotkey).map_err(|e| {
        anyhow!("[Daemon] Failed to register hotkey: {:?}", e)
    })?;

    manager.register(exitkey).map_err(|e| {
        anyhow!("[Daemon] Failed to register exit hotkey: {:?}", e)
    })?;

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
    
    let hotkey_receiver = GlobalHotKeyEvent::receiver();

    for event in hotkey_receiver {
        match event {
            GlobalHotKeyEvent { id, state: HotKeyState::Pressed } if id == hotkey.id() => {
                println!("[Daemon] Hotkey pressed! Signaling worker thread...");

                if let Err(e) = transmit.send(()){
                    eprintln!("[Daemon] Failed to send signal to worker thread with error: {}. Exiting daemon.", e);
                    process::exit(1);
                }
            }

            GlobalHotKeyEvent { id, state: HotKeyState::Pressed } if id == exitkey.id() => {
                println!("[Daemon] Exit hotkey pressed! Shutting down daemon...");
                process::exit(0);
            }

            _ => {}
        }
    }

    Ok(())
}