use anyhow::{anyhow, Result};
use rdev::{listen, Event, EventType, Key};
use std::{sync::mpsc, thread, process};

pub trait Job: FnMut() + 'static {}
impl<F: FnMut() + 'static> Job for F {}
pub trait Recipe<J: Job>: FnOnce() -> Result<J> + Send + 'static {}
impl<F, J: Job> Recipe<J> for F where F: FnOnce() -> Result<J> + Send + 'static {}

// Job is a closue that
// 1. can be called mutably (FnMut())
// 2. can be sent between threads (Send)
// 3. lives for the whole program ('static)
pub fn run_daemon<J>(trigger_key: Key, initializer: impl Recipe<J>) -> Result<()>
where
    J: Job
{
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
    
    let cb_listener = move |event: Event| {
        match event.event_type {
            EventType::KeyPress(key) if key == trigger_key => {
                if let Err(e) = transmit.send(()) {
                        eprintln!("[Daemon] Worker thread is dead with {}. Shutting down.", e);
                        process::exit(1);
                }
            }
            _ => {}
        }
    };

    listen(cb_listener)
        .map_err(|e| {
            anyhow!("[Daemon] Fatal listener error: {:?}", e)
        })
}