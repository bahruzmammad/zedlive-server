# Troubleshooting

## Zed says `live-server` cannot be found

Install Rust/Cargo and run `cargo install --path .` from this repository. Confirm `live-server --version` in a shell, ensure Cargo's binary directory is visible to the environment that launched Zed, and restart Zed. See [Zed setup](zed-setup.md).

## Port is already in use

Stop the existing worktree instance with the **Live Server: Stop** task or `live-server stop .`. Choose another fixed port, set `port = 0` / `port_mode = "random"` for an OS-assigned port, or use `port_mode = "next"`. The server does not silently switch ports in fixed mode.

## The browser does not open

Check `open_browser = true`, run **Live Server: Status**, and use **Live Server: Open Browser**. Browser launching depends on an installed system browser and platform launcher. You can start with `--no-browser` and copy the URL printed in the Zed task terminal.

## Reload does not occur

Check that `reload` and `watch` are enabled, that the changed file's extension is watched, and that it is not ignored by `ignore`. Keep the start task terminal visible for watcher errors. CSS changes can replace matching stylesheet links; other event kinds use a full-page reload. There is no browser-automation test suite yet.

## Invalid configuration

Check the CLI's printed error and the [configuration reference](configuration.md). Common causes include a missing/non-directory root, invalid host IP, invalid `index_file`, incomplete HTTPS certificate/key configuration, invalid CORS wildcard-plus-credentials, and missing mount directories.

## No Zed Live Server buttons or status widget

This project uses Zed's supported task templates. The public extension API does not provide custom command-palette commands, status-bar widgets, or file-tree context actions for this use case. Use **task: spawn** and the project tasks instead. **Live Server: Start Current Folder** serves the directory of the active file; **Live Server: Start** serves the project root.

## Report a reproducible issue

Include operating system, Rust version, `live-server --version`, sanitized configuration, exact CLI/task output, and reproduction steps. Do not include private project files, credentials, or TLS private keys.
