use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio::time::interval;

use events::EventBus;

use crate::messages::{ClientMessage, ServerMessage, SubscriptionFilter};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct WsState {
    pub event_bus: EventBus,
}

impl WsState {
    pub fn new(event_bus: EventBus) -> Self {
        Self { event_bus }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<WsState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut event_rx = state.event_bus.subscribe();
    let mut filter: Option<SubscriptionFilter> = None;
    let mut subscribed = false;

    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    heartbeat.reset();

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let ping_msg = serde_json::to_string(&ServerMessage::Pong).unwrap();
                if sender.send(Message::Text(ping_msg.into())).await.is_err() {
                    break;
                }
            }

            event_result = event_rx.recv() => {
                match event_result {
                    Ok(envelope) => {
                        if subscribed {
                            let should_send = filter.as_ref()
                                .map(|f| f.matches(&envelope))
                                .unwrap_or(true);

                            if should_send {
                                let msg = ServerMessage::Event { envelope };
                                let json = serde_json::to_string(&msg).unwrap();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged, missed {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            msg = tokio::time::timeout(CLIENT_TIMEOUT + HEARTBEAT_INTERVAL, receiver.next()) => {
                match msg {
                    Ok(Some(Ok(Message::Text(text)))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Subscribe { filter: new_filter }) => {
                                filter = new_filter.clone();
                                subscribed = true;
                                let response = ServerMessage::Subscribed { filter: new_filter };
                                let json = serde_json::to_string(&response).unwrap();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ClientMessage::Unsubscribe) => {
                                subscribed = false;
                                filter = None;
                                let response = ServerMessage::Unsubscribed;
                                let json = serde_json::to_string(&response).unwrap();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ClientMessage::Ping) => {
                                let response = ServerMessage::Pong;
                                let json = serde_json::to_string(&response).unwrap();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let response = ServerMessage::Error {
                                    message: format!("Invalid message: {}", e),
                                };
                                let json = serde_json::to_string(&response).unwrap();
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Ok(Some(Ok(Message::Close(_)))) => {
                        break;
                    }
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Ok(Some(Ok(_))) => {}
                    Ok(Some(Err(_))) => {
                        break;
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
                        tracing::debug!("WebSocket client timeout, sending ping");
                    }
                }
            }
        }
    }

    tracing::debug!("WebSocket connection closed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ws_state_creation() {
        let bus = EventBus::new();
        let state = WsState::new(bus);
        assert_eq!(state.event_bus.subscriber_count(), 0);
    }

    #[test]
    fn test_heartbeat_interval() {
        assert_eq!(HEARTBEAT_INTERVAL, Duration::from_secs(30));
    }

    #[test]
    fn test_client_timeout() {
        assert_eq!(CLIENT_TIMEOUT, Duration::from_secs(10));
    }
}
