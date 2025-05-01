use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde_json::json;
use actix::ActorContext;
use std::time::Instant;
use uuid::Uuid;
use chrono::Utc;
use log::error;
use actix::AsyncContext;
use tokio::sync::broadcast::Receiver;
use crate::events::{WebSocketEvent, WebSocketEventReceiver};

pub struct Matrix2dWs {
    last_ping: Instant,
    event_rx: WebSocketEventReceiver,
}

impl Matrix2dWs {
    pub fn new(event_rx: WebSocketEventReceiver) -> Self {
        Self {
            last_ping: Instant::now(),
            event_rx,
        }
    }
}

impl Actor for Matrix2dWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket connection established");
        ctx.text(r#"{"type": "welcome", "message": "Connected to Fusion WebSocket server"}"#);
        self.last_ping = Instant::now();
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Matrix2dWs {
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start event listener
        ctx.spawn(
            async move {
                while let Ok(event) = self.event_rx.recv().await {
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            ctx.text(json);
                        }
                        Err(e) => {
                            error!("Failed to serialize event to JSON: {}", e);
                        }
                    }
                }
            }
            .into_actor(self)
        );

        // Send heartbeat every 10 seconds
        ctx.run_interval(std::time::Duration::from_secs(10), |act, ctx| {
            if act.last_ping.elapsed() > std::time::Duration::from_secs(60) {
                ctx.stop();
                return;
            }
            ctx.ping(b"ping");
        });
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                self.last_ping = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                log::info!("Received text message: {}", text);
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => match json.get("type").and_then(|t| t.as_str()) {
                        Some("dex") => {
                            if let Some(dex) = json.get("dex").and_then(|d| d.as_str()) {
                                let id = Uuid::new().unwrap().to_string();
                                let timestamp = json.get("timestamp").and_then(|t| t.as_str()).unwrap_or_else(|| 
                                    Utc::now().to_rfc3339().as_str()
                                );
                                let message = json.get("message").and_then(|m| m.as_str()).unwrap_or("No message provided");
                                let status = json.get("status").and_then(|s| s.as_str()).unwrap_or("active");
                                let response = json!({
                                    "type": "dex",
                                    "id": id,
                                    "timestamp": timestamp,
                                    "dex": dex,
                                    "message": message,
                                    "status": status,
                                    "created_at": Utc::now().to_rfc3339()
                                });
                                ctx.text(response.to_string());
                            } else {
                                let error = json!({
                                    "type": "error",
                                    "id": Uuid::new().unwrap().to_string(),
                                    "timestamp": Utc::now().to_rfc3339(),
                                    "error": "Missing required 'dex' field in DEX event"
                                });
                                ctx.text(error.to_string());
                            }
                        }
                        Some("liquidation") => {
                            if let Some(account) = json.get("account").and_then(|a| a.as_str()) {
                                let id = Uuid::new().unwrap().to_string();
                                let timestamp = json.get("timestamp").and_then(|t| t.as_str()).unwrap_or_else(|| 
                                    Utc::now().to_rfc3339().as_str()
                                );
                                let status = json.get("status").and_then(|s| s.as_str()).unwrap_or("active");
                                let details = json.get("details").and_then(|d| d.as_str()).unwrap_or("No details provided");
                                let response = json!({
                                    "type": "liquidation",
                                    "id": id,
                                    "timestamp": timestamp,
                                    "account": account,
                                    "status": status,
                                    "details": details,
                                    "created_at": Utc::now().to_rfc3339()
                                });
                                if let Err(e) = ctx.text(response.to_string()) {
                                    log::error!("Failed to send liquidation event response: {}", e);
                                } else {
                                    ctx.text(response.to_string());
                                }
                            } else {
                                let error = json!({
                                    "type": "error",
                                    "id": Uuid::new().unwrap().to_string(),
                                    "timestamp": Utc::now().to_rfc3339(),
                                    "error": "Missing required 'account' field in liquidation event"
                                });
                                if let Err(e) = ctx.text(error.to_string()) {
                                    error!("Failed to send error response: {}", e);
                                }
                            }
                        }
                        Some(unknown_type) => {
                            ctx.text(format!(r#"{{"error": "Unknown event type: {}"}}"#, unknown_type));
                        }
                        None => {
                            ctx.text(r#"{"error": "Missing 'type' field in event"}"#);
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to parse JSON: {}", e);
                        ctx.text(format!(r#"{{"error": "Invalid JSON: {}"}}"#, e));
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("WebSocket connection closed: {:?}", reason);
                ctx.stop();
            }
            Err(e) => {
                log::error!("WebSocket protocol error: {}", e);
                ctx.stop();
            }
            _ => (),
        }
    }
}

pub async fn ws_matrix2d_handler(
    req: HttpRequest,
    stream: web::Payload,
    event_sender: web::Data<tokio::sync::broadcast::Sender<WebSocketEvent>>,
) -> Result<HttpResponse, Error> {
    log::info!("WebSocket connection attempt from: {}", req.connection_info().realip_remote_addr().unwrap_or("unknown"));
    log::info!("Request headers: {:#?}", req.headers());
    
    let (tx, rx) = event_sender.subscribe();
    let response = ws::start(
        Matrix2dWs::new(WebSocketEventReceiver::new(rx)),
        &req,
        stream,
    )?;
    
    log::info!("WebSocket connection response sent");
    Ok(response)
}
