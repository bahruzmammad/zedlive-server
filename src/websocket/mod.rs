use crate::http::AppState;
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use std::str::FromStr;

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if !same_origin(&headers, state.config.https.enabled) {
        return StatusCode::FORBIDDEN.into_response();
    }
    ws.on_upgrade(move |socket| client(socket, state))
}

fn same_origin(headers: &HeaderMap, https: bool) -> bool {
    let Some(origin) = headers.get(header::ORIGIN) else {
        return true;
    };
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|host| host.to_str().ok())
    else {
        return false;
    };
    let Ok(origin) = Uri::from_str(origin.to_str().unwrap_or_default()) else {
        return false;
    };
    let scheme = if https { "https" } else { "http" };
    origin
        .scheme_str()
        .is_some_and(|origin_scheme| origin_scheme.eq_ignore_ascii_case(scheme))
        && origin
            .authority()
            .is_some_and(|authority| authority.as_str().eq_ignore_ascii_case(host))
}

async fn client(mut socket: WebSocket, state: AppState) {
    let mut receiver = state.reload_tx.subscribe();
    loop {
        tokio::select! {
            event = receiver.recv() => match event { Ok(event) => { let Ok(payload)=serde_json::to_string(&event) else{continue}; if socket.send(Message::Text(payload.into())).await.is_err(){break;} }, Err(tokio::sync::broadcast::error::RecvError::Lagged(_))=>continue, Err(tokio::sync::broadcast::error::RecvError::Closed)=>break },
            incoming = socket.recv() => match incoming { Some(Ok(Message::Ping(v))) => { let _=socket.send(Message::Pong(v)).await; }, Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break, Some(Ok(_)) => {} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(host: &str, origin: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, host.parse().unwrap());
        headers.insert(header::ORIGIN, origin.parse().unwrap());
        headers
    }

    #[test]
    fn allows_same_http_origin() {
        assert!(same_origin(
            &headers("localhost:5500", "http://localhost:5500"),
            false
        ));
    }

    #[test]
    fn rejects_different_host_or_scheme() {
        assert!(!same_origin(
            &headers("localhost:5500", "http://example.com"),
            false
        ));
        assert!(!same_origin(
            &headers("localhost:5500", "https://localhost:5500"),
            false
        ));
    }

    #[test]
    fn allows_clients_without_an_origin_header() {
        assert!(same_origin(&HeaderMap::new(), false));
    }
}
