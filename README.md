# Onset

A lightweight GTK4/libadwaita application for managing XDG autostart entries on Linux.

## Features

-   **View autostart entries** from your user directory
-   **Create new entries** from installed applications or custom commands
-   **Edit entries** — modify name, command, comment, and startup delay
-   **Enable/Disable** entries without deleting them
-   **Startup delay** — optionally delay application startup
-   **XDG compliant** — follows freedesktop.org specifications

## Installation

### Arch Linux (Quick Install)

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/xPathin/onset/main/install_arch.sh)
```

### Arch Linux (AUR)

-   Coming soon

### From Source

```bash
# Dependencies (Arch)
sudo pacman -S gtk4 libadwaita rust

# Build
cargo build --release

# Install
sudo install -Dm755 target/release/onset /usr/bin/onset
sudo install -Dm644 data/com.github.xPathin.onset.desktop /usr/share/applications/com.github.xPathin.onset.desktop
sudo install -Dm644 data/icons/hicolor/scalable/apps/com.github.xPathin.onset.svg /usr/share/icons/hicolor/scalable/apps/com.github.xPathin.onset.svg
```

## Usage

Launch `onset` from your application menu or terminal.

-   **Toggle switch** — Enable/disable an entry
-   **Edit button** — Modify entry settings
-   **Delete button** — Remove the entry
-   **+ button** — Add a new autostart entry
-   **Refresh button** — Reload entries from disk

## Dependencies

-   GTK 4.12+
-   libadwaita 1.4+

## License

[MIT](LICENSE)
