use zed_extension_api::{
    self as zed, Command, ContextServerConfiguration, ContextServerId, Project, Result,
};

const UNIX_LAUNCHER: &str = r#"
set -eu
install_root="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}"
server="$install_root/bin/live-server"

is_mcp_server() {
    [ -x "$server" ] || return 1
    "$server" --help 2>&1 | grep -Fq 'Run the Zed Live Server MCP server over stdio'
}

if ! is_mcp_server; then
    if ! command -v cargo >/dev/null 2>&1; then
        printf '%s\n' 'Live Server needs Rust and Cargo. Install Rust from https://rustup.rs, restart Zed, and try again.' >&2
        exit 127
    fi
    cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --locked --force --bin live-server 1>&2
fi

if ! is_mcp_server; then
    printf '%s\n' 'Cargo finished, but the installed Live Server CLI does not provide the Zed MCP server.' >&2
    exit 1
fi

exec "$server" mcp
"#;

const WINDOWS_LAUNCHER: &str = r#"
$ErrorActionPreference = 'Stop'
if ($env:CARGO_INSTALL_ROOT) {
    $installRoot = $env:CARGO_INSTALL_ROOT
} elseif ($env:CARGO_HOME) {
    $installRoot = $env:CARGO_HOME
} else {
    $installRoot = Join-Path $env:USERPROFILE '.cargo'
}
$server = Join-Path $installRoot 'bin\live-server.exe'
$marker = 'Run the Zed Live Server MCP server over stdio'

function Test-McpServer {
    if (-not (Test-Path -LiteralPath $server -PathType Leaf)) { return $false }
    $help = (& $server --help 2>&1 | Out-String)
    return ($LASTEXITCODE -eq 0 -and $help.Contains($marker))
}

if (-not (Test-McpServer)) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        [Console]::Error.WriteLine('Live Server needs Rust and Cargo. Install Rust from https://rustup.rs, restart Zed, and try again.')
        exit 127
    }
    & cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --locked --force --bin live-server 1>&2
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not (Test-McpServer)) {
    [Console]::Error.WriteLine('Cargo finished, but the installed Live Server CLI does not provide the Zed MCP server.')
    exit 1
}

& $server mcp
exit $LASTEXITCODE
"#;

struct LiveServerExtension;

fn context_server_command() -> Command {
    let (os, _) = zed::current_platform();
    match os {
        zed::Os::Windows => Command {
            command: "powershell.exe".to_owned(),
            args: vec![
                "-NoLogo".to_owned(),
                "-NoProfile".to_owned(),
                "-NonInteractive".to_owned(),
                "-Command".to_owned(),
                WINDOWS_LAUNCHER.to_owned(),
            ],
            env: Vec::new(),
        },
        zed::Os::Mac | zed::Os::Linux => Command {
            command: "sh".to_owned(),
            args: vec!["-c".to_owned(), UNIX_LAUNCHER.to_owned()],
            env: Vec::new(),
        },
    }
}

impl zed::Extension for LiveServerExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Command> {
        Ok(context_server_command())
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        Ok(Some(ContextServerConfiguration {
            installation_instructions: include_str!("../configuration/installation.md").to_owned(),
            settings_schema: r#"{"type":"object","properties":{},"additionalProperties":false}"#
                .to_owned(),
            default_settings: "{}".to_owned(),
        }))
    }
}

zed::register_extension!(LiveServerExtension);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launchers_install_only_when_the_mcp_cli_is_missing() {
        let marker = "Run the Zed Live Server MCP server over stdio";
        for launcher in [UNIX_LAUNCHER, WINDOWS_LAUNCHER] {
            assert!(launcher.contains("cargo install"));
            assert!(launcher.contains("--force"));
            assert!(launcher.contains(marker));
            assert!(launcher.contains(" mcp"));
        }
        assert!(UNIX_LAUNCHER.contains("CARGO_INSTALL_ROOT"));
        assert!(WINDOWS_LAUNCHER.contains("CARGO_INSTALL_ROOT"));
    }

    #[test]
    fn unix_launcher_sends_installer_output_away_from_mcp_stdout() {
        assert!(UNIX_LAUNCHER.contains("1>&2"));
    }
}
