use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketEvent {
    Dex(DexEvent),
    Liquidation(LiquidationEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexEvent {
    pub id: String,
    pub timestamp: String,
    pub dex: String,
    pub message: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationEvent {
    pub id: String,
    pub timestamp: String,
    pub account: String,
    pub status: String,
    pub details: String,
    pub created_at: String,
}

pub struct WebSocketEventSender {
    tx: tokio::sync::broadcast::Sender<WebSocketEvent>,
}

impl WebSocketEventSender {
    pub fn new(tx: tokio::sync::broadcast::Sender<WebSocketEvent>) -> Self {
        Self { tx }
    }

    pub fn send_dex_event(&self, dex: &str, message: &str, status: &str) {
        let event = WebSocketEvent::Dex(DexEvent {
            id: Uuid::new().unwrap().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            dex: dex.to_string(),
            message: message.to_string(),
            status: status.to_string(),
            created_at: Utc::now().to_rfc3339(),
        });
        let _ = self.tx.send(event);
    }

    pub fn send_liquidation_event(&self, account: &str, details: &str, status: &str) {
        let event = WebSocketEvent::Liquidation(LiquidationEvent {
            id: Uuid::new().unwrap().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            account: account.to_string(),
            status: status.to_string(),
            details: details.to_string(),
            created_at: Utc::now().to_rfc3339(),
        });
        let _ = self.tx.send(event);
    }
}

pub struct WebSocketEventReceiver {
    rx: tokio::sync::broadcast::Receiver<WebSocketEvent>,
}

impl WebSocketEventReceiver {
    pub fn new(rx: tokio::sync::broadcast::Receiver<WebSocketEvent>) -> Self {
        Self { rx }
    }

    pub async fn recv(&mut self) -> Result<WebSocketEvent, tokio::sync::broadcast::error::RecvError> {
        self.rx.recv().await
    }
}
