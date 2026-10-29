Install the Live Server command before enabling this MCP server:

```sh
cargo install --git https://github.com/bahruzmammad/zedlive-server
```

Make sure Cargo's binary directory is available on the environment used to launch Zed, then restart Zed. The extension starts `live-server mcp`; it does not download or bundle a native executable.

When Zed asks for the extension's process execution capability, allow `live-server --version`. The extension uses this check to verify the CLI before starting the MCP server.

The MCP server runs with the active Zed worktree as its working directory. Tools default to that workspace root; an optional project path must resolve inside that worktree. Paths outside the active workspace are rejected.
