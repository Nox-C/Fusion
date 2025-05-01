use crate::matrix2d::Matrix2D;
use std::sync::{Arc, Mutex};
use crate::providers::ProviderManager;
use actix_web::{HttpResponse, Responder, get, post, web};
use std::time::SystemTime;




use ethers::middleware::Middleware;
use ethers::signers::Signer;
use ethers::types::{Address, TransactionRequest};
use ethers::utils::parse_ether;


#[get("/api/matrix2d")]
pub async fn get_matrix2d(data: web::Data<Arc<Mutex<Matrix2D>>>) -> impl Responder {
    let matrix = data.lock().unwrap();
    HttpResponse::Ok().json(&*matrix)
}

pub async fn health_check() -> HttpResponse {
    let uptime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "uptime_seconds": uptime,
        "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()
    }))
}

// ... (rest of the code remains the same)

// Legacy completed_transactions endpoint removed. If needed, implement for Matrix2D logic.

pub async fn execute_arbitrage(
    payload: web::Json<serde_json::Value>,
    provider_data: web::Data<Arc<ProviderManager>>,
) -> HttpResponse {
    let text = match payload.get("text").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"status": "error", "reason": "Missing text parameter"}));
        }
    };

    match provider_data.execute_arbitrage_onchain(text).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "reason": format!("Failed to execute arbitrage: {}", e)
        })),
    }
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
                HttpResponse::Ok().json(serde_json::json!({"status": "success", "tx_hash": format!("0x{:x}", receipt.transaction_hash)}))
            }
            Ok(None) => {
                log::error!(
                    "Transaction pending or dropped for transfer from {} to {} amount {}",
                    from_addr,
                    to_addr,
                    amount
                );
                HttpResponse::InternalServerError().json(serde_json::json!({"status": "error", "reason": "Transaction pending or dropped"}))
            }
            Err(e) => {
                log::error!("Error waiting for transaction receipt: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({"status": "error", "reason": "Failed to get transaction receipt"}))
            }
        },
        Err(e) => {
            log::error!("Error sending transaction: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"status": "error", "reason": "Failed to send transaction"}),
            )
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
