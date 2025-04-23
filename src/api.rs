use crate::matrix::MatrixManager;
use crate::providers::ProviderManager;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, post, web};
use actix_web_actors::ws;
use std::sync::Arc;

use ethers::middleware::Middleware;
use ethers::signers::Signer;
use ethers::types::{Address, TransactionRequest};
use ethers::utils::parse_ether;
use std::time::Duration;

#[get("/api/matrices")]
pub async fn get_matrices(data: web::Data<Arc<MatrixManager>>) -> impl Responder {
    let matrices = data.all();
    HttpResponse::Ok().json(matrices)
}

// ... (rest of the code remains the same)

#[get("/api/completed_transactions")]
pub async fn get_completed_transactions(data: web::Data<Arc<MatrixManager>>) -> impl Responder {
    let matrices = data.all();
    let mut transactions = Vec::new();
    for matrix in matrices {
        transactions.extend(matrix.recent_transactions.clone());
    }
    HttpResponse::Ok().json(transactions)
}

#[post("/api/transfer")]
pub async fn post_transfer(
    payload: web::Json<serde_json::Value>,
    provider_data: web::Data<Arc<ProviderManager>>,
) -> impl Responder {
    let to = match payload.get("to").and_then(|v| v.as_str()) {
        Some(addr) => addr,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"status": "error", "reason": "Missing 'to' address"}));
        }
    };
    let amount = match payload.get("amount").and_then(|v| v.as_f64()) {
        Some(val) if val > 0.0 => val,
        _ => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"status": "error", "reason": "Invalid or missing 'amount'"}),
            );
        }
    };
    let chain = match payload.get("chain").and_then(|v| v.as_str()) {
        Some(c) => c.to_uppercase(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({"status": "error", "reason": "Missing 'chain' parameter (must be 'BSC' or 'ETH')"}));
        }
    };
    let to_addr = match to.parse::<Address>() {
        Ok(addr) => addr,
        Err(_) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"status": "error", "reason": "Invalid 'to' address format"}),
            );
        }
    };
    let provider = match chain.as_str() {
        "BSC" => {
            match &provider_data.bsc_provider {
                Some(p) => p.http_provider.clone(),
                None => {
                    log::error!("BSC provider not configured");
                    return HttpResponse::InternalServerError().json(serde_json::json!({"status": "error", "reason": "BSC provider not configured"}));
                }
            }
        }
        "ETH" => {
            match &provider_data.eth_provider {
                Some(p) => p.http_provider.clone(),
                None => {
                    log::error!("ETH provider not configured");
                    return HttpResponse::InternalServerError().json(serde_json::json!({"status": "error", "reason": "ETH provider not configured"}));
                }
            }
        }
        _ => {
            log::error!("Unsupported chain: {}", chain);
            return HttpResponse::BadRequest().json(serde_json::json!({"status": "error", "reason": "Unsupported chain. Must be 'BSC' or 'ETH'"}));
        }
    };
    let from_addr = provider.address();
    let value = match parse_ether(amount) {
        Ok(val) => val,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"status": "error", "reason": "Invalid 'amount' value"}));
        }
    };
    let tx = TransactionRequest::pay(to_addr, value);
    let send_result = provider.send_transaction(tx, None).await;
    match send_result {
        Ok(pending) => match pending.await {
            Ok(Some(receipt)) => {
                return HttpResponse::Ok().json(serde_json::json!({"status": "success", "tx_hash": format!("0x{:x}", receipt.transaction_hash)}));
            }
            Ok(None) => {
                log::error!(
                    "Transaction pending or dropped for transfer from {} to {} amount {}",
                    from_addr,
                    to_addr,
                    amount
                );
                return HttpResponse::InternalServerError().json(serde_json::json!({"status": "error", "reason": "Transaction pending or dropped"}));
            }
            Err(e) => {
                log::error!("Error waiting for transaction receipt: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({"status": "error", "reason": "Failed to get transaction receipt"}));
            }
        },
        Err(e) => {
            log::error!("Error sending transaction: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"status": "error", "reason": "Failed to send transaction"}),
            );
        }
    }
}

#[get("/api/wallet_status")]
pub async fn get_wallet_status(provider_data: web::Data<Arc<ProviderManager>>) -> impl Responder {
    let address = provider_data.get_wallet().address();
    HttpResponse::Ok()
        .json(serde_json::json!({"connected": true, "address": format!("0x{:x}", address)}))
}

#[post("/api/connect_wallet")]
pub async fn post_connect_wallet(provider_data: web::Data<Arc<ProviderManager>>) -> impl Responder {
    let address = provider_data.get_wallet().address();
    HttpResponse::Ok()
        .json(serde_json::json!({"status": "connected", "address": format!("0x{:x}", address)}))
}

// --- WebSocket Handler ---
pub async fn ws_matrices_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(crate::api::MatrixWs {}, &req, stream)
}

use actix::{Actor, ActorContext, StreamHandler};
pub struct MatrixWs;

use actix::prelude::*;
use serde_json::json;

impl Actor for MatrixWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text("Welcome to Fusion Matrix WebSocket!");
        ctx.run_interval(Duration::from_secs(5), |_act, ctx| {
            // Simulate live matrix update
            let now = chrono::Utc::now().to_rfc3339();
            ctx.text(
                json!({"type": "matrix_update", "timestamp": now, "message": "Live matrix update"})
                    .to_string(),
            );
        });
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
                // Optionally handle commands from the client
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {}
            Ok(ws::Message::Nop) => {}
            Err(e) => {
                log::error!("WebSocket error: {:?}", e);
                ctx.stop();
            }
        }
    }
}
