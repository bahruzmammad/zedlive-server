Install the Live Server command before enabling this MCP server:

```sh
cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --force
```

On Fish, put Cargo's binary directory before other commands with the same name:

```fish
fish_add_path --prepend $HOME/.cargo/bin
type -a live-server
live-server --version
```

`$HOME/.cargo/bin/live-server` must appear before any Node/npm `live-server`; this project CLI reports version `0.0.1-alpha`. Fully quit Zed and launch it from that Fish session (for example, `zed .`) so it inherits the corrected `PATH`. The extension starts `live-server mcp`; it does not download or bundle a native executable.

When Zed asks for the extension's process execution capability, allow `live-server --help`. The extension verifies that the resolved command includes this project's MCP server before starting it, and rejects unrelated commands also named `live-server`.

The MCP server runs with the active Zed worktree as its working directory. Tools default to that workspace root; an optional project path must resolve inside that worktree. Paths outside the active workspace are rejected.
