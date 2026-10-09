# Rust Live Server

A native Rust HTTP server and browser live-reload tool for local HTML, CSS, JavaScript, JSON, and static assets.

## Build and test

```powershell
cargo fmt --all
cargo test --all-targets
cargo check --all-targets
cargo build --release
```

## CLI

```powershell
live-server --version
live-server start . --port 5500
live-server start . --no-browser
live-server open .
live-server status .
live-server restart .
live-server stop .
```

The root argument can be any project directory. The default bind address is `127.0.0.1`. Use `--host 0.0.0.0` only if other devices on your LAN need access.

## Live reload

The Rust server serves the virtual `__live_server/live-reload.js` endpoint, injects the client into HTML responses, and broadcasts file change events over WebSocket. CSS changes can use stylesheet replacement; other supported content changes trigger a full page reload.

## Release package

Run `FINALIZE-RUST-LIVE-SERVER-V1.ps1` from outside the repository and pass this directory as `-ProjectRoot`. The script tests the code, builds the Windows x64 release binary, performs HTTP smoke tests, installs the CLI per-user, and creates a versioned `Rust-Live-Server-<version>-windows-x64.zip` package.

## Zed integration limitation

This repository currently contains the native server/CLI. It is not yet a complete Zed extension with custom `Live Server: Start/Stop/Restart/Toggle` Command Palette commands, a custom status-bar widget, or a file-tree context-menu action. The current public Zed Extension API does not expose general registration APIs for those UI elements. No `.zed/tasks.json` workaround is installed by this release script.