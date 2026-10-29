use crate::config::{
    Browser, ServerConfig, parse_browser, parse_header, parse_mount, parse_port_mode, parse_proxy,
};
use std::{net::IpAddr, path::PathBuf, str::FromStr};

#[derive(Debug)]
pub enum CliCommand {
    Help,
    Version,
    Mcp,
    Start(Box<RunOptions>),
    Stop(ControlOptions),
    Restart(Box<RunOptions>),
    Open(ControlOptions),
    Status(ControlOptions),
}

#[derive(Clone, Debug, Default)]
pub struct ControlOptions {
    pub root: PathBuf,
    pub browser: Option<Browser>,
}

#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    pub root: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub port: Option<u16>,
    pub host: Option<IpAddr>,
    pub preferred_hostname: Option<String>,
    pub port_mode: Option<crate::config::PortMode>,
    pub no_browser: bool,
    pub browser: Option<Browser>,
    pub browser_command: Option<String>,
    pub index_file: Option<String>,
    pub directory_listing: Option<bool>,
    pub reload: Option<bool>,
    pub full_reload: bool,
    pub wait_ms: Option<u64>,
    pub inject_client: Option<bool>,
    pub watch: Option<bool>,
    pub recursive: Option<bool>,
    pub spa: Option<bool>,
    pub spa_entry: Option<String>,
    pub cors: Option<bool>,
    pub allow_origins: Option<Vec<String>>,
    pub allow_methods: Option<Vec<String>>,
    pub allow_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub https_cert: Option<PathBuf>,
    pub https_key: Option<PathBuf>,
    pub proxy: Vec<crate::config::ProxyRule>,
    pub mounts: Vec<String>,
    pub headers: Vec<crate::config::HeaderRule>,
    pub ignores: Vec<String>,
}

impl RunOptions {
    pub fn into_config(self) -> Result<ServerConfig, String> {
        let mut config = ServerConfig::default();

        let root = self
            .root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        config.root = root.clone();

        let config_path = self
            .config_path
            .clone()
            .unwrap_or_else(|| root.join("live-server.toml"));

        if config_path.is_file() {
            config
                .load_file(&config_path)
                .map_err(|error| error.to_string())?;
        }

        // Explicit command-line options override configuration-file values.
        if let Some(value) = self.root {
            config.root = value;
        }
        if let Some(value) = self.port {
            config.port = value;
        }
        if let Some(value) = self.host {
            config.host = value;
        }
        if let Some(value) = self.preferred_hostname {
            config.preferred_hostname = Some(value);
        }
        if let Some(value) = self.port_mode {
            config.port_mode = value;
        }
        if self.no_browser {
            config.open_browser = false;
        }
        if let Some(value) = self.browser {
            config.browser = value;
        }
        if let Some(value) = self.browser_command {
            config.browser_command = Some(value);
        }
        if let Some(value) = self.index_file {
            config.index_file = value;
        }
        if let Some(value) = self.directory_listing {
            config.directory_listing = value;
        }
        if let Some(value) = self.reload {
            config.reload.enabled = value;
        }
        if self.full_reload {
            config.reload.enabled = true;
            config.reload.full_reload = true;
        }
        if let Some(value) = self.wait_ms {
            config.reload.wait_ms = value;
        }
        if let Some(value) = self.inject_client {
            config.reload.inject_client = value;
        }
        if let Some(value) = self.watch {
            config.watch.enabled = value;
        }
        if let Some(value) = self.recursive {
            config.watch.recursive = value;
        }
        if !self.ignores.is_empty() {
            config.watch.ignore.extend(self.ignores);
        }
        if let Some(value) = self.spa {
            config.spa.enabled = value;
        }
        if let Some(value) = self.spa_entry {
            config.spa.entry_file = value;
        }
        if let Some(value) = self.cors {
            config.cors.enabled = value;
        }
        if let Some(value) = self.allow_origins {
            config.cors.allow_origins = value;
        }
        if let Some(value) = self.allow_methods {
            config.cors.allow_methods = value;
        }
        if let Some(value) = self.allow_headers {
            config.cors.allow_headers = value;
        }
        if let Some(value) = self.allow_credentials {
            config.cors.allow_credentials = value;
        }

        if self.https_cert.is_some() || self.https_key.is_some() {
            config.https.enabled = true;
            config.https.certificate = self.https_cert;
            config.https.private_key = self.https_key;
        }

        if !self.proxy.is_empty() {
            config.proxy.extend(self.proxy);
        }

        for value in self.mounts {
            config
                .mounts
                .push(parse_mount(&value, &config.root).map_err(|error| error.to_string())?);
        }

        if !self.headers.is_empty() {
            config.headers.extend(self.headers);
        }

        Ok(config)
    }
}

pub struct Cli;

impl Cli {
    pub fn parse<I, S>(args: I) -> Result<CliCommand, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args: Vec<String> = args.into_iter().map(Into::into).collect();

        if args.iter().any(|value| value == "--help" || value == "-h") {
            return Ok(CliCommand::Help);
        }

        if args
            .iter()
            .any(|value| value == "--version" || value == "-V")
        {
            return Ok(CliCommand::Version);
        }

        let (command, rest) = match args.first().map(String::as_str) {
            Some("mcp") => ("mcp", &args[1..]),
            Some("start" | "run") => ("start", &args[1..]),
            Some("stop") => ("stop", &args[1..]),
            Some("restart") => ("restart", &args[1..]),
            Some("open") => ("open", &args[1..]),
            Some("status") => ("status", &args[1..]),
            Some(_) | None => ("start", args.as_slice()),
        };

        match command {
            "mcp" if rest.is_empty() => Ok(CliCommand::Mcp),
            "mcp" => Err(format!("unexpected argument: {}", rest[0])),
            "start" => Self::parse_run(rest).map(|options| CliCommand::Start(Box::new(options))),
            "stop" => Self::parse_control(rest).map(CliCommand::Stop),
            "restart" => {
                Self::parse_run(rest).map(|options| CliCommand::Restart(Box::new(options)))
            }
            "open" => Self::parse_control(rest).map(CliCommand::Open),
            "status" => Self::parse_control(rest).map(CliCommand::Status),
            _ => Err(format!("unknown command: {command}")),
        }
    }

    fn parse_control(args: &[String]) -> Result<ControlOptions, String> {
        let mut root = None;
        let mut browser = None;
        let mut positional = None;
        let mut index = 0;

        while index < args.len() {
            let argument = &args[index];
            let (name, inline) = split_inline(argument);

            match name {
                "--root" => {
                    root = Some(PathBuf::from(value(args, &mut index, inline, "--root")?));
                }
                "--browser" => {
                    browser = Some(
                        parse_browser(&value(args, &mut index, inline, "--browser")?)
                            .map_err(|error| error.to_string())?,
                    );
                }
                _ if argument.starts_with('-') => {
                    return Err(format!("unknown option: {argument}"));
                }
                _ => {
                    if positional.is_some() {
                        return Err(format!("unexpected positional argument: {argument}"));
                    }
                    positional = Some(PathBuf::from(argument));
                }
            }

            index += 1;
        }

        if root.is_some() && positional.is_some() {
            return Err("use either --root or a positional root, not both".into());
        }

        Ok(ControlOptions {
            root: root
                .or(positional)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            browser,
        })
    }

    fn parse_run(args: &[String]) -> Result<RunOptions, String> {
        let mut options = RunOptions::default();
        let mut positional = None;
        let mut index = 0;

        while index < args.len() {
            let argument = &args[index];
            let (name, inline) = split_inline(argument);

            match name {
                "--root" => {
                    options.root = Some(PathBuf::from(value(args, &mut index, inline, "--root")?));
                }
                "--config" => {
                    options.config_path =
                        Some(PathBuf::from(value(args, &mut index, inline, "--config")?));
                }
                "--port" | "-p" => {
                    options.port = Some(parse_value(args, &mut index, inline, "--port")?);
                }
                "--host" => {
                    options.host = Some(parse_value(args, &mut index, inline, "--host")?);
                }
                "--hostname" => {
                    options.preferred_hostname =
                        Some(value(args, &mut index, inline, "--hostname")?);
                }
                "--port-mode" => {
                    options.port_mode = Some(
                        parse_port_mode(&value(args, &mut index, inline, "--port-mode")?)
                            .map_err(|error| error.to_string())?,
                    );
                }
                "--no-browser" => options.no_browser = true,
                "--browser" => {
                    options.browser = Some(
                        parse_browser(&value(args, &mut index, inline, "--browser")?)
                            .map_err(|error| error.to_string())?,
                    );
                }
                "--browser-command" => {
                    options.browser_command =
                        Some(value(args, &mut index, inline, "--browser-command")?);
                }
                "--index" => {
                    options.index_file = Some(value(args, &mut index, inline, "--index")?);
                }
                "--no-directory-listing" => options.directory_listing = Some(false),
                "--no-reload" => options.reload = Some(false),
                "--full-reload" => options.full_reload = true,
                "--wait" => {
                    options.wait_ms = Some(parse_value(args, &mut index, inline, "--wait")?);
                }
                "--no-inject" => options.inject_client = Some(false),
                "--no-watch" => options.watch = Some(false),
                "--non-recursive" => options.recursive = Some(false),
                "--spa" => options.spa = Some(true),
                "--spa-entry" => {
                    options.spa_entry = Some(value(args, &mut index, inline, "--spa-entry")?);
                }
                "--cors" => options.cors = Some(true),
                "--allow-origin" => {
                    options.allow_origins = Some(split_csv(&value(
                        args,
                        &mut index,
                        inline,
                        "--allow-origin",
                    )?));
                }
                "--allow-methods" => {
                    options.allow_methods = Some(split_csv(&value(
                        args,
                        &mut index,
                        inline,
                        "--allow-methods",
                    )?));
                }
                "--allow-headers" => {
                    options.allow_headers = Some(split_csv(&value(
                        args,
                        &mut index,
                        inline,
                        "--allow-headers",
                    )?));
                }
                "--allow-credentials" => options.allow_credentials = Some(true),
                "--https-cert" => {
                    options.https_cert = Some(PathBuf::from(value(
                        args,
                        &mut index,
                        inline,
                        "--https-cert",
                    )?));
                }
                "--https-key" => {
                    options.https_key = Some(PathBuf::from(value(
                        args,
                        &mut index,
                        inline,
                        "--https-key",
                    )?));
                }
                "--proxy" => {
                    options.proxy.push(
                        parse_proxy(&value(args, &mut index, inline, "--proxy")?)
                            .map_err(|error| error.to_string())?,
                    );
                }
                "--mount" => {
                    options
                        .mounts
                        .push(value(args, &mut index, inline, "--mount")?);
                }
                "--header" => {
                    options.headers.push(
                        parse_header(&value(args, &mut index, inline, "--header")?)
                            .map_err(|error| error.to_string())?,
                    );
                }
                "--ignore" => {
                    options
                        .ignores
                        .push(value(args, &mut index, inline, "--ignore")?);
                }
                _ if argument.starts_with('-') => {
                    return Err(format!("unknown option: {argument}"));
                }
                _ => {
                    if positional.is_some() {
                        return Err(format!("unexpected positional argument: {argument}"));
                    }
                    positional = Some(PathBuf::from(argument));
                }
            }

            index += 1;
        }

        if options.root.is_some() && positional.is_some() {
            return Err("use either --root or a positional root, not both".into());
        }

        options.root = options.root.or(positional);
        Ok(options)
    }

    pub fn help() -> &'static str {
        concat!(
            "Rust Live Server ",
            env!("CARGO_PKG_VERSION"),
            r#"

USAGE:
  live-server [start|run] [ROOT] [OPTIONS]
  live-server stop [ROOT]
  live-server restart [ROOT] [OPTIONS]
  live-server open [ROOT] [--browser chrome|edge|firefox]
  live-server status [ROOT]
  live-server mcp
  live-server --help
  live-server --version

LIFECYCLE:
  start                        Start server + watcher + browser
  stop                         Stop running server
  restart                      Stop and start server again
  open                         Open current server URL
  status                       Show PID, URL and state
  mcp                          Run the Zed Live Server MCP server over stdio

SERVER:
  -p, --port <PORT>            Fixed port; 0 selects an OS-assigned port
      --host <IP>              Bind address; 0.0.0.0 enables LAN access
      --hostname <NAME>        Preferred browser URL hostname
      --port-mode <MODE>       fixed | random | next
      --config <FILE>          TOML configuration
      --no-browser
      --browser <NAME>         default | chrome | edge | firefox
      --browser-command <CMD>  {url} is replaced with the server URL
      --index <FILE>
      --no-directory-listing

RELOAD / WATCH:
      --no-reload
      --full-reload
      --wait <MS>
      --no-inject
      --no-watch
      --non-recursive
      --ignore <PATTERN>

SPA / CORS:
      --spa
      --spa-entry <FILE>
      --cors
      --allow-origin <CSV>
      --allow-methods <CSV>
      --allow-headers <CSV>
      --allow-credentials

HTTPS:
      --https-cert <FILE>
      --https-key <FILE>

ROUTING:
      --proxy <PATH=URL>
      --mount <PATH=DIRECTORY>
      --header <NAME=VALUE>

EXAMPLES:
  live-server .
  live-server start ./site --port 5500
  live-server start ./site --host 0.0.0.0 --browser chrome
  live-server stop ./site
  live-server restart ./site --full-reload
  live-server open ./site
  live-server status ./site
"#
        )
    }
}

fn split_inline(arg: &str) -> (&str, Option<&str>) {
    match arg.split_once('=') {
        Some((name, value)) if name.starts_with("--") => (name, Some(value)),
        _ => (arg, None),
    }
}

fn value(
    args: &[String],
    index: &mut usize,
    inline: Option<&str>,
    name: &str,
) -> Result<String, String> {
    if let Some(value) = inline {
        return Ok(value.into());
    }

    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("{name} requires a value"))
}

fn parse_value<T>(
    args: &[String],
    index: &mut usize,
    inline: Option<&str>,
    name: &str,
) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    value(args, index, inline, name)?
        .parse::<T>()
        .map_err(|error| format!("{name}: {error}"))
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positional_root_is_parsed() {
        let command = Cli::parse(["start", "./site", "--no-browser"]).unwrap();

        match command {
            // FIX: match the Box value; do not call Box::new inside a pattern.
            CliCommand::Start(options) => {
                assert_eq!(options.root.as_ref().unwrap(), &PathBuf::from("./site"));
                assert!(options.no_browser);
            }
            _ => panic!("expected start"),
        }
    }

    #[test]
    fn stop_is_parsed() {
        let command = Cli::parse(["stop", "./site"]).unwrap();

        match command {
            CliCommand::Stop(options) => {
                assert_eq!(options.root, PathBuf::from("./site"));
            }
            _ => panic!("expected stop"),
        }
    }

    #[test]
    fn restart_is_parsed() {
        let command = Cli::parse(["restart", "./site", "--full-reload"]).unwrap();

        match command {
            // FIX: options is already a Box<RunOptions>.
            CliCommand::Restart(options) => {
                assert!(options.full_reload);
                assert_eq!(options.root.as_ref().unwrap(), &PathBuf::from("./site"));
            }
            _ => panic!("expected restart"),
        }
    }

    #[test]
    fn root_and_positional_cannot_be_combined() {
        assert!(Cli::parse(["start", "./site", "--root", "./other"]).is_err());
    }

    #[test]
    fn help_flag_is_parsed() {
        assert!(matches!(Cli::parse(["--help"]).unwrap(), CliCommand::Help));
    }

    #[test]
    fn help_uses_the_package_version() {
        assert!(Cli::help().contains(crate::VERSION));
    }

    #[test]
    fn version_flag_is_parsed() {
        assert!(matches!(
            Cli::parse(["--version"]).unwrap(),
            CliCommand::Version
        ));
    }

    #[test]
    fn mcp_command_is_parsed() {
        assert!(matches!(Cli::parse(["mcp"]).unwrap(), CliCommand::Mcp));
        assert!(Cli::parse(["mcp", "unexpected"]).is_err());
    }

    #[test]
    fn inline_port_is_parsed() {
        let command = Cli::parse(["start", "--port=5501", "--no-browser"]).unwrap();

        match command {
            CliCommand::Start(options) => assert_eq!(options.port, Some(5501)),
            _ => panic!("expected start"),
        }
    }

    #[test]
    fn unknown_option_is_rejected() {
        assert!(Cli::parse(["start", "--unknown-option"]).is_err());
    }

    #[test]
    fn missing_port_value_is_rejected() {
        assert!(Cli::parse(["start", "--port"]).is_err());
    }
}
