use crate::{
    browser,
    cli::RunOptions,
    control::{self, ServerState},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] =
    &["2024-11-05", "2025-03-26", "2025-06-18", PROTOCOL_VERSION];

struct McpServer {
    workspace_root: PathBuf,
    children: HashMap<PathBuf, Child>,
}

impl McpServer {
    fn new() -> Result<Self, String> {
        let workspace_root = std::env::current_dir()
            .and_then(std::fs::canonicalize)
            .map_err(|error| format!("cannot resolve the Zed workspace root: {error}"))?;
        Ok(Self {
            workspace_root,
            children: HashMap::new(),
        })
    }

    fn handle(&mut self, request: Value) -> Option<Value> {
        let id = request.get("id").cloned();
        let method = request.get("method").and_then(Value::as_str);
        let Some(method) = method else {
            return id.map(|id| rpc_error(id, -32600, "Invalid Request"));
        };

        match method {
            "notifications/initialized" | "notifications/cancelled" => None,
            "initialize" => Some(rpc_result(
                id?,
                json!({
                    "protocolVersion": negotiated_protocol_version(&request),
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": {
                        "name": "live-server",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            )),
            "ping" => Some(rpc_result(id?, json!({}))),
            "tools/list" => Some(rpc_result(id?, json!({ "tools": tools() }))),
            "tools/call" => {
                let id = id?;
                let params = request.get("params").unwrap_or(&Value::Null);
                let name = params.get("name").and_then(Value::as_str);
                let arguments = params.get("arguments").unwrap_or(&Value::Null);
                match name
                    .ok_or_else(|| "tools/call requires a tool name".to_owned())
                    .and_then(|name| self.call_tool(name, arguments))
                {
                    Ok(text) => Some(rpc_result(
                        id,
                        json!({
                            "content": [{ "type": "text", "text": text }],
                            "isError": false
                        }),
                    )),
                    Err(error) => Some(rpc_result(
                        id,
                        json!({
                            "content": [{ "type": "text", "text": error }],
                            "isError": true
                        }),
                    )),
                }
            }
            _ => id.map(|id| rpc_error(id, -32601, "Method not found")),
        }
    }

    fn call_tool(&mut self, name: &str, arguments: &Value) -> Result<String, String> {
        if !matches!(
            name,
            "live_server_start"
                | "live_server_stop"
                | "live_server_restart"
                | "live_server_status"
                | "live_server_open_browser"
                | "live_server_open_page"
                | "live_server_inspect_project"
        ) {
            return Err(format!("unknown Live Server tool: {name}"));
        }
        let root = resolve_root(arguments, &self.workspace_root)?;
        match name {
            "live_server_start" => self.start(&root),
            "live_server_stop" => self.stop(&root),
            "live_server_restart" => {
                if self.children.contains_key(&root) {
                    self.stop(&root)?;
                } else if let Ok(state) = control::read_state(&root) {
                    if control::process_exists(state.pid) {
                        return Err("This MCP session did not start the running server, so it will not stop it. Start it with live_server_start first.".into());
                    }
                    control::clear_state(&root);
                }
                self.start(&root)
            }
            "live_server_status" => status(&root),
            "live_server_inspect_project" => inspect_project(&root),
            "live_server_open_browser" => {
                control::open(&root, None)?;
                let state = control::read_state(&root)?;
                Ok(format!("Opened {}", state.url))
            }
            "live_server_open_page" => {
                let state = control::read_state(&root)?;
                if !control::process_exists(state.pid) {
                    return Err("Live Server process is not running".into());
                }
                let path = match arguments.get("path") {
                    None => "/",
                    Some(Value::String(path)) => path,
                    Some(_) => return Err("the path argument must be a route string".into()),
                };
                let url = page_url(&state.url, path)?;
                let config = RunOptions {
                    root: Some(root.clone()),
                    ..RunOptions::default()
                }
                .into_config()?;
                browser::open_url(&config.browser, config.browser_command.as_deref(), &url)?;
                Ok(format!("Opened {url}"))
            }
            _ => unreachable!("tool name was checked above"),
        }
    }

    fn start(&mut self, root: &Path) -> Result<String, String> {
        if self.children.contains_key(root) {
            return Err(
                "This MCP session already started the server for this project. Check its status or restart it."
                    .into(),
            );
        }

        let executable = std::env::current_exe()
            .map_err(|error| format!("cannot locate the live-server executable: {error}"))?;
        let mut child = Command::new(executable)
            .arg("start")
            .arg(root)
            .arg("--no-browser")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("cannot start Live Server: {error}"))?;

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("cannot check Live Server startup: {error}"))?
            {
                clear_state_if_owned(root, child.id());
                return Err(format!(
                    "Live Server exited during startup with {status}. Check project configuration and port availability."
                ));
            }

            if let Some(state) = read_running_state(root) {
                let pid = child.id();
                if state.pid != pid {
                    let _ = child.kill();
                    let _ = child.wait();
                    clear_state_if_owned(root, pid);
                    return Err(
                        "server state does not belong to the process started by this MCP session"
                            .into(),
                    );
                }
                let url = state.url.clone();
                self.children.insert(root.to_path_buf(), child);
                return Ok(format!("Live Server started at {url}"));
            }

            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                clear_state_if_owned(root, child.id());
                return Err("Live Server did not become ready within 5 seconds".into());
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn stop(&mut self, root: &Path) -> Result<String, String> {
        let Some(child) = self.children.get_mut(root) else {
            return Err(
                "This MCP session did not start the server, so it will not stop it. Start it with live_server_start first."
                    .into(),
            );
        };

        let pid = child.id();
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("cannot check Live Server process: {error}"))?
        {
            clear_state_if_owned(root, pid);
            self.children.remove(root);
            return Ok(format!("Live Server had already stopped ({status})."));
        }

        let state = control::read_state(root)
            .map_err(|_| "the MCP-managed server is no longer running".to_owned())?;
        if state.pid != pid {
            return Err("server state PID does not match the process started by this MCP session; refusing to stop it".into());
        }

        control::stop(root)?;
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if child
                .try_wait()
                .map_err(|error| format!("cannot verify Live Server shutdown: {error}"))?
                .is_some()
            {
                break;
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "Live Server process {pid} did not exit after the stop request"
                ));
            }
            thread::sleep(Duration::from_millis(50));
        }

        self.children.remove(root);
        Ok("Live Server stopped.".into())
    }

    fn stop_all(&mut self) -> Result<(), String> {
        let roots: Vec<_> = self.children.keys().cloned().collect();
        for root in roots {
            self.stop(&root)?;
        }
        Ok(())
    }
}

pub fn run_stdio() -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let mut server = McpServer::new()?;

    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("cannot read MCP input: {error}"))?;
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => server.handle(request),
            Err(error) => Some(rpc_error(
                Value::Null,
                -32700,
                &format!("Parse error: {error}"),
            )),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut output, &response)
                .map_err(|error| format!("cannot write MCP response: {error}"))?;
            output
                .write_all(b"\n")
                .and_then(|_| output.flush())
                .map_err(|error| format!("cannot flush MCP response: {error}"))?;
        }
    }

    server.stop_all()?;
    Ok(())
}

fn resolve_root(arguments: &Value, workspace_root: &Path) -> Result<PathBuf, String> {
    let root = match arguments.get("root") {
        None => workspace_root.to_path_buf(),
        Some(Value::String(root)) => PathBuf::from(root),
        Some(_) => return Err("the root argument must be a path string".into()),
    };
    let root = std::fs::canonicalize(&root)
        .map_err(|error| format!("cannot access project root {}: {error}", root.display()))?;
    if !root.starts_with(workspace_root) {
        return Err(format!(
            "project root {} is outside the active Zed workspace {}",
            root.display(),
            workspace_root.display()
        ));
    }
    if !root.is_dir() {
        return Err(format!(
            "project root is not a directory: {}",
            root.display()
        ));
    }
    Ok(root)
}

fn read_running_state(root: &Path) -> Option<ServerState> {
    let state = control::read_state(root).ok()?;
    (control::process_exists(state.pid)).then_some(state)
}

fn clear_state_if_owned(root: &Path, pid: u32) {
    if control::read_state(root).is_ok_and(|state| state.pid == pid) {
        control::clear_state(root);
    }
}

fn status(root: &Path) -> Result<String, String> {
    match control::read_state(root) {
        Ok(state) if control::process_exists(state.pid) => Ok(format!(
            "Live Server is running at {} (PID {}, root {}).",
            state.url, state.pid, state.root
        )),
        Ok(state) => Ok(format!(
            "Live Server state is stale for PID {}. It was not removed.",
            state.pid
        )),
        Err(_) => Ok(format!("Live Server is stopped for {}.", root.display())),
    }
}

fn inspect_project(root: &Path) -> Result<String, String> {
    let options = RunOptions {
        root: Some(root.to_path_buf()),
        ..RunOptions::default()
    };
    let config = options.into_config()?;
    config.validate().map_err(|error| error.to_string())?;
    let configured_index = root.join(&config.index_file);
    let index_status = if configured_index.is_file() {
        format!("present ({})", config.index_file)
    } else {
        format!("missing ({})", config.index_file)
    };
    let config_file = root.join("live-server.toml");
    let config_status = if config_file.is_file() {
        "loaded live-server.toml"
    } else {
        "using defaults (no live-server.toml)"
    };

    Ok(format!(
        "Live Server project inspection\n\
         Root: {}\n\
         Configuration: {config_status}\n\
         Bind address: {}:{}\n\
         Index file: {index_status}\n\
         Directory listing: {}\n\
         File watching: {}\n\
         Browser reload: {} (CSS hot swap: {})\n\
         SPA fallback: {}\n\
         Browser on start: {}",
        root.display(),
        config.host,
        config.port,
        enabled(config.directory_listing),
        enabled(config.watch.enabled),
        enabled(config.reload.enabled && config.reload.inject_client),
        enabled(config.reload.enabled && !config.reload.full_reload),
        enabled(config.spa.enabled),
        enabled(config.open_browser),
    ))
}

fn enabled(value: bool) -> &'static str {
    if value { "enabled" } else { "disabled" }
}

fn page_url(base_url: &str, path: &str) -> Result<String, String> {
    let decoded_path = percent_encoding::percent_decode_str(path)
        .decode_utf8()
        .map_err(|_| "page path must be valid UTF-8".to_owned())?;
    let path_without_query = decoded_path.split(['?', '#']).next().unwrap_or_default();
    if path.starts_with("//")
        || decoded_path.starts_with("//")
        || path.contains("://")
        || decoded_path.contains('\\')
        || decoded_path.chars().any(char::is_control)
        || path_without_query
            .split('/')
            .any(|segment| matches!(segment, "." | ".."))
    {
        return Err("page path must be a same-server route without traversal segments".into());
    }

    Ok(format!("{}{}", base_url, path.trim_start_matches('/')))
}

fn tools() -> Vec<Value> {
    [
        (
            "live_server_start",
            "Start the static Live Server using the project's live-server.toml or safe defaults. Serves files directly, watches frontend assets, and injects the reload client into HTML. Does not run npm scripts or compile frameworks. The browser is not opened automatically by this tool; call live_server_open_browser afterward if wanted.",
        ),
        (
            "live_server_stop",
            "Stop the server only when this MCP session started and still owns its child process. It verifies the recorded PID before requesting shutdown and refuses to stop servers started by another CLI, task, or MCP session.",
        ),
        (
            "live_server_restart",
            "Restart a server owned by this MCP session, applying the project's current live-server.toml. If another session owns the running server, this tool refuses to stop it; use that session or the CLI instead.",
        ),
        (
            "live_server_status",
            "Report whether this project's Live Server is running, including its URL and PID. Stale state is reported without deleting files or signaling a process.",
        ),
        (
            "live_server_open_browser",
            "Open the root URL of this project's running Live Server in the system browser. Fails if no live process is running.",
        ),
        (
            "live_server_open_page",
            "Open a specific same-server route in the system browser, useful for previewing a nested HTML page or SPA route. The route cannot specify another host or include dot-segment traversal. Requires a running server.",
        ),
        (
            "live_server_inspect_project",
            "Validate the project's live-server.toml and summarize effective settings before starting: bind address, index file presence, directory listing, watch/reload, SPA fallback, and browser behavior. This is a read-only diagnostic; it does not start the server or modify project files.",
        ),
    ]
    .into_iter()
    .map(|(name, description)| {
        let mut properties = serde_json::Map::new();
        properties.insert(
            "root".into(),
            json!({
                "type": "string",
                "description": "Optional directory inside the active Zed worktree. Omit to use the worktree root."
            }),
        );
        if name == "live_server_open_page" {
            properties.insert(
                "path".into(),
                json!({
                    "type": "string",
                    "description": "Optional same-server route such as /products/item.html or /docs/. Defaults to /."
                }),
            );
        }
        json!({
            "name": name,
            "description": description,
            "inputSchema": {
                "type": "object",
                "properties": properties,
                "additionalProperties": false
            }
        })
    })
    .collect()
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn negotiated_protocol_version(request: &Value) -> &str {
    request
        .pointer("/params/protocolVersion")
        .and_then(Value::as_str)
        .filter(|version| SUPPORTED_PROTOCOL_VERSIONS.contains(version))
        .unwrap_or(PROTOCOL_VERSION)
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_and_lists_live_server_tools() {
        let mut server = McpServer::new().unwrap();
        let initialized = server
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": PROTOCOL_VERSION }
            }))
            .unwrap();
        assert_eq!(initialized["result"]["protocolVersion"], PROTOCOL_VERSION);

        let listed = server
            .handle(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
            .unwrap();
        let tools = listed["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 7);
        assert_eq!(listed["result"]["tools"][0]["name"], "live_server_start");
        assert!(
            tools
                .iter()
                .any(|tool| tool["name"] == "live_server_inspect_project")
        );
        assert!(
            tools
                .iter()
                .any(|tool| tool["name"] == "live_server_open_page")
        );
        assert!(
            tools
                .iter()
                .all(|tool| !tool["description"].as_str().unwrap().is_empty())
        );
    }

    #[test]
    fn initialization_negotiates_supported_protocol_versions() {
        let mut server = McpServer::new().unwrap();
        let response = server
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": "2024-11-05" }
            }))
            .unwrap();
        assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    }

    #[test]
    fn notifications_do_not_write_responses() {
        let mut server = McpServer::new().unwrap();
        assert!(
            server
                .handle(json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/initialized"
                }))
                .is_none()
        );
    }

    #[test]
    fn invalid_project_root_is_reported_as_tool_error() {
        let mut server = McpServer::new().unwrap();
        let response = server
            .handle(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "live_server_status",
                    "arguments": { "root": "/a/path/that/does/not/exist" }
                }
            }))
            .unwrap();
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn project_root_cannot_escape_the_active_workspace() {
        let workspace_root = std::fs::canonicalize(".").unwrap();
        assert_eq!(
            resolve_root(&json!({}), &workspace_root).unwrap(),
            workspace_root
        );
        assert!(resolve_root(&json!({ "root": ".." }), &workspace_root).is_err());
    }

    #[test]
    fn page_routes_cannot_escape_the_server_or_open_external_hosts() {
        let base = "http://127.0.0.1:5500/";
        assert_eq!(
            page_url(base, "/products/index.html").unwrap(),
            "http://127.0.0.1:5500/products/index.html"
        );
        assert!(page_url(base, "//example.com").is_err());
        assert!(page_url(base, "https://example.com").is_err());
        assert!(page_url(base, "/../../secret.html").is_err());
        assert!(page_url(base, "/%2e%2e/secret.html").is_err());
    }

    #[test]
    fn project_inspection_reports_effective_settings_and_missing_index() {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let temp = std::env::temp_dir().join(format!(
            "live-server-inspect-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(
            temp.join("live-server.toml"),
            "port = 5512\nreload = false\n",
        )
        .unwrap();

        let report = inspect_project(&temp).unwrap();
        assert!(report.contains("loaded live-server.toml"));
        assert!(report.contains("127.0.0.1:5512"));
        assert!(report.contains("missing (index.html)"));
        assert!(report.contains("Browser reload: disabled"));

        std::fs::remove_dir_all(temp).unwrap();
    }
}
