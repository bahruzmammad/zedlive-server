use axum::Router;
use axum_server::{Handle, tls_rustls::RustlsConfig};
use std::{path::PathBuf, time::Duration};
use tokio::net::TcpListener;

pub async fn serve(
    listener: TcpListener,
    app: Router,
    certificate: PathBuf,
    private_key: PathBuf,
) -> Result<(), String> {
    let tls = RustlsConfig::from_pem_file(certificate, private_key)
        .await
        .map_err(|e| e.to_string())?;
    let std_listener = listener.into_std().map_err(|e| e.to_string())?;
    let addr = std_listener.local_addr().map_err(|e| e.to_string())?;
    let handle = Handle::new();
    let shutdown = handle.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown.graceful_shutdown(Some(Duration::from_secs(5)));
    });
    axum_server::from_tcp_rustls(std_listener, tls)
        .map_err(|e| e.to_string())?
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .map_err(|e| format!("HTTPS server on {addr} failed: {e}"))
}
