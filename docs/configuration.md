# Configuration reference

Create `live-server.toml` in the project root. The current schema uses flat TOML keys. Omitted values use the defaults shown below.

```toml
port = 5500
host = "127.0.0.1"
open_browser = true
browser = "default"
directory_listing = true
index_file = "index.html"

reload = true
full_reload = false
wait_ms = 150
inject_client = true
watch = true
recursive = true
ignore = [".git", "target", "node_modules", ".idea", ".vscode", ".zed", ".live-server"]
```

## Settings

| Key | Default | Meaning |
|---|---:|---|
| `root` | CLI root or current directory | Serving root. A relative value is resolved from the directory containing this TOML file. The explicit CLI root overrides it. |
| `host` | `"127.0.0.1"` | IP address to bind. Keep loopback for local-only use; set `"0.0.0.0"` only to expose the server on LAN interfaces. |
| `port` | `5500` | TCP port. `0` asks the OS to choose a free port. |
| `port_mode` | `"fixed"` | `"fixed"`, `"random"`, or `"next"` / `"next-available"`. `next` searches from `port` through up to 1000 higher ports. |
| `hostname` | unset | Optional browser URL hostname. It does not change the bind address. |
| `open_browser` | `true` | Open the browser at startup. |
| `browser` | `"default"` | `"default"`, `"chrome"`, `"edge"`, `"firefox"`, or a custom `program=args` form. |
| `index_file` | `"index.html"` | Relative path used as the directory index. Absolute paths, parent traversal, Windows separators, and drive-colon paths are rejected. |
| `directory_listing` | `true` | Show a generated listing when a directory has no index file. |
| `reload` | `true` | Enable reload events and client injection. |
| `full_reload` | `false` | Use a full page reload for every watched change instead of CSS replacement where applicable. |
| `wait_ms` | `150` | Watch-event debounce interval; maximum is 60000 ms. |
| `inject_client` | `true` | Inject the reload client into HTML responses when reload is enabled. |
| `watch` | `true` | Enable filesystem watching. |
| `recursive` | `true` | Watch nested directories. |
| `ignore` | built-in defaults | Paths/patterns excluded from watching. TOML replaces the default ignore list; `--ignore` adds to the resulting list. |
| `cors` | `false` | Enable CORS response headers. |
| `allow_origins` | `["*"]` | Allowed CORS origins when CORS is enabled. |
| `allow_credentials` | `false` | Credentialed CORS; cannot be combined with a wildcard origin. |

Other supported TOML keys include `extensions`, `spa`, `spa_entry`, `allow_methods`, `allow_headers`, `https`, `certificate`, `private_key`, `proxy`, `mount`, and `headers`. See `src/config/mod.rs` for the current alpha schema; additional validation and precedence tests remain release work.

## Precedence

Defaults are applied first, then the selected TOML file, then explicit CLI options. The default configuration file is `live-server.toml` under the initially selected CLI root/current directory; `--config` selects another file. Relative TOML `root`, mount, and certificate paths are based on the configuration file's directory. An explicitly supplied positional root or `--root` wins over TOML `root`; `--host`, `--port`, `--index`, and other supplied options override their TOML counterparts. `--ignore` adds patterns rather than replacing the TOML list.

For automatic ports, use `--port 0` / `port = 0` for an OS-assigned port, or `port_mode = "random"`; `port_mode = "next"` searches sequentially. The CLI's fixed default remains 5500. CLI switches are intentionally one-way for several booleans (for example `--no-browser`, `--no-reload`); use TOML for explicit `true` and `false` settings.
