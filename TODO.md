# v1.0.0 audit and acceptance plan

The initial audit environment lacked Cargo. Rust stable was subsequently installed through the existing `mise` tool manager; focused/full tests and a Linux release build have run for the current changes. Worktree changes from the audit/P2 tasks are intentionally preserved and uncommitted pending the final review.

## Release priority summary

- **P0 — release blockers:** process-state PID ownership/reuse and atomic duplicate-start protection; no manual Zed workflow validation is possible in this environment.
- **P1 — required before stable:** full reload/WebSocket/watcher behavior tests, CLI lifecycle and configuration precedence tests, broader path/security review including filesystem TOCTOU, and Windows/macOS build/runtime verification.
- **P2 — implemented, not yet validated in Zed:** project-local task templates and installation/use docs; same-origin WebSocket checks and wildcard-credential CORS rejection are covered by unit tests.
- **P3 — publication follow-up:** the requested custom command-palette commands/buttons are not supported by Zed's public extension API, so this CLI cannot be published as an extension that provides that UX. Keep the task-based integration; align to v1.0 only after release criteria pass, and consider packaged artifacts later.

## Phase 1 — Audit (completed)

- [x] Inventory the existing Rust modules, tests, fixtures, browser reload client, TOML examples, CLI, and Zed task configuration.
- [x] Record known extension API boundaries and supported Zed task/keybinding alternatives.
- [x] Record the implementation plan and testable release criteria below.

### Verified source inventory

- `src/cli` parses `start`/`run`, `stop`, `restart`, `open`, and `status`, plus server options. `src/main.rs` dispatches them. Existing unit tests cover selected argument parsing cases.
- `src/config` defines flat TOML options and CLI override application. The checked-in sample binds to `127.0.0.1`; CLI help describes `0.0.0.0` as LAN access.
- `src/http`, `src/security`, and `src/server` implement static file handling, MIME guessing, directory listings/index files, response headers, path checks, listener selection, and server startup. `src/https`, `src/proxy`, and `src/spa` provide additional feature modules.
- `src/watcher`, `src/reload`, `src/websocket`, and `web/live-reload.js` implement filesystem events, reload messages, WebSocket clients, CSS stylesheet replacement, and reconnect attempts.
- `src/control` stores per-root state under `.live-server/` and implements status/open/stop operations. `tests/integration.rs` currently covers health, HTML/client injection, and a missing-file response; the fixtures include static HTML, CSS, JS, JSON, SVG, and a proxy upstream.
- The unsupported root `extension.toml` was removed so Zed does not try to compile the native CLI as a WebAssembly dev extension. `.zed/tasks.json` and setup/configuration/troubleshooting/security/platform docs are present.
- Cargo/source remain at `0.0.1-alpha`; do not change to `1.0.0` until release criteria pass. The maintainer confirmed MIT and the matching `LICENSE` file is checked in.

### Zed API findings

Official documentation describes extension capabilities for languages, debuggers, themes, icon themes, snippets, and MCP servers. The extension API is not a general project command/plugin API; do not claim custom Live Server command-palette commands, status-bar widgets, file-tree context actions, or a settings UI.

Project-local `.zed/tasks.json` is the supported practical integration: Zed task templates run shell commands with `ZED_WORKTREE_ROOT`, appear in the task spawn/rerun workflow, and can be bound using the user's keymap and `task::Spawn`. The documented automatic task hook is `create_worktree`, not workspace lifecycle management. Zed documents `process:exec` and `download_file` capabilities for extensions, but that does not provide a generic extension-managed native CLI installer/launcher for this product.

Sources checked: [Zed extension development](https://zed.dev/docs/extensions/developing-extensions), [extension capabilities](https://zed.dev/docs/extensions/capabilities), [tasks](https://zed.dev/docs/tasks), [keybindings](https://zed.dev/docs/key-bindings), and [extension publishing prerequisites](https://zed.dev/docs/extensions/publishing/prerequisites).

## Remaining work and acceptance criteria

### Phase 2 — Rust server core

- [x] Preserve the current Rust-first CLI and document only behavior verified by tests.
- [x] Restrict configured directory indexes to relative paths and resolve them through root/symlink containment checks; tests cover nested index serving, MIME selection, traversal, and symlink escape.
- [ ] Test HTTP MIME types, index-file selection, directory behavior, GET/HEAD, errors, cache headers, and malformed/request traversal paths.
- [ ] Review symlink and filesystem race behavior; ensure traversal/out-of-root paths cannot be served.
- [x] Reject wildcard CORS with credentials and constrain browser WebSocket origins to the request's scheme/authority.
- [x] Handle Unix SIGTERM in the server shutdown future so CLI stop uses graceful HTTP shutdown.
- [ ] Test reload injection for existing client tags, HTML without closing tags, and non-UTF-8 HTML; verify CSS replacement and non-CSS reload messages in a browser client.
- [ ] Test watcher ignore matching, burst debouncing, event filtering, multiple WebSocket clients, reconnect, and orderly shutdown.
- [x] Fail server startup with a useful error if the file watcher cannot initialize or watch a configured root.
- [x] Test that CSS, JavaScript, and HTML edits generate the expected reload events.

### Phase 3 — CLI lifecycle and configuration

- [x] Linux CLI smoke exercised start, health, HTML injection, status, open dispatch (via a no-op browser executable), restart, and stop with a workspace path containing spaces.
- [ ] Verify `--version`, help, project-root detection, config-file discovery, CLI-over-TOML precedence, and config-relative paths.
- [ ] Test fixed/random/next-available ports, invalid config, occupied ports, and clear diagnostics.
- [ ] Test start/stop/restart/status/open across supported platforms, duplicate starts, stale state, and graceful shutdown. Ensure state cannot accidentally identify or terminate an unrelated process.
- [ ] Decide/document foreground task lifecycle and ensure Zed task workflows can start, stop, restart, inspect status, and open the current browser URL without manually searching for a PID.
- [ ] Keep loopback as the default; require an explicit host choice for LAN access. Document browser and ignore/reload precedence.
- [ ] Validate configuration values and unknown/malformed options with actionable errors; cover precedence and edge cases with tests.

### Phase 4 — Zed-native project workflow

- [x] Provide project-local `.zed/tasks.json` tasks for start, stop, restart, status, and open. JSON syntax and CLI command shapes are checked, but a manual Zed workflow cannot be run here.
- [x] Document CLI installation/discovery and global task setup for use across frontend projects; do not add a VS Code extension or claim extension UI.
- [x] Explain task spawn/rerun, optional keybindings, and task terminal diagnostics.
- [x] Disclose unsupported custom command-palette registrations, status-bar UI, file-tree actions, settings UI, and general workspace lifecycle hooks.

### Phase 5 — Security, reliability, and release documentation

- [ ] Review origin validation for WebSockets, LAN exposure, process state/PID safety, path traversal/symlinks, HTML injection, resource cleanup, and error propagation.
- [x] Add Zed installation/usage, configuration reference, troubleshooting, security notes, and supported-platform guidance; Linux is the only validated platform so far.
- [x] Align Cargo and source versions at `0.0.1-alpha` using Cargo package metadata and add the maintainer-confirmed MIT license text.
- [x] Add an unreleased changelog and reproducible build/install/uninstall instructions; no external publication without authorization.

### Phase 6 — Release gate

- [x] `cargo fmt --all -- --check`
- [x] `cargo check --all-targets`
- [x] `cargo test --all-targets`
- [x] `cargo build --release`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Review the final diff and confirm every documented feature and Zed integration claim is supported by tests or official documentation.
