use crate::{config::WatchConfig, reload};
use notify::{EventKind, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use tokio::sync::broadcast;

pub struct WatcherHandle {
    stop: Arc<AtomicBool>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl WatcherHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Drop for WatcherHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.join.take();
    }
}

pub fn start(
    roots: Vec<(PathBuf, String)>,
    config: WatchConfig,
    sender: broadcast::Sender<reload::ReloadEvent>,
    full_reload: bool,
    wait_ms: u64,
) -> Option<WatcherHandle> {
    if !config.enabled {
        return None;
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = Arc::clone(&stop);

    let join = tokio::task::spawn_blocking(move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(move |result| {
            let _ = tx.send(result);
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                eprintln!("watcher: {error}");
                return;
            }
        };

        let mode = if config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        for (root, _) in &roots {
            if let Err(error) = watcher.watch(root, mode) {
                eprintln!("watcher: cannot watch {}: {error}", root.display());
            }
        }

        let debounce = Duration::from_millis(wait_ms);
        let mut last_seen: HashMap<PathBuf, Instant> = HashMap::new();

        while !stop_thread.load(Ordering::SeqCst) {
            let result = match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(result) => result,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            };

            let event = match result {
                Ok(event) => event,
                Err(error) => {
                    eprintln!("watcher: {error}");
                    continue;
                }
            };

            if !matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            ) {
                continue;
            }

            for path in event.paths {
                if ignored(&path, &config.ignore) || !interesting(&path, &config.extensions) {
                    continue;
                }

                let now = Instant::now();
                if let Some(previous) = last_seen.get(&path)
                    && now.duration_since(*previous) < debounce
                {
                    continue;
                }
                last_seen.insert(path.clone(), now);

                let Some(virtual_path) = virtual_path_for(&roots, &path) else {
                    continue;
                };

                let kind = reload::classify(&path, full_reload);
                let event = reload::event(&virtual_path, full_reload, kind);
                let _ = sender.send(event);
            }
        }
    });

    Some(WatcherHandle {
        stop,
        join: Some(join),
    })
}

fn interesting(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            extensions
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(extension))
        })
}

fn ignored(path: &Path, patterns: &[String]) -> bool {
    let text = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    patterns.iter().any(|pattern| {
        let pattern = pattern.replace('\\', "/").to_ascii_lowercase();
        if pattern.ends_with("/**") {
            text.contains(pattern.trim_end_matches("/**"))
        } else if pattern.contains('*') {
            wildcard_contains(&text, &pattern)
        } else {
            text.split('/').any(|segment| segment == pattern)
                || text.contains(&format!("/{pattern}/"))
                || text.ends_with(&format!("/{pattern}"))
        }
    })
}

fn wildcard_contains(text: &str, pattern: &str) -> bool {
    let pieces: Vec<&str> = pattern
        .split('*')
        .filter(|piece| !piece.is_empty())
        .collect();
    if pieces.is_empty() {
        return true;
    }

    let mut offset = 0;
    for piece in pieces {
        let Some(position) = text[offset..].find(piece) else {
            return false;
        };
        offset += position + piece.len();
    }
    true
}

fn virtual_path_for(roots: &[(PathBuf, String)], path: &Path) -> Option<String> {
    for (root, prefix) in roots {
        let root_abs = std::fs::canonicalize(root).ok()?;
        let path_abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if path_abs.starts_with(&root_abs) {
            let rel = path_abs.strip_prefix(&root_abs).ok()?;
            let rel = rel.to_string_lossy().replace('\\', "/");
            return if rel.is_empty() {
                Some(prefix.clone())
            } else if prefix == "/" {
                Some(format!("/{rel}"))
            } else {
                Some(format!("{}/{}", prefix.trim_end_matches('/'), rel))
            };
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_ignore_works() {
        assert!(wildcard_contains("C:/site/node_modules/a.js", "node_*"));
        assert!(!wildcard_contains("C:/site/src/a.js", "node_*"));
    }
}
