use anyhow::Result;
use rdev::{listen, Event, EventType, Key};
use std::{sync::mpsc, thread};

pub trait Job: FnMut() + Send + 'static {}
impl<F: FnMut() + Send + 'static> Job for F {}

// Job is a closue that
// 1. can be called mutably (FnMut())
// 2. can be sent between threads (Send)
// 3. lives for the whole program ('static)
pub fn run_daemon(trigger_key: Key, mut action: impl Job) -> Result<()>
{
    let (transmit, receive) = mpsc::channel::<()>();

    thread::spawn(move || {
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
                match transmit.send(()) {
                    Err(e) => {
                        eprintln!("[Daemon] Worker thread is dead with {}. Shutting down.", e);
                        process::exit(1);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    };

    listen(cb_listener)?;

    Ok(())
}