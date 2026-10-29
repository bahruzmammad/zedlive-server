# Zed setup

The native server is a CLI. The `zed-extension/` directory contains a separate Zed MCP server extension that exposes server controls in the Agent Panel; it does not register custom project commands in the Command Palette. The extension is not yet published. Zed's task templates are also supported for direct task-spawn controls.

## MCP extension setup

Install Rust and Cargo from <https://rustup.rs>, then install the Zed extension from the registry after its PR is merged, or install `zed-extension/` as a dev extension for local testing. On its first start, the extension automatically builds and installs this project's CLI from the feature branch into Cargo's install directory. This still requires Rust/Cargo, internet access, a writable Cargo install directory, and Zed's permission to run the process. Later starts reuse the CLI; an unrelated npm `live-server` on PATH is not used. **The extension alone is not yet a Rust-free, AI-free installation:** prebuilt native binaries and a supported no-AI task integration are not available.

If Zed reports Cargo is missing, install Rust and fully restart Zed. If automatic installation fails, check network access and write permissions for Cargo's install directory. You can install the CLI manually as a fallback:

```sh
cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --locked --force --bin live-server
```

To remove the installed CLI, run `cargo uninstall live-server`. This removes the Cargo-installed executable and does not delete project files or `.live-server` state.

## Use MCP tools in the Agent Panel

The extension exposes seven Agent Panel tools: start, stop, restart, status, open the project root, open a same-server route, and inspect effective project configuration. Project inspection validates `live-server.toml` and reports the bind address, index-file presence, and reload/watch/SPA settings without starting the server or editing files. Route preview is intended for nested HTML pages and SPA routes; external hosts and traversal segments are rejected. Tools default to the active Zed worktree; an optional project path is confined to that workspace.

For safety, the MCP server only stops a server process that it started in its current session. If the MCP session restarts, use the CLI or task workflow to control servers started by the previous session.

The automatic bootstrap checks for this project's MCP-enabled CLI using its help output, installing it only if it is missing or is a different executable. Cargo's installer output is kept off MCP standard output.

## Recommended: use tasks without AI

You do not need the Agent Panel or AI for this workflow. The current task workflow does require a one-time Rust/Cargo CLI install; Marketplace extension installation does not make the CLI available to tasks. To set it up:

1. Install Rust/Cargo from <https://rustup.rs>, then install this CLI:
   ```sh
   cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --locked --force --bin live-server
   ```
   This is a one-time machine setup. Restart Zed after installation if it cannot find `live-server`.
2. In Zed, run **zed: open tasks**. Copy the ten **Live Server** entries from this repository's [`.zed/tasks.json`](../.zed/tasks.json) into the global tasks file. Alternatively, copy them to a project's `.zed/tasks.json` to keep them project-local.
3. Open a frontend project folder in Zed. Run **task: spawn**, choose **Live Server: Start**, and start working. The task serves the active worktree folder and opens the configured browser.

The task file provides two matching sets of five actions: **Start**, **Stop**, **Restart**, **Status**, and **Open Browser**, for either the project root or the directory containing the active file. Start opens the browser automatically. Use the `Current Folder` variants together so stop/restart/status/open target the same server root. Project tasks use `ZED_WORKTREE_ROOT`; current-folder tasks use Zed's `ZED_DIRNAME`. Paths are passed as distinct arguments so spaces are preserved. A `live-server.toml` in the selected root is used when present.

Tasks require `live-server` on Zed's task `PATH`. The MCP extension has its own first-run CLI bootstrap, but tasks do not invoke it; for this AI-free workflow, use the one-time Cargo install above and ensure Cargo's `bin` directory is on Zed's task `PATH`. A port conflict or invalid project configuration is reported in the task terminal.

Once started, the server watches saved HTML, CSS, JavaScript, and asset changes: CSS is updated live, while HTML/JavaScript changes reload the browser. Zed autosave is optional. To save edits automatically after a short pause, add this to Zed's user-level `settings.json`:

```json
{
  "autosave": {
    "after_delay": {
      "milliseconds": 1000
    }
  }
}
```

This is a static development server: it serves files directly, does not compile JSX/TypeScript or run npm scripts, and does not replace a framework server such as Vite.

## Optional keybindings

Zed keybindings are user-level settings. Add bindings like these to your `keymap.json` and change the shortcuts if they conflict with your setup:

```json
[
  {
    "context": "Workspace",
    "bindings": {
      "ctrl-alt-l": ["task::Spawn", { "task_name": "Live Server: Start" }],
      "ctrl-alt-f": [
        "task::Spawn",
        { "task_name": "Live Server: Start Current Folder" }
      ],
      "ctrl-alt-shift-f": [
        "task::Spawn",
        { "task_name": "Live Server: Stop Current Folder" }
      ],
      "ctrl-alt-r": ["task::Spawn", { "task_name": "Live Server: Restart" }],
      "ctrl-alt-shift-r": [
        "task::Spawn",
        { "task_name": "Live Server: Restart Current Folder" }
      ],
      "ctrl-alt-shift-l": [
        "task::Spawn",
        { "task_name": "Live Server: Stop" }
      ],
      "ctrl-alt-o": ["task::Spawn", { "task_name": "Live Server: Open Browser" }],
      "ctrl-alt-shift-o": [
        "task::Spawn",
        { "task_name": "Live Server: Open Current Folder in Browser" }
      ]
    }
  }
]
```

Tasks remain available through Zed's task spawn/rerun actions even without custom keybindings. Shortcuts are optional and user-level; change them if they conflict with your keymap. For links and current behavior, see [Zed Tasks](https://zed.dev/docs/tasks) and [Zed Key Bindings](https://zed.dev/docs/key-bindings).

## Integration limits

The public extension API does not provide the general project UI hooks this server would need for custom Command Palette entries, status-bar UI, context menus, extension-provided keybindings, or automatic start when an HTML file opens. Zed also does not install this project's global task templates when the extension is installed. Therefore, the current no-AI workflow requires a one-time CLI install and task setup; it cannot truthfully promise a VS Code-style Go Live button, no-Rust setup, or automatic startup on file open. These are tracked in [issue #4](https://github.com/bahruzmammad/zedlive-server/issues/4). If Zed adds supported APIs or we publish a cross-platform binary installer, revisit the task and setup docs together.
