use zed_extension_api::{
    self as zed, Command, ContextServerConfiguration, ContextServerId, Project, Result,
};

struct LiveServerExtension;

impl zed::Extension for LiveServerExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Command> {
        let output = zed::process::Command::new("live-server")
            .arg("--help")
            .output()
            .map_err(|error| {
                format!(
                    "The MCP-enabled Live Server CLI is not available. Install it with `cargo install --git https://github.com/bahruzmammad/zedlive-server --branch live-server-mcp-extension --force`, ensure Cargo's bin directory is first on Zed's PATH, and restart Zed. Details: {error}"
                )
            })?;
        if output.status != Some(0) {
            return Err(
                "The installed Live Server CLI could not show its help. Reinstall the MCP-enabled CLI and restart Zed."
                    .into(),
            );
        }
        let help = String::from_utf8_lossy(&output.stdout);
        if !help.contains("Run the Zed Live Server MCP server over stdio") {
            return Err(
                "A different `live-server` executable is first on Zed's PATH. Install this project's MCP-enabled CLI and put Cargo's bin directory before other `live-server` commands."
                    .into(),
            );
        }

        Ok(Command {
            command: "live-server".to_owned(),
            args: vec!["mcp".to_owned()],
            env: Vec::new(),
        })
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
