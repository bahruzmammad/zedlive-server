use percent_encoding::percent_decode_str;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum SecurityError {
    InvalidEncoding,
    Traversal,
    OutsideRoot,
    Io(String),
}
impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEncoding => write!(f, "invalid URL path encoding"),
            Self::Traversal => write!(f, "path traversal denied"),
            Self::OutsideRoot => write!(f, "requested path is outside the server root"),
            Self::Io(e) => write!(f, "filesystem error: {e}"),
        }
    }
}
impl std::error::Error for SecurityError {}

pub fn normalize_request_path(raw: &str) -> Result<String, SecurityError> {
    let mut value = raw.to_owned();
    for _ in 0..3 {
        let decoded = percent_decode_str(&value)
            .decode_utf8()
            .map_err(|_| SecurityError::InvalidEncoding)?
            .into_owned();
        if decoded == value {
            value = decoded;
            break;
        }
        value = decoded;
    }
    let decoded = value.replace('\\', "/");
    if decoded.contains('\0') {
        return Err(SecurityError::Traversal);
    }
    let mut parts = Vec::new();
    for part in decoded.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(':') {
            return Err(SecurityError::Traversal);
        }
        parts.push(part);
    }
    if parts.is_empty() {
        Ok("/".into())
    } else {
        Ok(format!("/{}", parts.join("/")))
    }
}

pub fn safe_join(root: &Path, request_path: &str) -> Result<PathBuf, SecurityError> {
    let normalized = normalize_request_path(request_path)?;
    let root = std::fs::canonicalize(root).map_err(|e| SecurityError::Io(e.to_string()))?;
    let candidate = root.join(normalized.trim_start_matches('/'));
    if candidate.exists() {
        let canonical =
            std::fs::canonicalize(&candidate).map_err(|e| SecurityError::Io(e.to_string()))?;
        if !inside(&root, &canonical) {
            return Err(SecurityError::OutsideRoot);
        }
        Ok(canonical)
    } else {
        if !inside(&root, &candidate) {
            return Err(SecurityError::OutsideRoot);
        }
        Ok(candidate)
    }
}
fn inside(root: &Path, candidate: &Path) -> bool {
    candidate == root || candidate.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn root_path_is_normalized() {
        assert_eq!(normalize_request_path("/"), Ok("/".into()));
    }
    #[test]
    fn encoded_traversal_is_denied() {
        assert!(normalize_request_path("/..%2FCargo.toml").is_err());
        assert!(normalize_request_path("/%252e%252e/Cargo.toml").is_err());
    }
}
