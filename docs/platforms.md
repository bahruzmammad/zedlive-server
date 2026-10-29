# Platform support

The project is written in Rust and uses cross-platform Tokio, Axum, filesystem-watcher, and TLS dependencies. Source includes platform-specific browser launch and process-control branches, but those branches have not all been built and exercised.

## Validation status

- **Linux x86_64:** formatting, all-target check/tests, release build, and clippy passed in the current development environment.
- **Windows:** CI is configured to run formatting, all-target check/tests, release build, and clippy. The Windows job has not yet been observed passing; CLI lifecycle, browser launch, and Zed-task behavior still need manual verification.
- **macOS:** CI is configured for compile/test coverage, but the job has not yet been observed passing and manual Zed-task behavior is unverified.

Do not treat unverified platforms as stable-release supported. Check successful CI runs and perform a manual Zed workflow check on each release platform before declaring support.
