use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::error::{OpenCodeError, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenCodeEvent {
    #[serde(rename = "session.message")]
    SessionMessage { session_id: String, content: String },

    #[serde(rename = "session.completed")]
    SessionCompleted { session_id: String },

    #[serde(rename = "session.error")]
    SessionError { session_id: String, error: String },

    #[serde(rename = "task.status_changed")]
    TaskStatusChanged { task_id: String, status: String },

    #[serde(other)]
    Unknown,
}

pub struct EventStream {
    base_url: String,
    client: reqwest::Client,
}

impl EventStream {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn connect(&self) -> Result<EventReceiver> {
        let url = format!("{}/event", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .map_err(OpenCodeError::Request)?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Connection(format!(
                "Failed to connect to event stream: {}",
                response.status()
            )));
        }

        let (tx, rx) = mpsc::channel::<Result<OpenCodeEvent>>(100);

        let byte_stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut event_stream = byte_stream.eventsource();

            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        if event.data.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<OpenCodeEvent>(&event.data) {
                            Ok(parsed) => {
                                if tx.send(Ok(parsed)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to parse event: {} - data: {}",
                                    e,
                                    event.data
                                );
                            }
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("{}", e);
                        let _ = tx.send(Err(OpenCodeError::EventStream(err_msg))).await;
                        break;
                    }
                }
            }
        });

        Ok(EventReceiver { rx })
    }
}

pub struct EventReceiver {
    rx: mpsc::Receiver<Result<OpenCodeEvent>>,
}

impl EventReceiver {
    pub async fn next_event(&mut self) -> Option<Result<OpenCodeEvent>> {
        self.rx.recv().await
    }
}
