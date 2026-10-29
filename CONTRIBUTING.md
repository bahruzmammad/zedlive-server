# Contributing

Contributions should preserve the Rust CLI/server architecture and target Zed Editor through documented mechanisms such as project-local tasks. Do not add VS Code extension infrastructure or claim unsupported Zed UI APIs.

## Before submitting

1. Install the stable Rust toolchain.
2. Read `AGENTS.md`, `README.md`, and the relevant configuration/security documentation.
3. Keep changes focused; add regression tests for new or corrected behavior.
4. Run from the repository root:

   ```sh
   cargo fmt --all -- --check
   cargo check --all-targets
   cargo test --all-targets
   cargo build --release
   ```

5. If Zed is available, verify the project tasks in Zed. Otherwise, state that only the task file/schema and CLI behavior were checked.
6. Review `git diff` and exclude build output, `.live-server/`, credentials, TLS keys, and personal configuration.

## Reporting issues

Include operating system, Rust version, CLI version, sanitized TOML, exact output, and reproduction steps. Never attach credentials, private keys, or unredacted project files.

## Releases

Do not create releases, publish to an extension/package registry, or upload binaries without maintainer authorization. The current manifest is metadata only, not a registry-ready Zed extension. Do not change the declared MIT license or create a license file without maintainer confirmation.
