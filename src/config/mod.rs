use axum::http::{HeaderName, HeaderValue};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env, fmt,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Browser {
    #[default]
    Default,
    Chrome,
    Edge,
    Firefox,
    Custom {
        program: String,
        args: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PortMode {
    #[default]
    Fixed,
    Random,
    NextAvailable,
}

#[derive(Clone, Debug)]
pub struct ReloadConfig {
    pub enabled: bool,
    pub full_reload: bool,
    pub wait_ms: u64,
    pub inject_client: bool,
}
impl Default for ReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            full_reload: false,
            wait_ms: 150,
            inject_client: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WatchConfig {
    pub enabled: bool,
    pub recursive: bool,
    pub ignore: Vec<String>,
    pub extensions: Vec<String>,
}
impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            recursive: true,
            ignore: vec![
                ".git".into(),
                "target".into(),
                "node_modules".into(),
                ".idea".into(),
                ".vscode".into(),
                ".zed".into(),
                ".live-server".into(),
            ],
            extensions: vec![
                "html".into(),
                "htm".into(),
                "css".into(),
                "js".into(),
                "mjs".into(),
                "cjs".into(),
                "json".into(),
                "xml".into(),
                "csv".into(),
                "txt".into(),
                "md".into(),
                "map".into(),
                "svg".into(),
                "png".into(),
                "jpg".into(),
                "jpeg".into(),
                "gif".into(),
                "webp".into(),
                "avif".into(),
                "bmp".into(),
                "ico".into(),
                "tif".into(),
                "tiff".into(),
                "woff".into(),
                "woff2".into(),
                "ttf".into(),
                "otf".into(),
                "eot".into(),
                "wasm".into(),
                "mp3".into(),
                "wav".into(),
                "ogg".into(),
                "m4a".into(),
                "mp4".into(),
                "webm".into(),
                "mov".into(),
                "pdf".into(),
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpaConfig {
    pub enabled: bool,
    pub entry_file: String,
}
impl Default for SpaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            entry_file: "index.html".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub allow_credentials: bool,
}
impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_origins: vec!["*".into()],
            allow_methods: vec![
                "GET".into(),
                "POST".into(),
                "PUT".into(),
                "PATCH".into(),
                "DELETE".into(),
                "OPTIONS".into(),
            ],
            allow_headers: vec!["*".into()],
            allow_credentials: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HttpsConfig {
    pub enabled: bool,
    pub certificate: Option<PathBuf>,
    pub private_key: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProxyRule {
    pub path: String,
    pub target: String,
    pub change_origin: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountRule {
    pub url_path: String,
    pub directory: PathBuf,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderRule {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub root: PathBuf,
    pub host: IpAddr,
    pub preferred_hostname: Option<String>,
    pub port: u16,
    pub port_mode: PortMode,
    pub open_browser: bool,
    pub browser: Browser,
    pub browser_command: Option<String>,
    pub index_file: String,
    pub directory_listing: bool,
    pub reload: ReloadConfig,
    pub watch: WatchConfig,
    pub spa: SpaConfig,
    pub cors: CorsConfig,
    pub https: HttpsConfig,
    pub proxy: Vec<ProxyRule>,
    pub mounts: Vec<MountRule>,
    pub headers: Vec<HeaderRule>,
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            root: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            preferred_hostname: None,
            port: 5500,
            port_mode: PortMode::Fixed,
            open_browser: true,
            browser: Browser::Default,
            browser_command: None,
            index_file: "index.html".into(),
            directory_listing: true,
            reload: ReloadConfig::default(),
            watch: WatchConfig::default(),
            spa: SpaConfig::default(),
            cors: CorsConfig::default(),
            https: HttpsConfig::default(),
            proxy: Vec::new(),
            mounts: Vec::new(),
            headers: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct FileConfig {
    pub root: Option<String>,
    pub host: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub port_mode: Option<String>,
    pub open_browser: Option<bool>,
    pub browser: Option<String>,
    pub browser_command: Option<String>,
    pub index_file: Option<String>,
    pub directory_listing: Option<bool>,
    pub reload: Option<bool>,
    pub full_reload: Option<bool>,
    pub wait_ms: Option<u64>,
    pub inject_client: Option<bool>,
    pub watch: Option<bool>,
    pub recursive: Option<bool>,
    pub ignore: Option<Vec<String>>,
    pub extensions: Option<Vec<String>>,
    pub spa: Option<bool>,
    pub spa_entry: Option<String>,
    pub cors: Option<bool>,
    pub allow_origins: Option<Vec<String>>,
    pub allow_methods: Option<Vec<String>>,
    pub allow_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub https: Option<bool>,
    pub certificate: Option<String>,
    pub private_key: Option<String>,
    pub proxy: Option<HashMap<String, String>>,
    pub mount: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidRoot(PathBuf),
    InvalidHost(String),
    InvalidPortMode(String),
    InvalidBrowser(String),
    InvalidWait(u64),
    InvalidIndex,
    InvalidCors,
    InvalidSpaEntry,
    InvalidHttps,
    InvalidProxy(String),
    InvalidMount(String),
    InvalidHeader(String),
    InvalidConfigFile(String),
    InvalidHeaderName(String),
    InvalidHeaderValue(String),
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot(p) => write!(
                f,
                "root does not exist or is not a directory: {}",
                p.display()
            ),
            Self::InvalidHost(v) => write!(f, "invalid host/IP address: {v}"),
            Self::InvalidPortMode(v) => write!(f, "invalid port mode: {v}"),
            Self::InvalidBrowser(v) => write!(f, "invalid browser: {v}"),
            Self::InvalidWait(v) => write!(f, "wait must be <= 60000 ms, got {v}"),
            Self::InvalidIndex => {
                write!(
                    f,
                    "index_file must be a relative path inside the server root"
                )
            }
            Self::InvalidCors => {
                write!(
                    f,
                    "CORS credentials cannot be enabled with a wildcard origin"
                )
            }
            Self::InvalidSpaEntry => {
                write!(f, "SPA entry file must be a relative file inside root")
            }
            Self::InvalidHttps => {
                write!(f, "HTTPS requires both certificate and private_key files")
            }
            Self::InvalidProxy(v) => write!(f, "invalid proxy rule: {v}"),
            Self::InvalidMount(v) => write!(f, "invalid mount rule: {v}"),
            Self::InvalidHeader(v) => write!(f, "invalid header rule: {v}"),
            Self::InvalidConfigFile(v) => write!(f, "cannot parse live-server.toml: {v}"),
            Self::InvalidHeaderName(v) => write!(f, "invalid header name: {v}"),
            Self::InvalidHeaderValue(v) => write!(f, "invalid header value: {v}"),
        }
    }
}
impl std::error::Error for ConfigError {}

pub fn normalize_route(value: &str) -> String {
    let trimmed = value.trim().replace('\\', "/");
    if trimmed.is_empty() {
        return "/".into();
    }
    let mut result = trimmed;
    if !result.starts_with('/') {
        result.insert(0, '/');
    }
    while result.contains("//") {
        result = result.replace("//", "/");
    }
    if result.len() > 1 {
        result = result.trim_end_matches('/').to_string();
    }
    result
}

pub fn parse_port_mode(value: &str) -> Result<PortMode, ConfigError> {
    match value.to_ascii_lowercase().as_str() {
        "fixed" => Ok(PortMode::Fixed),
        "random" => Ok(PortMode::Random),
        "next" | "next-available" => Ok(PortMode::NextAvailable),
        _ => Err(ConfigError::InvalidPortMode(value.into())),
    }
}
pub fn parse_browser(value: &str) -> Result<Browser, ConfigError> {
    match value.to_ascii_lowercase().as_str() {
        "default" => Ok(Browser::Default),
        "chrome" => Ok(Browser::Chrome),
        "edge" => Ok(Browser::Edge),
        "firefox" => Ok(Browser::Firefox),
        _ if value.contains('=') => {
            let (program, args) = value.split_once('=').unwrap();
            Ok(Browser::Custom {
                program: program.into(),
                args: args.split_whitespace().map(str::to_owned).collect(),
            })
        }
        _ => Err(ConfigError::InvalidBrowser(value.into())),
    }
}
pub fn parse_proxy(value: &str) -> Result<ProxyRule, ConfigError> {
    let (path, target) = value
        .split_once('=')
        .ok_or_else(|| ConfigError::InvalidProxy(value.into()))?;
    if path.trim().is_empty() || target.trim().is_empty() {
        return Err(ConfigError::InvalidProxy(value.into()));
    }
    if !target.starts_with("http://") && !target.starts_with("https://") {
        return Err(ConfigError::InvalidProxy(value.into()));
    }
    Ok(ProxyRule {
        path: normalize_route(path),
        target: target.trim_end_matches('/').into(),
        change_origin: true,
    })
}
pub fn parse_mount(value: &str, base: &Path) -> Result<MountRule, ConfigError> {
    let (url_path, directory) = value
        .split_once('=')
        .ok_or_else(|| ConfigError::InvalidMount(value.into()))?;
    if url_path.trim().is_empty() || directory.trim().is_empty() {
        return Err(ConfigError::InvalidMount(value.into()));
    }
    let dir = PathBuf::from(directory.trim());
    Ok(MountRule {
        url_path: normalize_route(url_path),
        directory: if dir.is_absolute() {
            dir
        } else {
            base.join(dir)
        },
    })
}
pub fn parse_header(value: &str) -> Result<HeaderRule, ConfigError> {
    let (name, val) = value
        .split_once('=')
        .ok_or_else(|| ConfigError::InvalidHeader(value.into()))?;
    if name.trim().is_empty() {
        return Err(ConfigError::InvalidHeader(value.into()));
    }
    HeaderName::from_str(name.trim())
        .map_err(|_| ConfigError::InvalidHeaderName(name.trim().into()))?;
    HeaderValue::from_str(val).map_err(|_| ConfigError::InvalidHeaderValue(name.into()))?;
    Ok(HeaderRule {
        name: name.trim().into(),
        value: val.into(),
    })
}

impl ServerConfig {
    pub fn load_file(&mut self, path: &Path) -> Result<(), ConfigError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::InvalidConfigFile(e.to_string()))?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let file: FileConfig =
            toml::from_str(&text).map_err(|e| ConfigError::InvalidConfigFile(e.to_string()))?;
        self.apply_file_config(file, base)
    }

    pub fn apply_file_config(&mut self, file: FileConfig, base: &Path) -> Result<(), ConfigError> {
        if let Some(v) = file.root {
            let p = PathBuf::from(v);
            self.root = if p.is_absolute() { p } else { base.join(p) };
        }
        if let Some(v) = file.host {
            self.host = v.parse().map_err(|_| ConfigError::InvalidHost(v))?;
        }
        if let Some(v) = file.hostname {
            self.preferred_hostname = Some(v);
        }
        if let Some(v) = file.port {
            self.port = v;
        }
        if let Some(v) = file.port_mode {
            self.port_mode = parse_port_mode(&v)?;
        }
        if let Some(v) = file.open_browser {
            self.open_browser = v;
        }
        if let Some(v) = file.browser {
            self.browser = parse_browser(&v)?;
        }
        if let Some(v) = file.browser_command {
            self.browser_command = Some(v);
        }
        if let Some(v) = file.index_file {
            self.index_file = v;
        }
        if let Some(v) = file.directory_listing {
            self.directory_listing = v;
        }
        if let Some(v) = file.reload {
            self.reload.enabled = v;
        }
        if let Some(v) = file.full_reload {
            self.reload.full_reload = v;
        }
        if let Some(v) = file.wait_ms {
            self.reload.wait_ms = v;
        }
        if let Some(v) = file.inject_client {
            self.reload.inject_client = v;
        }
        if let Some(v) = file.watch {
            self.watch.enabled = v;
        }
        if let Some(v) = file.recursive {
            self.watch.recursive = v;
        }
        if let Some(v) = file.ignore {
            self.watch.ignore = v;
        }
        if let Some(v) = file.extensions {
            self.watch.extensions = v;
        }
        if let Some(v) = file.spa {
            self.spa.enabled = v;
        }
        if let Some(v) = file.spa_entry {
            self.spa.entry_file = v;
        }
        if let Some(v) = file.cors {
            self.cors.enabled = v;
        }
        if let Some(v) = file.allow_origins {
            self.cors.allow_origins = v;
        }
        if let Some(v) = file.allow_methods {
            self.cors.allow_methods = v;
        }
        if let Some(v) = file.allow_headers {
            self.cors.allow_headers = v;
        }
        if let Some(v) = file.allow_credentials {
            self.cors.allow_credentials = v;
        }
        if let Some(v) = file.https {
            self.https.enabled = v;
        }
        if let Some(v) = file.certificate {
            self.https.certificate = Some(resolve_path(base, v));
        }
        if let Some(v) = file.private_key {
            self.https.private_key = Some(resolve_path(base, v));
        }
        if let Some(proxy) = file.proxy {
            for (path, target) in proxy {
                self.proxy.push(ProxyRule {
                    path: normalize_route(&path),
                    target: target.trim_end_matches('/').into(),
                    change_origin: true,
                });
            }
        }
        if let Some(mounts) = file.mount {
            for (path, dir) in mounts {
                self.mounts
                    .push(parse_mount(&format!("{path}={dir}"), base)?);
            }
        }
        if let Some(headers) = file.headers {
            for (name, value) in headers {
                self.headers.push(parse_header(&format!("{name}={value}"))?);
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.root.is_dir() {
            return Err(ConfigError::InvalidRoot(self.root.clone()));
        }
        let index_path = Path::new(&self.index_file);
        if self.index_file.trim().is_empty()
            || self.index_file.starts_with('/')
            || index_path.is_absolute()
            || index_path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
            || !index_path
                .components()
                .any(|component| matches!(component, std::path::Component::Normal(_)))
            || self
                .index_file
                .chars()
                .any(|character| matches!(character, '\\' | ':'))
        {
            return Err(ConfigError::InvalidIndex);
        }
        if self.reload.wait_ms > 60_000 {
            return Err(ConfigError::InvalidWait(self.reload.wait_ms));
        }
        if self.cors.enabled
            && self.cors.allow_credentials
            && self.cors.allow_origins.iter().any(|origin| origin == "*")
        {
            return Err(ConfigError::InvalidCors);
        }
        let entry = Path::new(&self.spa.entry_file);
        if self.spa.enabled
            && (entry.is_absolute()
                || entry
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir)))
        {
            return Err(ConfigError::InvalidSpaEntry);
        }
        if self.https.enabled
            && (self.https.certificate.is_none() || self.https.private_key.is_none())
        {
            return Err(ConfigError::InvalidHttps);
        }
        for mount in &self.mounts {
            if !mount.directory.is_dir() {
                return Err(ConfigError::InvalidMount(
                    mount.directory.display().to_string(),
                ));
            }
        }
        for header in &self.headers {
            HeaderName::from_str(&header.name)
                .map_err(|_| ConfigError::InvalidHeaderName(header.name.clone()))?;
            HeaderValue::from_str(&header.value)
                .map_err(|_| ConfigError::InvalidHeaderValue(header.name.clone()))?;
        }
        Ok(())
    }

    pub fn watched_roots(&self) -> Vec<(PathBuf, String)> {
        let mut roots = vec![(self.root.clone(), "/".into())];
        roots.extend(
            self.mounts
                .iter()
                .map(|m| (m.directory.clone(), m.url_path.clone())),
        );
        roots
    }

    pub fn normalized_root(&self) -> Result<PathBuf, ConfigError> {
        std::fs::canonicalize(&self.root).map_err(|_| ConfigError::InvalidRoot(self.root.clone()))
    }
    pub fn summary(&self, url: &str, actual_port: u16) -> String {
        format!(
            "\n=============================================\n             RUST LIVE SERVER\n=============================================\nVersion:  {}\nRoot:     {}\nLocal:    {url}\nBind:     {}:{}\nReload:   {}\nWatcher:  {}\nHTTPS:    {}\n=============================================\n",
            crate::VERSION,
            self.root.display(),
            self.host,
            actual_port,
            if self.reload.enabled {
                "enabled"
            } else {
                "disabled"
            },
            if self.watch.enabled {
                "enabled"
            } else {
                "disabled"
            },
            if self.https.enabled {
                "enabled"
            } else {
                "disabled"
            }
        )
    }
}

fn resolve_path(base: &Path, value: String) -> PathBuf {
    let p = PathBuf::from(value);
    if p.is_absolute() { p } else { base.join(p) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_route_becomes_root() {
        assert_eq!(normalize_route(""), "/");
    }
    #[test]
    fn slash_route_is_normalized() {
        assert_eq!(normalize_route("foo/"), "/foo");
    }
    #[test]
    fn port_zero_means_random_is_allowed() {
        let c = ServerConfig {
            port: 0,
            ..ServerConfig::default()
        };
        assert_eq!(c.port, 0);
    }
    #[test]
    fn browser_default_is_usable() {
        assert_eq!(Browser::default(), Browser::Default);
    }
    #[test]
    fn hostname_from_file_is_applied() {
        let mut config = ServerConfig::default();
        config
            .apply_file_config(
                FileConfig {
                    hostname: Some("dev.local".into()),
                    ..FileConfig::default()
                },
                Path::new("."),
            )
            .unwrap();
        assert_eq!(config.preferred_hostname.as_deref(), Some("dev.local"));
    }

    #[test]
    fn index_file_must_be_a_relative_path_inside_root() {
        let root = std::env::temp_dir();
        let mut config = ServerConfig {
            root,
            ..ServerConfig::default()
        };

        for index_file in ["", "../index.html", "/tmp/index.html", r"..\index.html"] {
            config.index_file = index_file.into();
            assert_eq!(config.validate(), Err(ConfigError::InvalidIndex));
        }

        for index_file in ["pages/index.html", "./pages/index.html"] {
            config.index_file = index_file.into();
            assert!(config.validate().is_ok());
        }
    }

    #[test]
    fn cors_rejects_wildcard_origins_with_credentials() {
        let config = ServerConfig {
            root: std::env::temp_dir(),
            cors: CorsConfig {
                enabled: true,
                allow_credentials: true,
                ..CorsConfig::default()
            },
            ..ServerConfig::default()
        };

        assert_eq!(config.validate(), Err(ConfigError::InvalidCors));
    }
}
