.PHONY: all check_root

# Default target
all: check_root build

build:
	cargo build --release

check_root:
	@echo "Checking for root privileges..."
	@if [ "$(shell id -u)" -eq 0 ]; then \
		echo "Error: run build as a non-root user. Exiting."; \
		exit 1; \
	fi
	@echo "User is not root. Continuing."

# Removed 'build' from dependencies here so sudo doesn't trigger cargo
install:
	@install -m 755 target/release/rs-brite /usr/bin/rs-brite
	@mkdir -p /etc/rs-brite
	@install -m 644 rs-brite.toml /etc/rs-brite/config.toml
	@install -m 644 packaging/rs-brite.service /usr/lib/systemd/user/rs-brite.service
	@echo "---------------------------------------------------"
	@echo "Installation complete."
	@echo "Run 'systemctl --user daemon-reload' as your user to refresh."
	@echo "To start the service, run:"
	@echo "  systemctl --user enable --now rs-brite"
	@echo "----------------------------------------------------------------"

uninstall:
	rm -f /usr/bin/rs-brite
	rm -f /usr/lib/systemd/user/rs-brite.service
	rm -rf /etc/rs-brite