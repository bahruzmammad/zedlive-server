use axum::Router;
use live_server::{
    config::{ServerConfig, WatchConfig},
    http::{AppState, router},
    reload::ReloadEvent,
    watcher,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
use tokio::net::TcpListener;

async fn spawn_fixture_server() -> (tokio::task::JoinHandle<()>, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("basic-site");
    spawn_server(root, "index.html").await
}

async fn spawn_server(root: PathBuf, index_file: &str) -> (tokio::task::JoinHandle<()>, String) {
    let config = ServerConfig {
        root,
        index_file: index_file.into(),
        open_browser: false,
        ..ServerConfig::default()
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app: Router = router(AppState::new(config));
    let url = format!("http://{address}");
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (task, url)
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "zedlive server test {}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

#[tokio::test]
async fn health_endpoint_works() {
    let (task, url) = spawn_fixture_server().await;
    let response = reqwest::get(format!("{url}/__live_server/health"))
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.text().await.unwrap().trim(), "ok");
    task.abort();
}

#[tokio::test]
async fn html_asset_is_served_and_reload_client_is_injected() {
    let (task, url) = spawn_fixture_server().await;
    let response = reqwest::get(format!("{url}/index.html")).await.unwrap();
    let body = response.text().await.unwrap();
    assert!(body.contains("Rust Live Server 0.0.1-alpha"));
    assert!(body.contains("/__live_server/live-reload.js"));
    task.abort();
}

#[tokio::test]
async fn watcher_emits_reload_events_for_frontend_edits() {
    use tokio::time::{Duration, timeout};

    let temp = TestDirectory::new();
    let root = temp.path().join("site");
    fs::create_dir_all(&root).unwrap();
    let (sender, mut receiver) = tokio::sync::broadcast::channel::<ReloadEvent>(16);
    let watcher = watcher::start(
        vec![(root.clone(), "/".into())],
        WatchConfig::default(),
        sender,
        false,
        0,
    )
    .await
    .unwrap()
    .unwrap();

    for (name, expected_kind) in [
        ("style.css", "css"),
        ("app.js", "js"),
        ("index.html", "html"),
    ] {
        fs::write(root.join(name), "updated").unwrap();
        let event = timeout(Duration::from_secs(5), async {
            loop {
                if let Ok(event) = receiver.recv().await
                    && event.path.ends_with(name)
                {
                    break event;
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(event.kind, expected_kind);
    }

    watcher.stop();
}

#[tokio::test]
async fn watcher_startup_failure_is_reported() {
    let temp = TestDirectory::new();
    let missing_root = temp.path().join("missing");
    let (sender, _) = tokio::sync::broadcast::channel::<ReloadEvent>(16);

    let error = match watcher::start(
        vec![(missing_root.clone(), "/".into())],
        WatchConfig::default(),
        sender,
        false,
        0,
    )
    .await
    {
        Err(error) => error,
        Ok(_) => panic!("watcher unexpectedly accepted a missing root"),
    };

    assert!(error.contains(&missing_root.display().to_string()));
}

#[tokio::test]
async fn missing_asset_returns_404() {
    let (task, url) = spawn_fixture_server().await;
    let response = reqwest::get(format!("{url}/missing-file.css"))
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    task.abort();
}

#[tokio::test]
async fn directory_index_uses_configured_relative_file() {
    let temp = TestDirectory::new();
    let root = temp.path().join("site");
    let products = root.join("products");
    fs::create_dir_all(&products).unwrap();
    fs::write(products.join("item.json"), r#"{"item":"served"}"#).unwrap();

    let (task, url) = spawn_server(root, "item.json").await;
    let response = reqwest::get(format!("{url}/products/")).await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("application/json")
    );
    assert_eq!(response.text().await.unwrap(), r#"{"item":"served"}"#);
    task.abort();
}

#[tokio::test]
async fn invalid_directory_index_cannot_escape_server_root() {
    let temp = TestDirectory::new();
    let root = temp.path().join("site");
    fs::create_dir(&root).unwrap();
    fs::write(temp.path().join("secret.html"), "outside the root").unwrap();

    let (task, url) = spawn_server(root, "../secret.html").await;
    let response = reqwest::get(format!("{url}/")).await.unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    assert!(!response.text().await.unwrap().contains("outside the root"));
    task.abort();
}

#[cfg(unix)]
#[tokio::test]
async fn directory_index_symlink_cannot_escape_server_root() {
    use std::os::unix::fs::symlink;

    let temp = TestDirectory::new();
    let root = temp.path().join("site");
    fs::create_dir(&root).unwrap();
    fs::write(temp.path().join("secret.html"), "outside the root").unwrap();
    symlink(temp.path().join("secret.html"), root.join("index.html")).unwrap();

    let (task, url) = spawn_server(root, "index.html").await;
    let response = reqwest::get(format!("{url}/")).await.unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    assert!(!response.text().await.unwrap().contains("outside the root"));
    task.abort();
}

#[cfg(unix)]
#[test]
fn cli_stop_gracefully_stops_server_in_a_workspace_with_spaces() {
    let temp = TestDirectory::new();
    fs::write(temp.path().join("index.html"), "<html>cli smoke</html>").unwrap();
    let binary = env!("CARGO_BIN_EXE_live-server");
    let mut server = ChildGuard(
        Command::new(binary)
            .args([
                "start",
                temp.path().to_str().unwrap(),
                "--port",
                "0",
                "--no-browser",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let state_path = temp.path().join(".live-server").join("server.json");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !state_path.is_file() && Instant::now() < deadline {
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "server exited during startup"
        );
        thread::sleep(Duration::from_millis(25));
    }
    assert!(state_path.is_file(), "server did not write state");

    let stop = Command::new(binary)
        .args(["stop", temp.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        stop.status.success(),
        "stop failed: {}",
        String::from_utf8_lossy(&stop.stderr)
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = server.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "server did not exit after stop");
        thread::sleep(Duration::from_millis(25));
    };
    assert!(
        status.success(),
        "server exited without graceful shutdown: {status}"
    );
    assert!(
        !state_path.exists(),
        "server state was not cleared on shutdown"
    );
}
