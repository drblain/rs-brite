# rs-brite

rs-brite is a Rust-based background daemon that automatically adjusts your screen brightness based on ambient light levels detected by your webcam. It features global hotkey support to trigger brightness adjustments on demand.

## Features

Ambient Light Detection: Captures frames from the primary webcam to calculate the average luminance of the environment.

Smart Compensation: Attempts to account for light reflected from the screen itself onto the user (screen contribution) to prevent feedback loops.

Global Hotkeys: Runs in the background and responds to keyboard shortcuts regardless of which application is in focus.

Performance: Uses rayon for parallel processing of image data and nokhwa for efficient camera access.

## Installation & Build

Ensure you have Rust and Cargo installed.

    # Clone the repository
    git clone <repository-url>
    cd rs-brite

    # Build the project
    cargo build --release

## Usage

Run the executable directly. The application initializes a daemon that listens for specific global hotkeys.
Bash

    ./target/release/rs-brite

### Controls

| Action | Hotkey |
|---|---|
| Auto-Adjust Brightness | Ctrl + Shift + F12 |
| Exit Application | Ctrl + Shift + Esc |

## How It Works

Initialization: The app sets up a GlobalHotKeyManager and waits for input.

Trigger: When Ctrl+Shift+F12 is pressed, the worker thread wakes up.

Capture: It opens the default camera (index 0), discards warmup frames, and captures a single RGB frame.

Processing:

- It calculates the raw luma (brightness) of the image.

- It calculates a "Screen Contribution" factor based on the monitor's current brightness setting.

- It subtracts the screen contribution from the raw luma to estimate actual ambient light.

Adjustment: The calculated ambient luma is mapped to a brightness percentage, and the primary display is updated via the brightness crate.

## TODO

[ ] Advanced Screen Contribution: Include support for factoring in luma of a screen capture to adjust screen_contribution.

Context: Currently, the app estimates light coming from the monitor based solely on the hardware brightness percentage. Taking a screenshot would allow the app to know if the user is looking at a dark image vs. a white document, improving the accuracy of the ambient light calculation.

[ ] Configurability: Add a configuration file to customize hotkeys, camera index, and sensitivity thresholds.

[ ] Smoothing: Implement a moving average or hysteresis to prevent sudden brightness jumps if lighting changes momentarily.

[ ] Multi-Monitor Support: Allow selection of specific monitors to control rather than just the primary device.
