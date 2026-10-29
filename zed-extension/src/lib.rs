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
            .arg("--version")
            .output()
            .map_err(|error| {
                format!(
                    "Live Server CLI is not available. Install it with `cargo install --git https://github.com/bahruzmammad/zedlive-server` and restart Zed. Details: {error}"
                )
            })?;
        if output.status != Some(0) {
            return Err(
                "The installed Live Server CLI could not report its version. Reinstall it and restart Zed."
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
