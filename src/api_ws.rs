use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use actix::ActorContext;
use std::time::Instant;
use log::{error, info};
use actix::fut;

use crate::events::{WebSocketEvent};

pub struct Matrix2dWs {
    last_ping: Instant,
    event_rx: Receiver<WebSocketEvent>,
}

impl Matrix2dWs {
    pub fn new(event_rx: Receiver<WebSocketEvent>) -> Self {
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
        ctx.run_interval(std::time::Duration::from_secs(1), move |act, ctx| {
            act.event_rx.recv().then(|result| {
                match result {
                    Ok(event) => match serde_json::to_string(&event) {
                        Ok(json) => {
                            if let Err(e) = ctx.text(json) {
                                error!("Failed to send event to client: {}", e);
                                ctx.stop();
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize event: {}", e);
                            ctx.stop();
                        }
                    },
                    Err(e) => {
                        error!("Failed to receive event: {}", e);
                        ctx.stop();
                    }
                }
                actix::fut::ready(())
            })
        });

        // Send heartbeat every 10 seconds
        ctx.run_interval(std::time::Duration::from_secs(10), |_, ctx| {
            ctx.ping(b"ping").then(|result| {
                if let Err(e) = result {
                    error!("Failed to send ping: {}", e);
                    ctx.stop();
                }
                actix::fut::ready(())
            })
        });
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                self.last_ping = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                info!("Received text message: {}", text);
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket connection closed: {:?}", reason);
                ctx.stop();
            }
            Err(e) => {
                error!("WebSocket protocol error: {}", e);
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
    
    let rx = event_sender.subscribe();
    let response = ws::start(
        Matrix2dWs::new(rx),
        &req,
        stream,
    )?;
    
    log::info!("WebSocket connection response sent");
    Ok(response)
}
