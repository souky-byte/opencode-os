use std::sync::Arc;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::State;
use axum::response::IntoResponse;

use websocket::WsState;

use crate::state::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ws_state = Arc::new(WsState::new(state.event_bus.clone()));
    websocket::ws_handler(ws, State(ws_state)).await
}
