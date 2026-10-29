# Repository guidance

## Project scope

- Keep the implementation Rust-first and focused on Zed Editor workflows.
- Do not add a VS Code extension, JavaScript/TypeScript extension infrastructure, or undocumented Zed UI workarounds.
- Use Zed's documented project-task mechanisms for project actions. Do not add an `extension.toml` unless the repository provides a supported Zed extension feature.

## Code and configuration

- Preserve existing CLI behavior and configuration keys unless a security or correctness fix requires a documented change.
- Prefer standard-library and existing dependencies; justify new dependencies in the change.
- Keep server defaults safe: bind to loopback, constrain file access to the configured root, and validate user configuration with useful errors.
- Do not swallow startup, watcher, process, or filesystem errors. Keep platform-specific code explicit and test it on supported CI runners.
- Do not add machine-specific paths, credentials, private keys, or local runtime state.
- Keep browser-side JavaScript limited to the reload client.

## Validation

For Rust changes, run:

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo build --release
```

Run focused tests first when practical. Update docs and tests to reflect verified behavior, and state clearly when editor or platform verification was unavailable.

## Security and release

- Test traversal and symlink boundaries for any filesystem path handling.
- Review WebSocket origin, CORS, process/PID, port binding, and shutdown behavior for changes that touch them.
- Do not publish, tag, or upload artifacts without explicit authorization. Do not invent or alter licensing.
- Keep generated artifacts under ignored build directories; never stage `target/`, `.live-server/`, secrets, or local configuration.
