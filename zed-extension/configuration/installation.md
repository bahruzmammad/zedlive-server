The MCP extension automatically builds and installs this project's CLI the first time it starts, then reuses that installation on later starts. This requires Rust/Cargo to be installed and available to Zed, internet access, and permission to run the process when Zed prompts. The extension installs into Cargo's normal binary directory and launches the absolute path, so another `live-server` command on `PATH` cannot shadow it. Installing the extension does not currently install Zed's global tasks and is not a Rust-free setup.

If Zed reports that Cargo is missing, install Rust from <https://rustup.rs>, fully restart Zed, and start the MCP server again. If Cargo installation fails, check network access and that your Cargo install directory is writable. Installation output is sent to the process error stream; MCP protocol output remains on standard output.

To install the CLI manually or use the Zed task templates, run `cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --locked --force --bin live-server`. Tasks require the resulting Cargo binary directory to be on Zed's `PATH`.

The MCP server runs with the active Zed worktree as its working directory. Tools default to that workspace root; an optional project path must resolve inside that worktree. Paths outside the active workspace are rejected.
