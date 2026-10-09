use std::path::{Path, PathBuf};

pub fn entry_path(root: &Path, entry_file: &str) -> PathBuf {
    root.join(entry_file)
}
pub fn should_fallback(path: &Path, accepts_html: bool, enabled: bool) -> bool {
    if !enabled || !accepts_html || path.exists() {
        return false;
    }
    path.extension().is_none()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn html_accepts_spa_fallback() {
        let p = PathBuf::from("/missing/route");
        assert!(should_fallback(&p, true, true));
        assert!(!should_fallback(&p, false, true));
        assert!(!should_fallback(&PathBuf::from("/app.js"), true, true));
    }
}
