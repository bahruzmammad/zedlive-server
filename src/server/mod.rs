use crate::{browser, cli::RunOptions, control, http, https, watcher};
use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

pub async fn run(options: RunOptions) -> Result<(), String> {
    let config = options.into_config()?;
    config.validate().map_err(|error| error.to_string())?;

    let root = config
        .normalized_root()
        .map_err(|error| error.to_string())?;

    control::ensure_not_running(&root)?;

    let (listener, address) = http::bind_listener(config.host, config.port, config.port_mode)
        .await
        .map_err(|error| format!("cannot bind {}:{}: {error}", config.host, config.port))?;

    let scheme = if config.https.enabled {
        "https"
    } else {
        "http"
    };
    let host = config
        .preferred_hostname
        .clone()
        .unwrap_or_else(|| browser_host(config.host));
    let url = format!("{scheme}://{host}:{}/", address.port());

    let mut runtime_config = config.clone();
    runtime_config.root = root.clone();
    runtime_config.port = address.port();

    let state = http::AppState::new(runtime_config.clone());
    let app = http::router(state.clone());
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();

    control::write_state(
        &root,
        &control::ServerState {
            version: crate::VERSION.into(),
            pid: control::current_pid(),
            root: root.display().to_string(),
            url: url.clone(),
            host: config.host.to_string(),
            port: address.port(),
            https: config.https.enabled,
            started_at,
        },
    )?;

    println!("{}", runtime_config.summary(&url, address.port()));
    println!("Control: live-server stop {}", root.display());
    println!("Press Ctrl+C to stop.");

    if runtime_config.open_browser
        && let Err(error) = browser::open_url(
            &runtime_config.browser,
            runtime_config.browser_command.as_deref(),
            &url,
        )
    {
        eprintln!("browser: {error}");
    }

    let _watcher = watcher::start(
        runtime_config.watched_roots(),
        runtime_config.watch.clone(),
        state.reload_tx.clone(),
        runtime_config.reload.full_reload,
        runtime_config.reload.wait_ms,
    );

    let result = if runtime_config.https.enabled {
        https::serve(
            listener,
            app,
            runtime_config
                .https
                .certificate
                .clone()
                .ok_or("missing HTTPS certificate")?,
            runtime_config
                .https
                .private_key
                .clone()
                .ok_or("missing HTTPS private key")?,
        )
        .await
    } else {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|error| format!("HTTP server failed: {error}"))
    };

    control::clear_state(&root);
    result
}

pub async fn restart(options: RunOptions) -> Result<(), String> {
    let root = options
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let root = std::fs::canonicalize(root).map_err(|error| error.to_string())?;

    let _ = control::stop(&root);
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    run(options).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    tokio::select! {
        _ = ctrl_c => {}
    }
}

fn browser_host(host: IpAddr) -> String {
    match host {
        IpAddr::V4(value) if value.is_unspecified() => "127.0.0.1".into(),
        IpAddr::V6(value) if value.is_unspecified() => "[::1]".into(),
        IpAddr::V4(value) => value.to_string(),
        IpAddr::V6(value) => format!("[{value}]"),
    }
}
