# Zed setup

The native server is a CLI. The `zed-extension/` directory contains a separate Zed MCP server extension that exposes server controls in the Agent Panel; it does not register custom project commands in the Command Palette. The extension is not yet published. Zed's task templates are also supported for direct task-spawn controls.

## Install the CLI

Install Rust and Cargo, clone this repository, then run `cargo install --path .` from its root. The binary is named `live-server`. Verify that it is available in the shell with `live-server --version`.

If Zed reports that `live-server` cannot be found, install it with Cargo as above, make sure Cargo's binary directory is on the environment used to launch Zed, and restart Zed. The project does not hard-code a machine-specific binary path or download/install a native binary through an extension.

To remove the installed CLI, run `cargo uninstall live-server`. This removes the Cargo-installed executable and does not delete project files or `.live-server` state.

## Use MCP tools in the Agent Panel

Install the CLI and ensure `live-server` is on the environment used to launch Zed. Then install the published **Live Server** extension when available, or install `zed-extension/` as a dev extension for local testing. The extension starts `live-server mcp` and exposes seven Agent Panel tools: start, stop, restart, status, open the project root, open a same-server route, and inspect effective project configuration. Project inspection validates `live-server.toml` and reports the bind address, index-file presence, and reload/watch/SPA settings without starting the server or editing files. Route preview is intended for nested HTML pages and SPA routes; external hosts and traversal segments are rejected. Tools default to the active Zed worktree; an optional project path is confined to that workspace.

For safety, the MCP server only stops a server process that it started in its current session. If the MCP session restarts, use the CLI or task workflow to control servers started by the previous session.

The extension checks for the CLI by running `live-server --version` through Zed's extension process API. Allow that command when prompted; if the executable is missing, install it and restart Zed.

## Use tasks in every Zed project

For the tasks to appear in every frontend project, open Zed's **zed: open tasks** action and copy the five **Live Server** task entries from this repository's `.zed/tasks.json` into your global task configuration. Alternatively, copy them into a web project's `.zed/tasks.json` to keep them project-local. Open the project as a Zed worktree, then use **task: spawn** to select:

- **Live Server: Start** — runs from `ZED_WORKTREE_ROOT`, reads that root's `live-server.toml` if present, and stays attached to a visible task terminal. The server opens the configured browser by default.
- **Live Server: Stop** — asks the CLI to stop the server recorded for the worktree.
- **Live Server: Restart** — restarts that worktree's server using the current configuration.
- **Live Server: Status** — prints the process state, URL, and root.
- **Live Server: Open Browser** — opens the URL recorded for the running server.

Arguments and the task working directory use Zed's `ZED_WORKTREE_ROOT` variable. The root is passed as a distinct argument rather than interpolated into a shell command, so spaces in project paths are preserved. Project configuration is discovered at that root; a command-line root takes precedence over a TOML `root`.

The task templates require `live-server` to be on Zed's task environment `PATH`. If a task cannot start, its terminal output reports the executable error; install the CLI and restart Zed so its task environment picks up the updated `PATH`. A port conflict or invalid project configuration is reported by the CLI in the same terminal.

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
