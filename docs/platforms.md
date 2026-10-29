# Platform support

The native project is written in Rust and uses cross-platform Tokio, Axum, filesystem-watcher, and TLS dependencies. The Zed extension is a separate Rust crate compiled to `wasm32-wasip2`.

## Validation status

- **Linux:** formatting, all-target check/tests, release build, and Clippy passed locally and in GitHub Actions before the MCP extension was added. The new extension needs CI and manual Zed verification.
- **Windows:** native formatting, all-target check/tests, release build, and Clippy passed in GitHub Actions before the MCP extension was added. The new extension needs CI; CLI lifecycle, browser launch, and Zed integration still need manual verification.
- **macOS:** native formatting, all-target check/tests, release build, and Clippy passed in GitHub Actions before the MCP extension was added. The new extension needs CI and manual Zed verification.

Do not treat the extension as verified or marketplace-ready until the WASM CI check passes and it has been manually tested in Zed. Native platform CI does not replace editor/browser verification.
