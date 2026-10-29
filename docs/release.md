# Release readiness

## What this project can provide in Zed

This project is a native Rust static web server and live-reload client. The Zed MCP extension in `zed-extension/` exposes server tools in the Agent Panel. The CLI and `.zed/tasks.json` also provide start, stop, restart, status, and browser controls through Zed's supported task-spawn workflow.

It serves HTML, CSS, JavaScript, and static assets directly. Editing a watched HTML, CSS, or JavaScript file sends a reload event to the browser. It does not compile TypeScript, JSX, or framework components, and does not replace Vite, webpack, or another framework development server.

## Zed extension and marketplace status

The repository now includes a real MCP extension at `zed-extension/`. On first start, it uses Cargo to install the native CLI if needed, then launches its MCP process; it exposes seven Agent Panel tools for server lifecycle controls, project configuration diagnostics, and same-server route preview. It does not register custom project commands in the Command Palette, toolbar/status-bar controls, or task registration.

The Marketplace submission is open as [`zed-industries/extensions#8174`](https://github.com/zed-industries/extensions/pull/8174). Registry packaging, sorting, Danger, and CLA checks pass; maintainer review and merge are pending. The repository already contains a [`live-server` extension](https://zed.dev/extensions/live-server), so maintainers must assess whether this Agent Panel interface and its project-diagnostic/route-preview tools are distinct enough to merit another listing. End-to-end lifecycle and browser-reload testing in Zed is still outstanding. Zed plans to deprecate MCP server extensions in favor of the official MCP Registry, so distribution may need migration. Do not describe this integration as Command Palette task controls or as already published.

See [Zed setup](zed-setup.md) for installing the CLI and using the extension or tasks.

The AI-free onboarding and Zed task workflow follow-up is tracked in [issue #4](https://github.com/bahruzmammad/zedlive-server/issues/4). Keep `.zed/tasks.json` as the source of truth for the five task actions, and update it alongside the setup docs if the workflow changes.

## v1.0.0 release gates

Do not tag or announce `v1.0.0` until all of these are true:

- The GitHub Actions workflow passes on Linux, Windows, and macOS, including the Zed extension WASM build.
- The server lifecycle cannot stop an unrelated process through stale or forged `.live-server` state, and simultaneous starts for one project are handled safely.
- The extension is manually installed and tested in Zed, including its MCP tools and browser reload.
- The release checklist in `TODO.md` has no unresolved P0/P1 items.
- `Cargo.toml`'s MIT declaration matches the checked-in `LICENSE`.

The project remains alpha. The latest pushed extension build and Linux/macOS/Windows CI pass; manually test the extension in Zed and wait for Marketplace maintainer review before claiming publication or promoting a release.

## Release procedure after the gates pass

1. Update `Cargo.toml` to the approved version. `Cargo.lock`, the CLI version output, and help text should derive from the package version rather than duplicate it.
2. Update `CHANGELOG.md` with only verified changes and mark the release date.
3. Run `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets`, `cargo build --release`, and `cargo clippy --all-targets --all-features -- -D warnings`.
4. Confirm the Linux, Windows, and macOS CI jobs are green and review the exact commit diff.
5. Commit the release changes, create the `v1.0.0` tag, and push the commit and tag only with maintainer authorization.

This procedure releases the CLI and Zed integrations. The MCP extension has a separate Marketplace PR; publication depends on maintainer review/merge, manual Zed verification, and addressing Zed's MCP Registry migration guidance.
