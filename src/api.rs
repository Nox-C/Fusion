use actix_web::{get, post, web, HttpResponse, Responder, Error, HttpRequest};
use actix_web_actors::ws;
use crate::matrix::MatrixManager;
use std::sync::Arc;

#[get("/api/matrices")]
pub async fn get_matrices(data: web::Data<Arc<MatrixManager>>) -> impl Responder {
    let matrices = data.all();
    HttpResponse::Ok().json(matrices)
}

#[get("/api/scanning")]
pub async fn get_scanning() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"opportunities": []}))
}

#[get("/api/completed_transactions")]
pub async fn get_completed_transactions() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"transactions": []}))
}

#[get("/api/marginal_optimizer")]
pub async fn get_marginal_optimizer() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"marginal_optimizer": 0.0}))
}

#[post("/api/marginal_optimizer")]
pub async fn set_marginal_optimizer(payload: web::Json<serde_json::Value>) -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(payload.into_inner())
}

#[get("/api/liquidity")]
pub async fn get_liquidity() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"liquidity": 0.0}))
}

#[post("/api/liquidity")]
pub async fn set_liquidity(payload: web::Json<serde_json::Value>) -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(payload.into_inner())
}

#[get("/api/profit")]
pub async fn get_profit() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"profit": 0.0}))
}

#[post("/api/transfer")]
pub async fn post_transfer(_payload: web::Json<serde_json::Value>) -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

#[get("/api/wallet_status")]
pub async fn get_wallet_status() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"connected": false, "address": null}))
}

#[post("/api/connect_wallet")]
pub async fn post_connect_wallet() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"status": "connected"}))
}

#[get("/api/flashloan_providers")]
pub async fn get_flashloan_providers() -> impl Responder {
    // TODO: Implement real logic
    HttpResponse::Ok().json(serde_json::json!({"providers": []}))
}


// --- WebSocket Handler ---
pub async fn ws_matrices_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(crate::api::MatrixWs {}, &req, stream)
}

// Dummy actor for now, will be expanded for live updates
use actix::{Actor, StreamHandler, ActorContext};
use log;
pub struct MatrixWs;


impl Actor for MatrixWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Send welcome message on connect
        ctx.text("Welcome to Fusion Matrix WebSocket!");
        log::info!("WebSocket client connected");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::info!("WebSocket client disconnected");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MatrixWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // Optionally handle pong
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                // Optionally handle continuation
            }
            Ok(ws::Message::Nop) => {
                // No operation, do nothing
            }
            Err(e) => {
                log::error!("WebSocket error: {:?}", e);
                ctx.stop();
            }
        }
    }
}


