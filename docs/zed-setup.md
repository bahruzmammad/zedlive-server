# Zed setup

The native server is a CLI. The `zed-extension/` directory contains a separate Zed MCP server extension that exposes server controls in the Agent Panel; it does not register custom project commands in the Command Palette. The extension is not yet published. Zed's task templates are also supported for direct task-spawn controls.

## MCP extension setup

Install Rust and Cargo from <https://rustup.rs>, then install the Zed extension from the registry after its PR is merged, or install `zed-extension/` as a dev extension for local testing. On its first start, the extension automatically installs this project's CLI from the feature branch into Cargo's install directory. It requires internet access, a writable Cargo install directory, and Zed's permission to run the process. Later starts reuse the CLI; an unrelated npm `live-server` on PATH is not used.

If Zed reports Cargo is missing, install Rust and fully restart Zed. If automatic installation fails, check network access and write permissions for Cargo's install directory. You can install the CLI manually as a fallback:

```sh
cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --locked --force --bin live-server
```

To remove the installed CLI, run `cargo uninstall live-server`. This removes the Cargo-installed executable and does not delete project files or `.live-server` state.

## Use MCP tools in the Agent Panel

The extension exposes seven Agent Panel tools: start, stop, restart, status, open the project root, open a same-server route, and inspect effective project configuration. Project inspection validates `live-server.toml` and reports the bind address, index-file presence, and reload/watch/SPA settings without starting the server or editing files. Route preview is intended for nested HTML pages and SPA routes; external hosts and traversal segments are rejected. Tools default to the active Zed worktree; an optional project path is confined to that workspace.

For safety, the MCP server only stops a server process that it started in its current session. If the MCP session restarts, use the CLI or task workflow to control servers started by the previous session.

The automatic bootstrap checks for this project's MCP-enabled CLI using its help output, installing it only if it is missing or is a different executable. Cargo's installer output is kept off MCP standard output.

## Use tasks in every Zed project

For the tasks to appear in every frontend project, open Zed's **zed: open tasks** action and copy the five **Live Server** task entries from this repository's `.zed/tasks.json` into your global task configuration. Alternatively, copy them into a web project's `.zed/tasks.json` to keep them project-local. Open the project as a Zed worktree, then use **task: spawn** to select:

- **Live Server: Start** — runs from `ZED_WORKTREE_ROOT`, reads that root's `live-server.toml` if present, and stays attached to a visible task terminal. The server opens the configured browser by default.
- **Live Server: Stop** — asks the CLI to stop the server recorded for the worktree.
- **Live Server: Restart** — restarts that worktree's server using the current configuration.
- **Live Server: Status** — prints the process state, URL, and root.
- **Live Server: Open Browser** — opens the URL recorded for the running server.

Arguments and the task working directory use Zed's `ZED_WORKTREE_ROOT` variable. The root is passed as a distinct argument rather than interpolated into a shell command, so spaces in project paths are preserved. Project configuration is discovered at that root; a command-line root takes precedence over a TOML `root`.

The task templates require `live-server` to be on Zed's task environment `PATH`; unlike the MCP extension, tasks do not bootstrap the CLI. Install it with Cargo and restart Zed if a task cannot find it. A port conflict or invalid project configuration is reported by the CLI in the same terminal.

This is a static development server for HTML, CSS, JavaScript, and assets. It serves files directly and reloads the browser after edits; it does not compile JSX/TypeScript, run npm scripts, or replace a framework dev server such as Vite.

## Optional keybindings

Zed keybindings are user-level settings. Add bindings like these to your `keymap.json` and change the shortcuts if they conflict with your setup:

```json
[
  {
    "context": "Workspace",
    "bindings": {
      "ctrl-alt-l": ["task::Spawn", { "task_name": "Live Server: Start" }],
      "ctrl-alt-r": ["task::Spawn", { "task_name": "Live Server: Restart" }],
      "ctrl-alt-shift-l": [
        "task::Spawn",
        { "task_name": "Live Server: Stop" }
      ],
      "ctrl-alt-o": ["task::Spawn", { "task_name": "Live Server: Open Browser" }]
    }
  }
]
```

Tasks remain available through Zed's task spawn/rerun actions even without custom keybindings. For links and current behavior, see [Zed Tasks](https://zed.dev/docs/tasks) and [Zed Key Bindings](https://zed.dev/docs/key-bindings).

## Integration limits

The public extension API supports extension features such as languages, debuggers, themes, icon themes, snippets, and MCP servers. It does not provide the general project UI hooks this server would need for custom command-palette entries, status-bar UI, context menus, settings UI, or workspace open/close lifecycle management. Zed tasks are the supported alternative; task terminal output is the server's visible state and diagnostic surface.
