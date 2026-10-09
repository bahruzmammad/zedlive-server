use axum::Router;
use live_server::{
    config::ServerConfig,
    http::{AppState, router},
};
use tokio::net::TcpListener;

async fn spawn_fixture_server() -> (tokio::task::JoinHandle<()>, String) {
    let mut config = ServerConfig::default();
    config.root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("basic-site");
    config.open_browser = false;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app: Router = router(AppState::new(config));
    let url = format!("http://{address}");
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (task, url)
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
async fn missing_asset_returns_404() {
    let (task, url) = spawn_fixture_server().await;
    let response = reqwest::get(format!("{url}/missing-file.css"))
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    task.abort();
}
