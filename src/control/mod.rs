use crate::browser;
use crate::config::Browser;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::process::Stdio;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerState {
    pub version: String,
    pub pid: u32,
    pub root: String,
    pub url: String,
    pub host: String,
    pub port: u16,
    pub https: bool,
    pub started_at: String,
}

fn normalized_root(root: &Path) -> PathBuf {
    fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf())
}

pub fn control_dir(root: &Path) -> PathBuf {
    normalized_root(root).join(".live-server")
}

pub fn state_path(root: &Path) -> PathBuf {
    control_dir(root).join("server.json")
}

pub fn pid_path(root: &Path) -> PathBuf {
    control_dir(root).join("server.pid")
}

pub fn write_state(root: &Path, state: &ServerState) -> Result<(), String> {
    fs::create_dir_all(control_dir(root)).map_err(|e| e.to_string())?;
    let json = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    fs::write(state_path(root), json).map_err(|e| e.to_string())?;
    fs::write(pid_path(root), state.pid.to_string()).map_err(|e| e.to_string())
}

pub fn read_state(root: &Path) -> Result<ServerState, String> {
    let path = state_path(root);
    let bytes = fs::read(&path)
        .map_err(|_| format!("no Live Server state found for {}", root.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid server state: {e}"))
}

pub fn clear_state(root: &Path) {
    let _ = fs::remove_file(state_path(root));
    let _ = fs::remove_file(pid_path(root));
    let _ = fs::remove_dir(control_dir(root));
}

pub fn ensure_not_running(root: &Path) -> Result<(), String> {
    match read_state(root) {
        Ok(state) if process_exists(state.pid) => Err(format!(
            "Live Server is already running (PID {}, {}). Use `live-server stop`.",
            state.pid, state.url
        )),
        Ok(state) => {
            eprintln!("removing stale Live Server state for PID {}", state.pid);
            clear_state(root);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

pub fn process_exists(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let filter = format!("PID eq {pid}");
        Command::new("tasklist")
            .args(["/FI", &filter, "/FO", "CSV", "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&format!("\"{pid}\"")))
            .unwrap_or(false)
    }
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .is_ok_and(|s| s.success())
    }
    #[cfg(not(any(windows, unix)))]
    {
        false
    }
}

pub fn stop(root: &Path) -> Result<String, String> {
    let state = match read_state(root) {
        Ok(state) => state,
        Err(_) => return Ok("Live Server is not running.".into()),
    };
    if !process_exists(state.pid) {
        clear_state(root);
        return Ok(format!("stale server state removed for PID {}", state.pid));
    }

    terminate(state.pid)?;
    for _ in 0..25 {
        if !process_exists(state.pid) {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    clear_state(root);
    Ok(format!("Live Server stopped (PID {})", state.pid))
}

fn terminate(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "taskkill failed for PID {pid}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
    #[cfg(unix)]
    {
        let status = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("kill failed for PID {pid}"))
        }
    }
    #[cfg(not(any(windows, unix)))]
    {
        Err("process stop is unsupported on this platform".into())
    }
}

pub fn open(root: &Path, browser_override: Option<&Browser>) -> Result<String, String> {
    let state = read_state(root)?;
    if !process_exists(state.pid) {
        return Err("Live Server process is not running".into());
    }

    let browser = browser_override.unwrap_or(&Browser::Default);
    browser::open_url(browser, None, &state.url)?;
    Ok(format!("opened {}", state.url))
}

pub fn status(root: &Path) -> Result<(), String> {
    match read_state(root) {
        Ok(state) => {
            let running = process_exists(state.pid);
            println!("Live Server status");
            println!("  State: {}", if running { "RUNNING" } else { "STALE" });
            println!("  PID:   {}", state.pid);
            println!("  URL:   {}", state.url);
            println!("  Root:  {}", state.root);
            if !running {
                clear_state(root);
            }
            Ok(())
        }
        Err(_) => {
            println!("Live Server status: STOPPED");
            Ok(())
        }
    }
}

pub fn current_pid() -> u32 {
    std::process::id()
}
