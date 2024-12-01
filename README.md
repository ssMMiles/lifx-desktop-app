# LIFX Desktop App

Avoid needing the LIFX app, this is intended to at some point be a desktop app used can control lights from your taskbar.
Currently it just runs a basic web UI on port 3000 exposing the controls.

Supports onboarding new lights, switching on/off, changing brightness and colour.

## Running

1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone this repo and the underlying LIFX library
```bash
git clone https://github.com/ssMMiles/lifx-lan
git clone https://github.com/ssMMiles/lifx-desktop-app
```

3. Run the app
```bash
cd lifx-desktop-app
cargo run
```

Done! You should now be able to access the web UI at `http://localhost:3000`

![Web UI](/screenshot.png "Web UI")