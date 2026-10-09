use crate::config::ProxyRule;
use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{Response, StatusCode, header},
};
use reqwest::Client;

pub fn matches(rule: &ProxyRule, path: &str) -> bool {
    path == rule.path || path.starts_with(&(rule.path.trim_end_matches('/').to_string() + "/"))
}

pub fn join_target(rule: &ProxyRule, uri: &axum::http::Uri) -> String {
    let base = rule.target.trim_end_matches('/');
    let suffix = uri
        .path()
        .strip_prefix(rule.path.trim_end_matches('/'))
        .unwrap_or(uri.path());
    let mut result = if suffix.is_empty() {
        format!("{base}/")
    } else if suffix.starts_with('/') {
        format!("{base}{suffix}")
    } else {
        format!("{base}/{suffix}")
    };
    if let Some(query) = uri.query() {
        result.push('?');
        result.push_str(query);
    }
    result
}

pub async fn forward(
    client: &Client,
    request: Request,
    rule: &ProxyRule,
) -> Result<Response<Body>, String> {
    let (parts, body) = request.into_parts();
    let url = join_target(rule, &parts.uri);
    let bytes = to_bytes(body, 16 * 1024 * 1024)
        .await
        .map_err(|e| format!("reading request body: {e}"))?;
    let mut builder = client.request(parts.method, &url).body(bytes);
    for (name, value) in &parts.headers {
        if name == header::CONTENT_LENGTH
            || name == header::TRANSFER_ENCODING
            || name == header::CONNECTION
        {
            continue;
        }
        if name == header::HOST {
            if !rule.change_origin {
                builder = builder.header(name, value);
            }
            continue;
        }
        builder = builder.header(name, value);
    }
    let upstream = builder.send().await.map_err(|e| e.to_string())?;
    let status = StatusCode::from_u16(upstream.status().as_u16()).map_err(|e| e.to_string())?;
    let headers = upstream.headers().clone();
    let bytes = upstream.bytes().await.map_err(|e| e.to_string())?;
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    for (name, value) in &headers {
        if name == header::CONNECTION
            || name == header::TRANSFER_ENCODING
            || name == header::CONTENT_LENGTH
        {
            continue;
        }
        response.headers_mut().insert(name.clone(), value.clone());
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_boundary_matches() {
        let r = ProxyRule {
            path: "/api".into(),
            target: "http://localhost:3000".into(),
            change_origin: true,
        };
        assert!(matches(&r, "/api/x"));
        assert!(!matches(&r, "/apis/x"));
    }
    #[test]
    fn query_is_preserved() {
        let r = ProxyRule {
            path: "/api".into(),
            target: "http://localhost:3000".into(),
            change_origin: true,
        };
        let u = axum::http::Uri::from_static("/api/users?x=1");
        assert_eq!(join_target(&r, &u), "http://localhost:3000/users?x=1");
    }
}
