# Release checklist

The project is alpha. Do not tag `v1.0.0` or submit the Zed extension until the remaining verification and lifecycle work below is complete.

## Completed

- [x] Static Rust server and browser live-reload client for frontend assets.
- [x] Zed task templates for start, stop, restart, status, and opening the browser.
- [x] MCP stdio server with Agent Panel tools for the same operations.
- [x] Zed MCP extension in `zed-extension/`, using the separately installed native CLI.
- [x] MCP stop tool only stops processes started by its own current session, matching server-state PID before stopping.
- [x] MIT license and public setup, configuration, troubleshooting, security, and platform docs.
- [x] Native Rust format, check, tests, release build, and Clippy passed locally before the MCP extension addition.
- [x] Native Linux, macOS, and Windows CI passed on the commit before the MCP extension addition.

## Required before marketplace submission

- [ ] Run the new `wasm32-wasip2` extension build on CI and fix any platform/build failures.
- [ ] Manually install `zed-extension/` in Zed and verify server setup, tool discovery, start/status/open/restart/stop, and browser reload.
- [ ] Confirm extension user experience and installation on Windows and Linux; test macOS if claiming support there.
- [ ] Address Zed's MCP extension deprecation notice by publishing the MCP server through the official MCP Registry or confirming the current accepted migration path.
- [ ] Resolve overlap with the existing [`live-server` extension](https://zed.dev/extensions/live-server) with its maintainers; contribute upstream if they consider the feature overlapping.
- [ ] After manual verification, fork `zed-industries/extensions`, add exactly this extension as a submodule using HTTPS, add the matching `extensions.toml` version/path, sort, and submit one PR.

## Required before v1.0.0

- [ ] Bind persisted server state to verifiable process identity so stale or forged state cannot stop an unrelated process.
- [ ] Prevent concurrent starts for the same project root with an atomic lock/state claim.
- [ ] Add lifecycle regression tests for stale state, PID reuse/ownership, duplicate starts, and stop timeout/error reporting.
- [ ] Expand HTTP/reload/watcher/configuration tests, including browser verification of CSS replacement and full-page reload.
- [ ] Pass formatting, check, Clippy, tests, and release build on Linux, Windows, and macOS for the exact release commit.
- [ ] Resolve all P0/P1 items, update the changelog with verified behavior, and obtain maintainer approval before tagging or publishing.

## Zed integration boundaries

- The MCP extension exposes tools in the Agent Panel; it does not register custom project commands in the Command Palette, status bar, or file-tree menus.
- The MCP extension depends on the separately installed `live-server` executable and requires a project path for each tool call.
- MCP stops only a server started in the same MCP session. Use CLI/Zed tasks for processes started in another session.
- Zed task templates remain the supported direct editor-task workflow; this is not a VS Code extension and does not compile JSX/TypeScript.
