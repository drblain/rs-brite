# rs-brite

rs-brite is a Rust-based background daemon that automatically adjusts your screen brightness based on ambient light levels detected by your webcam. It features global hotkey support to trigger brightness adjustments on demand.

## Features

* **Ambient Light Detection:** Captures frames from the primary webcam to calculate the average luminance of the environment.
* **Smart Compensation:** Attempts to account for light reflected from the screen itself onto the user (screen contribution) to prevent feedback loops.
* **Global Hotkeys:** Runs in the background and responds to keyboard shortcuts regardless of which application is in focus.
* **Configurable:** Customize hotkeys and modifiers via configuration files or environment variables.
* **Systemd Integration:** Includes user-level service files for easy background management.
* **Performance:** Uses `rayon` for parallel processing of image data and `nokhwa` for efficient camera access.

## Installation & Build

Ensure you have Rust and Cargo installed.

### 1. Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd rs-brite

# Build the release binary
cargo build --release
```

### 2. Install (Linux)

You can install the binary, configuration, and systemd service using the provided Makefile or install script.

#### Using Make

```bash
#Installs to /usr/bin/rs-brite and /etc/rs-brite/config.toml
sudo make install
```

#### Using Install Script

```bash
chmod +x install.sh
./install.sh
```

#### Debian/Ubuntu Packaging

The project includes metadata for cargo-deb. If you have cargo-deb installed, you can generate a .deb package:

```bash
cargo deb
```

## Configuration

rs-brite looks for configuration in the following order:

* /etc/rs-brite/config.toml

* rs-brite.toml (in the current working directory)

* Environment variables prefixed with RS_BRITE_

### Configuration Options

You can customize the control keys by editing /etc/rs-brite/config.toml.

Default Configuration:

```Ini, TOML
# Modifier keys required (e.g., Control, Shift, Alt, Super)
# Separated by '+'
key_prefix = "Control+Shift"

# Key to trigger auto-brightness
hotkey = "F12"

# Key to kill the daemon
exit_key = "Escape"
```

### Environment Variables

You can override settings using environment variables:

```bash
    RS_BRITE_KEY_PREFIX

    RS_BRITE_HOTKEY

    RS_BRITE_EXIT_KEY
```

## Usage

### Running as a Service (Recommended)

After installation, enable and start the user service so rs-brite runs automatically in the background:

```bash
systemctl --user enable --now rs-brite
```

### Running Manually

You can also run the executable directly from the terminal:

```bash
./target/release/rs-brite
```

### Controls

| Action | Default Hotkey |
|---|---|
| Auto-Adjust Brightness | Ctrl + Shift + F12 |
| Exit Application | Ctrl + Shift + Esc |

Note: These keys can be changed in the configuration file.

## How It Works

* Initialization: The app loads your config, sets up a GlobalHotKeyManager, and waits for input.

* Trigger: When the configured hotkey is pressed, the worker thread wakes up.

* Capture: It opens the default camera (index 0), discards warmup frames, and captures a single RGB frame.

* Processing:

    It calculates the raw luma (brightness) of the image.

    It calculates a "Screen Contribution" factor based on the monitor's current brightness setting to prevent the screen's own light from biasing the sensor.

    It subtracts the screen contribution from the raw luma to estimate actual ambient light.

* Adjustment: The calculated ambient luma is mapped to a brightness percentage, and the primary display is updated.

## TODO

* [ ] Advanced Screen Contribution: Include support for factoring in luma of a screen capture to adjust screen_contribution. Currently, the app estimates light coming from the monitor based solely on the hardware brightness percentage.

* [ ] Multi-Monitor Support: Allow selection of specific monitors to control rather than just the primary device.
