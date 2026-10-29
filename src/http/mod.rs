use crate::{
    config::{HeaderRule, MountRule, PortMode, ServerConfig},
    proxy, security, spa, websocket,
};
use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    response::Response,
    routing::{any, get},
};
use mime_guess::from_path;
use reqwest::Client;
use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ServerConfig>,
    pub reload_tx: broadcast::Sender<crate::reload::ReloadEvent>,
    pub client: Client,
}
impl AppState {
    pub fn new(config: ServerConfig) -> Self {
        let (reload_tx, _) = broadcast::channel(256);
        Self {
            config: Arc::new(config),
            reload_tx,
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/__live_server/health", get(health))
        .route("/__live_server/status", get(status))
        .route("/__live_server/live-reload.js", get(reload_client))
        .route("/__live_server/ws", any(websocket::handler))
        .fallback(any(handle_request))
        .with_state(state)
}

pub async fn bind_listener(
    host: IpAddr,
    port: u16,
    mode: PortMode,
) -> std::io::Result<(TcpListener, SocketAddr)> {
    match mode {
        PortMode::Fixed => {
            let addr = SocketAddr::new(host, port);
            let listener = TcpListener::bind(addr).await?;
            let local = listener.local_addr()?;
            Ok((listener, local))
        }
        PortMode::Random => {
            let addr = SocketAddr::new(host, 0);
            let listener = TcpListener::bind(addr).await?;
            let local = listener.local_addr()?;
            Ok((listener, local))
        }
        PortMode::NextAvailable => {
            let start = if port == 0 { 5500 } else { port };
            let end = start.saturating_add(1000);
            for candidate in start..=end {
                match TcpListener::bind(SocketAddr::new(host, candidate)).await {
                    Ok(listener) => {
                        let local = listener.local_addr()?;
                        return Ok((listener, local));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => continue,
                    Err(e) => return Err(e),
                }
            }
            Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "no available TCP port found",
            ))
        }
    }
}

async fn health(State(state): State<AppState>, request: Request) -> Response {
    let mut response = text_response(StatusCode::OK, "ok\n", "text/plain; charset=utf-8");
    decorate(
        &mut response,
        &state.config,
        request.headers().get(header::ORIGIN),
    );
    response
}
async fn status(State(state): State<AppState>, request: Request) -> Response {
    let root = state.config.root.display().to_string();
    let body = serde_json::json!({"version":crate::VERSION,"root":root,"port":state.config.port,"reload":state.config.reload.enabled,"watch":state.config.watch.enabled});
    let mut response = json_response(StatusCode::OK, &body.to_string());
    decorate(
        &mut response,
        &state.config,
        request.headers().get(header::ORIGIN),
    );
    response
}
async fn reload_client(State(state): State<AppState>, request: Request) -> Response {
    let content = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/web/live-reload.js"));
    let mut response = text_response(
        StatusCode::OK,
        content,
        "application/javascript; charset=utf-8",
    );
    decorate(
        &mut response,
        &state.config,
        request.headers().get(header::ORIGIN),
    );
    response
}

async fn handle_request(State(state): State<AppState>, request: Request) -> Response {
    let method = request.method().clone();
    let raw_path = request.uri().path().to_string();
    let origin = request.headers().get(header::ORIGIN).cloned();
    if method == Method::OPTIONS && state.config.cors.enabled {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::NO_CONTENT;
        decorate(&mut response, &state.config, origin.as_ref());
        return response;
    }
    if !matches!(
        method,
        Method::GET
            | Method::HEAD
            | Method::POST
            | Method::PUT
            | Method::PATCH
            | Method::DELETE
            | Method::OPTIONS
    ) {
        return simple(
            StatusCode::METHOD_NOT_ALLOWED,
            "method not allowed",
            &state.config,
            origin.as_ref(),
        );
    }
    if !matches!(method, Method::GET | Method::HEAD)
        && state
            .config
            .proxy
            .iter()
            .all(|r| !proxy::matches(r, &raw_path))
    {
        return simple(
            StatusCode::METHOD_NOT_ALLOWED,
            "static files accept GET and HEAD only",
            &state.config,
            origin.as_ref(),
        );
    }

    if let Some(rule) = state
        .config
        .proxy
        .iter()
        .find(|r| proxy::matches(r, &raw_path))
        .cloned()
    {
        return match proxy::forward(&state.client, request, &rule).await {
            Ok(mut response) => {
                decorate(&mut response, &state.config, origin.as_ref());
                response
            }
            Err(e) => simple(
                StatusCode::BAD_GATEWAY,
                &format!("Proxy error: {e}"),
                &state.config,
                origin.as_ref(),
            ),
        };
    }

    let normalized = match security::normalize_request_path(&raw_path) {
        Ok(v) => v,
        Err(e) => {
            return simple(
                StatusCode::FORBIDDEN,
                &format!("403 Forbidden: {e}"),
                &state.config,
                origin.as_ref(),
            );
        }
    };
    let (root, local_path) = mount_target(&state.config, &normalized)
        .unwrap_or_else(|| (state.config.root.clone(), normalized.clone()));
    let requested = match security::safe_join(&root, &local_path) {
        Ok(v) => v,
        Err(e) => {
            return simple(
                StatusCode::FORBIDDEN,
                &format!("403 Forbidden: {e}"),
                &state.config,
                origin.as_ref(),
            );
        }
    };

    if requested.is_dir() && normalized != "/" && !raw_path.ends_with('/') && method == Method::GET
    {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::PERMANENT_REDIRECT;
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&format!("{raw_path}/"))
                .unwrap_or_else(|_| HeaderValue::from_static("/")),
        );
        decorate(&mut response, &state.config, origin.as_ref());
        return response;
    }

    let accepts_html = request
        .headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').any(|x| x.trim().starts_with("text/html")))
        .unwrap_or(true);
    let file = if requested.is_file() {
        Some(requested.clone())
    } else if requested.is_dir() {
        let index_path = format!(
            "{}/{}",
            local_path.trim_end_matches('/'),
            state.config.index_file
        );
        match security::safe_join(&root, &index_path) {
            Ok(index) if index.is_file() => Some(index),
            Ok(_) => None,
            Err(error) => {
                return simple(
                    StatusCode::FORBIDDEN,
                    &format!("403 Forbidden: {error}"),
                    &state.config,
                    origin.as_ref(),
                );
            }
        }
    } else {
        None
    };
    let file = match file {
        Some(v) => Some(v),
        None if spa::should_fallback(&requested, accepts_html, state.config.spa.enabled) => {
            security::safe_join(
                &state.config.root,
                &format!("/{}", state.config.spa.entry_file.trim_start_matches('/')),
            )
            .ok()
            .filter(|v| v.is_file())
        }
        _ => None,
    };

    if let Some(file) = file {
        return match serve_file(&file, &method, &state.config).await {
            Ok(mut r) => {
                decorate(&mut r, &state.config, origin.as_ref());
                r
            }
            Err(e) => simple(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e,
                &state.config,
                origin.as_ref(),
            ),
        };
    }
    if requested.is_dir() && state.config.directory_listing && method == Method::GET {
        return directory_listing(&requested, &normalized, &state.config, origin.as_ref()).await;
    }
    simple(
        StatusCode::NOT_FOUND,
        "404 Not Found",
        &state.config,
        origin.as_ref(),
    )
}

fn mount_target(config: &ServerConfig, path: &str) -> Option<(PathBuf, String)> {
    let mut best: Option<&MountRule> = None;
    for m in &config.mounts {
        if (path == m.url_path
            || path.starts_with(&(m.url_path.trim_end_matches('/').to_string() + "/")))
            && best.is_none_or(|b| m.url_path.len() > b.url_path.len())
        {
            best = Some(m);
        }
    }
    best.map(|m| {
        let rem = path.strip_prefix(m.url_path.as_str()).unwrap_or("/");
        (
            m.directory.clone(),
            if rem.is_empty() {
                "/".into()
            } else {
                rem.into()
            },
        )
    })
}

async fn serve_file(
    path: &Path,
    method: &Method,
    config: &ServerConfig,
) -> Result<Response<Body>, String> {
    let mut bytes = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
    let mime = from_path(path).first_or_octet_stream().to_string();
    let is_html = matches!(
        path.extension()
            .and_then(|x| x.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "html" | "htm"
    );
    if is_html
        && config.reload.enabled
        && config.reload.inject_client
        && let Ok(text) = String::from_utf8(bytes.clone())
        && !text.contains("/__live_server/live-reload.js")
    {
        let script = "<script src=\"/__live_server/live-reload.js\"></script>";
        let lower = text.to_ascii_lowercase();
        let at = lower
            .find("</body>")
            .or_else(|| lower.find("</html>"))
            .unwrap_or(text.len());
        let mut out = text;
        out.insert_str(at, script);
        bytes = out.into_bytes();
    }
    let mut response = Response::new(if *method == Method::HEAD {
        Body::empty()
    } else {
        Body::from(bytes.clone())
    });
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).map_err(|e| e.to_string())?,
    );
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&bytes.len().to_string()).map_err(|e| e.to_string())?,
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    Ok(response)
}

async fn directory_listing(
    directory: &Path,
    request_path: &str,
    config: &ServerConfig,
    origin: Option<&HeaderValue>,
) -> Response {
    let mut entries = match tokio::fs::read_dir(directory).await {
        Ok(v) => v,
        Err(e) => {
            return simple(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
                config,
                origin,
            );
        }
    };
    let mut items = Vec::new();
    while let Ok(Some(e)) = entries.next_entry().await {
        let name = e.file_name().to_string_lossy().into_owned();
        let dir = e.file_type().await.map(|v| v.is_dir()).unwrap_or(false);
        items.push((name, dir));
    }
    items.sort_by_key(|a| a.0.to_ascii_lowercase());
    let mut html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Index of {request_path}</title></head><body><h1>Index of {request_path}</h1><ul>"
    );
    if request_path != "/" {
        let parent = request_path
            .trim_end_matches('/')
            .rsplit_once('/')
            .map(|x| {
                if x.0.is_empty() {
                    "/".to_string()
                } else {
                    format!("{}/", x.0)
                }
            })
            .unwrap_or("/".into());
        html.push_str(&format!("<li><a href=\"{parent}\">../</a></li>"));
    }
    for (name, dir) in items {
        let safe = escape_html(&name);
        let href = format!(
            "{}{}/",
            request_path.trim_end_matches('/'),
            urlencoding_name(&name)
        );
        let href = if dir {
            href
        } else {
            href.trim_end_matches('/').to_string()
        };
        html.push_str(&format!(
            "<li><a href=\"{href}\">{}{}</a></li>",
            safe,
            if dir { "/" } else { "" }
        ));
    }
    html.push_str("</ul></body></html>");
    let mut response = text_response(StatusCode::OK, &html, "text/html; charset=utf-8");
    decorate(&mut response, config, origin);
    response
}
fn urlencoding_name(v: &str) -> String {
    v.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
fn escape_html(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn decorate(response: &mut Response<Body>, config: &ServerConfig, origin: Option<&HeaderValue>) {
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    for HeaderRule { name, value } in &config.headers {
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            response.headers_mut().insert(n, v);
        }
    }
    if config.cors.enabled {
        if let Some(origin) = origin {
            let allowed = if config.cors.allow_origins.iter().any(|allowed| {
                allowed.eq_ignore_ascii_case(origin.to_str().unwrap_or(""))
                    || (allowed == "*" && !config.cors.allow_credentials)
            }) {
                origin.clone()
            } else {
                HeaderValue::from_static("")
            };
            if !allowed.is_empty() {
                response
                    .headers_mut()
                    .insert("access-control-allow-origin", allowed);
            }
        } else if !config.cors.allow_credentials
            && config.cors.allow_origins.iter().any(|x| x == "*")
        {
            response
                .headers_mut()
                .insert("access-control-allow-origin", HeaderValue::from_static("*"));
        }
        response.headers_mut().insert(
            "access-control-allow-methods",
            HeaderValue::from_str(&config.cors.allow_methods.join(", "))
                .unwrap_or_else(|_| HeaderValue::from_static("GET, OPTIONS")),
        );
        response.headers_mut().insert(
            "access-control-allow-headers",
            HeaderValue::from_str(&config.cors.allow_headers.join(", "))
                .unwrap_or_else(|_| HeaderValue::from_static("*")),
        );
        if config.cors.allow_credentials {
            response.headers_mut().insert(
                "access-control-allow-credentials",
                HeaderValue::from_static("true"),
            );
        }
        response
            .headers_mut()
            .insert("vary", HeaderValue::from_static("Origin"));
    }
}
fn text_response(status: StatusCode, body: &str, mime: &str) -> Response<Body> {
    let mut r = Response::new(Body::from(body.to_owned()));
    *r.status_mut() = status;
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime)
            .unwrap_or_else(|_| HeaderValue::from_static("text/plain; charset=utf-8")),
    );
    r
}
fn json_response(status: StatusCode, body: &str) -> Response<Body> {
    text_response(status, body, "application/json; charset=utf-8")
}
fn simple(
    status: StatusCode,
    body: &str,
    config: &ServerConfig,
    origin: Option<&HeaderValue>,
) -> Response<Body> {
    let mut r = text_response(status, body, "text/plain; charset=utf-8");
    decorate(&mut r, config, origin);
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    #[tokio::test]
    async fn random_port() {
        let (_, a) = bind_listener(IpAddr::V4(Ipv4Addr::LOCALHOST), 0, PortMode::Random)
            .await
            .unwrap();
        assert!(a.port() > 0);
    }
    #[test]
    fn directory_name_url_encoding() {
        assert_eq!(urlencoding_name("a b"), "a%20b");
    }
}
