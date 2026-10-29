# Release readiness

## What this project can provide in Zed

This project is a native Rust static web server and live-reload client. The Zed MCP extension in `zed-extension/` exposes server tools in the Agent Panel. The CLI and `.zed/tasks.json` also provide start, stop, restart, status, and browser controls through Zed's supported task-spawn workflow.

It serves HTML, CSS, JavaScript, and static assets directly. Editing a watched HTML, CSS, or JavaScript file sends a reload event to the browser. It does not compile TypeScript, JSX, or framework components, and does not replace Vite, webpack, or another framework development server.

## Zed extension and marketplace status

The repository now includes a real MCP extension at `zed-extension/`. It starts the separately installed native `live-server mcp` process and exposes Live Server start, stop, restart, status, and browser tools in the Agent Panel. It does not register custom project commands in the Command Palette, toolbar/status-bar controls, or task registration.

The extension has not yet been manually tested in Zed or submitted to the registry. Zed plans to deprecate MCP server extensions in favor of the official MCP Registry, and the publishing prerequisites ask MCP server authors to publish there as well. Address that migration/distribution requirement before submitting the extension. Do not describe this integration as Command Palette task controls.

See [Zed setup](zed-setup.md) for installing the CLI and using the extension or tasks.

## v1.0.0 release gates

Do not tag or announce `v1.0.0` until all of these are true:

- The GitHub Actions workflow passes on Linux, Windows, and macOS, including the Zed extension WASM build.
- The server lifecycle cannot stop an unrelated process through stale or forged `.live-server` state, and simultaneous starts for one project are handled safely.
- The extension is manually installed and tested in Zed, including its MCP tools and browser reload.
- The release checklist in `TODO.md` has no unresolved P0/P1 items.
- `Cargo.toml`'s MIT declaration matches the checked-in `LICENSE`.

The project remains alpha. Verify the extension's WASM build in CI, manually test the extension in Zed, and review CI results before promotion or marketplace submission.

## Release procedure after the gates pass

1. Update `Cargo.toml` to the approved version. `Cargo.lock`, the CLI version output, and help text should derive from the package version rather than duplicate it.
2. Update `CHANGELOG.md` with only verified changes and mark the release date.
3. Run `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets`, `cargo build --release`, and `cargo clippy --all-targets --all-features -- -D warnings`.
4. Confirm the Linux, Windows, and macOS CI jobs are green and review the exact commit diff.
5. Commit the release changes, create the `v1.0.0` tag, and push the commit and tag only with maintainer authorization.

This procedure releases the CLI and Zed integrations. Submitting the MCP extension requires its own manual Zed verification, an MCP Registry distribution plan, and a separate registry PR.
