# Changelog

## Unreleased — alpha development

- Added project-local Zed task templates for start, stop, restart, status, and opening the browser.
- Added Zed setup, configuration, troubleshooting, and security documentation.
- Restricted configured directory index paths and checked index symlinks against the serving root.
- Added same-origin validation for browser WebSocket connections and rejected wildcard CORS origins combined with credentials.
- Added Unix SIGTERM handling so CLI stop requests trigger the HTTP server's graceful-shutdown path.
- Added a Linux/macOS/Windows CI matrix for formatting, checks, tests, and release builds.
- Added regression tests for directory index selection, MIME types, traversal, symlink escape, index validation, CORS validation, WebSocket origin matching, and graceful CLI stop.
- Removed the misleading extension manifest so Zed does not attempt to compile the native CLI as a dev extension.
- Made watcher initialization failures abort startup instead of silently disabling reload; added checks for watcher events on HTML, CSS, and JavaScript edits.
- Kept the Zed task list focused on server controls and documented global tasks for use across frontend projects.
- Added release readiness notes covering Zed's supported workflows, extension marketplace limits, and v1.0.0 gates.
- Added the maintainer-confirmed MIT license and derived CLI version/help text from Cargo package metadata.
- Added an MCP stdio server with seven Agent Panel tools for server lifecycle, project configuration diagnostics, and same-server route preview.
- Added a Zed MCP extension that bootstraps the CLI with Cargo on first start, keeps installer output off MCP stdout, and protects session-owned stop behavior.
- Added the MIT license to the nested Zed extension package for registry validation.

This is not a v1.0.0 release. The latest MCP extension and CLI checks pass on Linux, macOS, and Windows; the Marketplace PR is open and awaits maintainer review. End-to-end Zed verification, CLI process ownership, and atomic duplicate-instance protection remain open.
