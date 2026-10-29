# Zed Live Server

A Rust-powered local web server and live-reload CLI intended for web projects opened in **Zed**. This repository does not contain a VS Code extension. It is currently an alpha implementation, not a v1.0.0 release.

## Included in the alpha

- Rust CLI for starting/stopping/restarting the server, status, and browser opening.
- Static HTTP serving with MIME guessing, directory indexes/listings, and configurable root, host, port, and index file.
- WebSocket reload events and an injected browser client for stylesheet replacement and full-page reload events.
- TOML configuration plus CLI overrides; default binding is loopback.
- Additional source features include HTTPS, mounts, proxying, SPA fallback, CORS, and response headers. These are included in the alpha source but are not all covered by release-level tests.

Directory index configuration is restricted to relative paths within the serving root. Index resolution checks symlinks and rejects targets outside that root. Current integration tests cover valid nested index serving, MIME type selection, traversal, symlink escape, HTML client injection, and graceful CLI stop.

This repository is a native Rust CLI, not a Zed extension, and is not installable from the Zed extension registry. Its supported integration is Zed's project-local `.zed/tasks.json`, which provides Start, Stop, Restart, Status, and Open Browser tasks. Setup, optional keybindings, and current API limits are documented in [Zed setup](docs/zed-setup.md).

## Requirements and installation

Install the stable Rust toolchain and Cargo. From a clone of this repository, install the CLI with:

```sh
cargo install --path .
live-server --version
```

For updates, run `cargo install --path . --force`. To remove the installed executable, run `cargo uninstall live-server`; this does not remove project files or `.live-server` runtime state. No extension-registry installation is available.

## Current commands

```text
live-server --version
live-server start . --port 5500
live-server start . --no-browser
live-server open .
live-server status .
live-server restart .
live-server stop .
```

The CLI defaults to loopback (`127.0.0.1`). Use `--host 0.0.0.0` only when intentionally exposing the server to the LAN. See [configuration](docs/configuration.md), [troubleshooting](docs/troubleshooting.md), and [security notes](docs/security.md) for verified behavior and remaining limitations.

To use it in every Zed project, install the CLI and add the five server tasks from `.zed/tasks.json` to Zed's global tasks with **zed: open tasks**. Then use **task: spawn** to start, stop, restart, check status, or open the browser. See [Zed setup](docs/zed-setup.md).

## Build and test

With Rust/Cargo installed:

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo build --release
cargo clippy --all-targets --all-features -- -D warnings
```

The initial audit environment did not have `cargo` on `PATH`. Subsequent validation used Rust stable on Linux. See `TODO.md` for the latest executed checks and [platform support](docs/platforms.md) for what has and has not been verified.

Development conventions and contribution validation are in [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md). GitHub Actions runs formatting, check, clippy, tests, and release builds on Linux, macOS, and Windows; those platform jobs must pass before claiming support.

## Documentation

- [Zed installation and daily workflow](docs/zed-setup.md)
- [Configuration reference](docs/configuration.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Security notes](docs/security.md)
- [Supported platforms](docs/platforms.md)
- [Changelog](CHANGELOG.md)
- [Release readiness and process](docs/release.md)

## Release status

The repository is MIT-licensed; see [`LICENSE`](LICENSE). See [`docs/release.md`](docs/release.md) and [`TODO.md`](TODO.md) for v1.0.0 readiness. This is alpha software, not a v1.0.0 release candidate. CLI process ownership, atomic duplicate-instance protection, manual Zed verification, and confirmed Windows/macOS CI remain release blockers.
