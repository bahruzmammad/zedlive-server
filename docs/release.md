# Release readiness

## What this project can provide in Zed

This project is a native Rust static web server and live-reload client. On Linux, Windows, and macOS, the intended workflow is to install the `live-server` CLI, add its task templates to Zed's global tasks once, and use **task: spawn** to start, stop, restart, inspect, or open the site. Optional keybindings can spawn those tasks.

It serves HTML, CSS, JavaScript, and static assets directly. Editing a watched HTML, CSS, or JavaScript file sends a reload event to the browser. It does not compile TypeScript, JSX, or framework components, and does not replace Vite, webpack, or another framework development server.

## What the Zed extension marketplace cannot provide for this project

Zed extensions must provide a supported extension feature such as language support, a theme, snippets, a debugger, or an MCP server. The public extension API does not expose general project lifecycle commands, custom Command Palette entries, toolbar/status-bar controls, or project task registration. A native CLI repository without an extension feature is not a valid marketplace extension; adding a metadata-only `extension.toml` makes Zed try to compile it as a WebAssembly extension and is not a fix.

An MCP extension could expose tools in the Agent Panel, but it would not provide the requested Live Server Start/Stop Command Palette workflow and is planned for deprecation in favor of the MCP Registry. It is intentionally not part of this product.

The supported Zed workflow and limitations are documented in [Zed setup](zed-setup.md). Do not submit this CLI to the Zed extension registry as though it were a Zed extension.

## v1.0.0 release gates

Do not tag or announce `v1.0.0` until all of these are true:

- The GitHub Actions workflow passes on Linux, Windows, and macOS.
- The server lifecycle cannot stop an unrelated process through stale or forged `.live-server` state, and simultaneous starts for one project are handled safely.
- A manual Zed task workflow and browser reload are verified on Linux and Windows.
- The release checklist in `TODO.md` has no unresolved P0/P1 items.
- `Cargo.toml`'s MIT declaration matches the checked-in `LICENSE`.

The current branch is a workflow/reliability update, not a v1 release. CI results and manual editor checks must be reviewed before promoting it.

## Release procedure after the gates pass

1. Update `Cargo.toml` to the approved version. `Cargo.lock`, the CLI version output, and help text should derive from the package version rather than duplicate it.
2. Update `CHANGELOG.md` with only verified changes and mark the release date.
3. Run `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets`, `cargo build --release`, and `cargo clippy --all-targets --all-features -- -D warnings`.
4. Confirm the Linux, Windows, and macOS CI jobs are green and review the exact commit diff.
5. Commit the release changes, create the `v1.0.0` tag, and push the commit and tag only with maintainer authorization.

This procedure releases the CLI and task-based Zed workflow. It does not submit an extension to the Zed marketplace; that would require a separate supported extension feature and a separate review.
