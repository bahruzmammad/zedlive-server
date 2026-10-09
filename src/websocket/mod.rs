use crate::http::AppState;
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};

pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| client(socket, state))
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
