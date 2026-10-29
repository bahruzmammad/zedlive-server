# Zed Live Server

[![Rust CI](https://github.com/bahruzmammad/zedlive-server/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bahruzmammad/zedlive-server/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A local static web server and live-reload tool for frontend projects in **Zed**. This repository provides a native Rust CLI, Zed task templates, and a Zed MCP extension that exposes Live Server controls in the Agent Panel.

> **Status: alpha.** The MCP extension and CLI pass Linux, macOS, and Windows CI on the current feature branch. Marketplace submission is open as [zed-industries/extensions#8174](https://github.com/zed-industries/extensions/pull/8174), awaiting maintainer review. Manual end-to-end testing in Zed is still outstanding. MCP extensions are planned for deprecation by Zed in favor of the official MCP Registry; see [release readiness](docs/release.md).

## Features

- Serves HTML, CSS, JavaScript, and static assets from a local project.
- Reloads the browser after watched frontend files change; CSS updates use stylesheet replacement.
- Starts, stops, restarts, checks status, and opens the browser using the CLI, Zed tasks, or the MCP tools in the Agent Panel.
- Adds Agent Panel project diagnostics for configuration/index readiness and route-level browser preview for nested pages and SPA routes.
- Defaults to loopback (`127.0.0.1`) and supports TOML configuration and CLI overrides.
- Includes additional source features such as HTTPS, mounts, proxying, SPA fallback, CORS, and response headers; see [configuration](docs/configuration.md).

This serves files directly. It does not compile TypeScript or JSX, run npm scripts, or replace Vite, webpack, or another framework development server.

## Install the CLI

Install Rust and Cargo from <https://rustup.rs>. The MCP extension automatically installs the native CLI on first start. To use the CLI directly or the Zed task templates, install it manually:

```sh
cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --force
live-server --version
```

The Zed MCP extension requires Cargo, network access for its first start, a writable Cargo install directory, and Zed's process permission. It installs the CLI from the `live-server-mcp-extension` branch into Cargo's normal install directory, then starts that absolute path so an unrelated `live-server` on PATH cannot shadow it. The CLI/tasks require the `live-server-mcp-extension` branch until the change is merged into `main`. Update with the same install command. Uninstall with `cargo uninstall live-server`. There are no prebuilt release binaries yet.

## Use in Zed

### Agent Panel MCP tools

The Zed extension is in [`zed-extension/`](zed-extension/). On first start it automatically installs and launches this project's CLI. Its seven Agent Panel tools are:

- `live_server_start`, `live_server_stop`, `live_server_restart`, and `live_server_status` for the server lifecycle.
- `live_server_open_browser` for the project root and `live_server_open_page` for a same-server route such as `/products/item.html` or an SPA path.
- `live_server_inspect_project` to validate configuration and report the effective bind address, index-file presence, watching/reload, and SPA settings without starting or changing anything.

All tools default to the active Zed workspace and reject project paths outside that workspace. Route preview rejects external hosts and dot-segment traversal. Start serves static files; it does not invoke framework build scripts.

The Zed registry already lists a [`live-server` extension](https://zed.dev/extensions/live-server). This MCP extension is a separate Agent Panel tool interface with project diagnostics and route preview; registry maintainers will assess whether its functionality is sufficiently distinct. Its registry PR is open and pending review. For local development, install it as a dev extension by selecting `zed-extension/` in Zed's **Install Dev Extension** flow. Manual Zed lifecycle and browser-reload testing is still needed.

**MCP lifecycle note:** the MCP server only stops processes it started in its current session. This avoids terminating an unrelated process through stale project state. It stops its owned servers when the MCP session shuts down normally. Servers started with a Zed task or another CLI session can still be controlled using the CLI/tasks.

Zed currently plans to deprecate MCP server extensions in favor of the official MCP Registry. The project's current MCP extension and publishing status are documented in [release readiness](docs/release.md).

### Zed tasks

Use the five task entries in [`.zed/tasks.json`](.zed/tasks.json) from **task: spawn**, or copy them to Zed's global tasks using **zed: open tasks**. The tasks support start, stop, restart, status, and opening the browser without the Agent Panel. See [Zed setup](docs/zed-setup.md).

## CLI quick start

From a frontend project directory:

```sh
live-server start . --port 5500
live-server status .
live-server open .
live-server restart .
live-server stop .
```

Add `--no-browser` to start without opening a browser. The default bind address is loopback; only use `--host 0.0.0.0` when you intentionally want LAN access. See [configuration](docs/configuration.md).

## Build and test

Requires Rust stable and Cargo:

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo build --release
cargo clippy --all-targets --all-features -- -D warnings
```

The Zed extension is a separate crate and compiles to WebAssembly:

```sh
rustup target add wasm32-wasip2
cargo check --manifest-path zed-extension/Cargo.toml --target wasm32-wasip2
```

GitHub Actions runs native checks on Linux, macOS, and Windows. CI does not replace manual Zed extension and browser-reload verification.

## Documentation

- [Zed setup and workflows](docs/zed-setup.md)
- [Configuration](docs/configuration.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Security notes](docs/security.md)
- [Platform support](docs/platforms.md)
- [Changelog](CHANGELOG.md)
- [Release readiness](docs/release.md)
- [Contributing](CONTRIBUTING.md)

This project is MIT-licensed; see [`LICENSE`](LICENSE).
