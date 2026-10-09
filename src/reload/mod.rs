use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReloadEvent {
    pub path: String,
    pub kind: String,
    pub timestamp_ms: u128,
}

pub fn classify(path: &Path, full_reload: bool) -> &'static str {
    if full_reload {
        return "reload";
    }
    match path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "css" => "css",
        "html" | "htm" => "html",
        "js" | "mjs" | "cjs" => "js",
        _ => "reload",
    }
}
pub fn event(path: &str, full_reload: bool, kind_hint: &str) -> ReloadEvent {
    ReloadEvent {
        path: path.into(),
        kind: if full_reload {
            "reload".into()
        } else {
            kind_hint.into()
        },
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn css_is_hot_reload() {
        assert_eq!(classify(Path::new("a.css"), false), "css");
    }
    #[test]
    fn html_is_full_browser_reload_kind() {
        assert_eq!(classify(Path::new("a.html"), false), "html");
    }
}
