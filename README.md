# Zed Live Server

[![Rust CI](https://github.com/bahruzmammad/zedlive-server/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bahruzmammad/zedlive-server/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A local static web server and live-reload tool for frontend projects in **Zed**. This repository provides a native Rust CLI, Zed task templates, and a Zed MCP extension that exposes Live Server controls in the Agent Panel.

> **Status: alpha.** Linux, macOS, and Windows CI passed on the current `main` before the MCP extension was added. The new extension still needs build/CI and manual Zed verification before marketplace submission. MCP extensions are planned for deprecation by Zed in favor of the official MCP Registry; see [release readiness](docs/release.md).

## Features

- Serves HTML, CSS, JavaScript, and static assets from a local project.
- Reloads the browser after watched frontend files change; CSS updates use stylesheet replacement.
- Starts, stops, restarts, checks status, and opens the browser using the CLI, Zed tasks, or the MCP tools in the Agent Panel.
- Defaults to loopback (`127.0.0.1`) and supports TOML configuration and CLI overrides.
- Includes additional source features such as HTTPS, mounts, proxying, SPA fallback, CORS, and response headers; see [configuration](docs/configuration.md).

This serves files directly. It does not compile TypeScript or JSX, run npm scripts, or replace Vite, webpack, or another framework development server.

## Install the CLI

Install the stable Rust toolchain, then install the native server:

```sh
cargo install --git https://github.com/bahruzmammad/zedlive-server
live-server --version
```

For local development, clone this repository and run `cargo install --path .`. Update a Git-installed CLI with `cargo install --git https://github.com/bahruzmammad/zedlive-server --force`. Uninstall it with `cargo uninstall live-server`. There are no prebuilt release binaries yet.

## Use in Zed

### Agent Panel MCP tools

The Zed extension is in [`zed-extension/`](zed-extension/). It runs `live-server mcp`; therefore install the CLI first and make sure Cargo's binary directory is on the environment used to launch Zed. The extension exposes `live_server_start`, `live_server_stop`, `live_server_restart`, `live_server_status`, and `live_server_open_browser` to the Agent Panel. Tools default to the active Zed workspace and reject project paths outside that workspace.

The extension is not yet published to the Zed registry. For local development, install it as a dev extension by selecting `zed-extension/` in Zed's **Install Dev Extension** flow. The extension requires manual testing in Zed before registry submission.

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
