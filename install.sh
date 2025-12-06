#!/bin/bash
set -e

echo "Building rs-brite..."
cargo build --release

echo "Installing binary..."
sudo install -m 755 target/relese/rs-brite /usr/bin/rs-brite

echo "Installing configuration..."
sudo mkdir -p /etc/rs-brite
sudo install -m 644 -b rs-brite.toml /etc/rs-brite/config.toml

echo "Installing systemd user service..."
sudo install -m 644 packaging/rs-brite.service /usr/lib/systemd/user/rs-brite.service

echo "Reloading systemd..."
systemctl --user daemon-reload

echo "----------------------------------------------------------------"
echo "Installation complete!"
echo "To start the service, run:"
echo "  systemctl --user enable --now rs-brite"
echo "----------------------------------------------------------------"