use crate::config::Browser;
use std::{
    env,
    path::Path,
    process::{Command, Stdio},
};

pub fn open_url(browser: &Browser, custom_command: Option<&str>, url: &str) -> Result<(), String> {
    if let Some(command) = custom_command.filter(|v| !v.trim().is_empty()) {
        return run_custom(command, url);
    }
    match browser {
        Browser::Default => open_default(url),
        Browser::Chrome => run_named_browser("Chrome", chrome_candidates(), url),
        Browser::Edge => run_named_browser("Edge", edge_candidates(), url),
        Browser::Firefox => run_named_browser("Firefox", firefox_candidates(), url),
        Browser::Custom { program, args } => {
            let mut cmd = Command::new(program);
            for arg in args {
                cmd.arg(arg.replace("{url}", url));
            }
            cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
        }
    }
}

fn run_custom(command: &str, url: &str) -> Result<(), String> {
    let expanded = command.replace("{url}", url);
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", &expanded])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        Command::new("sh")
            .args(["-c", &expanded])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
fn run_named_browser(name: &str, candidates: Vec<String>, url: &str) -> Result<(), String> {
    for candidate in candidates {
        let path_exists =
            candidate.contains(std::path::MAIN_SEPARATOR) && Path::new(&candidate).is_file();
        if path_exists || !candidate.contains(std::path::MAIN_SEPARATOR) {
            if Command::new(&candidate).arg(url).spawn().is_ok() {
                return Ok(());
            }
        }
    }
    open_default(url).map_err(|e| format!("{name} not found; default browser failed: {e}"))
}
fn open_default(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
#[cfg(windows)]
fn chrome_candidates() -> Vec<String> {
    let mut v = Vec::new();
    if let Some(x) = env::var_os("LOCALAPPDATA") {
        v.push(format!(
            r"{}\Google\Chrome\Application\chrome.exe",
            x.to_string_lossy()
        ));
    }
    if let Some(x) = env::var_os("PROGRAMFILES") {
        v.push(format!(
            r"{}\Google\Chrome\Application\chrome.exe",
            x.to_string_lossy()
        ));
    }
    v.push("chrome".into());
    v
}
#[cfg(not(windows))]
fn chrome_candidates() -> Vec<String> {
    vec![
        "google-chrome".into(),
        "chromium".into(),
        "chromium-browser".into(),
    ]
}
#[cfg(windows)]
fn edge_candidates() -> Vec<String> {
    let mut v = Vec::new();
    for key in ["PROGRAMFILES(X86)", "PROGRAMFILES"] {
        if let Some(x) = env::var_os(key) {
            v.push(format!(
                r"{}\Microsoft\Edge\Application\msedge.exe",
                x.to_string_lossy()
            ));
        }
    }
    v.push("msedge".into());
    v
}
#[cfg(not(windows))]
fn edge_candidates() -> Vec<String> {
    vec!["microsoft-edge".into(), "microsoft-edge-stable".into()]
}
#[cfg(windows)]
fn firefox_candidates() -> Vec<String> {
    let mut v = Vec::new();
    for key in ["PROGRAMFILES", "PROGRAMFILES(X86)"] {
        if let Some(x) = env::var_os(key) {
            v.push(format!(
                r"{}\Mozilla Firefox\firefox.exe",
                x.to_string_lossy()
            ));
        }
    }
    v.push("firefox".into());
    v
}
#[cfg(not(windows))]
fn firefox_candidates() -> Vec<String> {
    vec!["firefox".into()]
}
