// /home/nox/Fusion/src/main.rs


use actix_web::{App, HttpServer, web};
use config::{Config, Environment, File};
use dotenvy::dotenv;

use fusion::config::Settings;
use fusion::matrix2d::Matrix2D;
use std::sync::Arc;
use tokio::sync::Mutex;


use fusion::providers::ProviderManager;
use fusion::liquidation_monitor::LiquidationMonitor;
use fusion::liquidation_monitor_real::{RealArbitrageExecutor, VenusHelper};
use tokio::sync::mpsc;
// use fusion::optimizer_ai::OptimizerAI;
use fusion::shared_state::SharedState;
use fusion::api;
use actix_cors::Cors;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables and logging
    dotenv().ok();
    env_logger::init();
    // DEBUG: Print current working directory and env vars
    println!("[DEBUG] CWD: {}", std::env::current_dir().unwrap().display());
    println!("[DEBUG] PRIVATE_KEY={:?}", std::env::var("PRIVATE_KEY"));
    println!("[DEBUG] PROFIT_WALLET={:?}", std::env::var("PROFIT_WALLET"));
    println!("[DEBUG] INFURA_API_KEY={:?}", std::env::var("INFURA_API_KEY"));
    println!("[DEBUG] ALCHEMY_API_KEY={:?}", std::env::var("ALCHEMY_API_KEY"));
    println!("[DEBUG] NODEREAL_API_KEY={:?}", std::env::var("NODEREAL_API_KEY"));
    log::info!("Starting Fusion backend API...");
    log::info!("WebSocket endpoint: ws://localhost:8080/ws/matrix2d");
    log::info!("Health check endpoint: http://localhost:8080/health");
    // Load configuration
    let settings: Settings = Config::builder()
        .add_source(File::with_name("config/default.toml"))
        .add_source(Environment::default())
        .build()
        .expect("Failed to build config")
        .try_deserialize()
        .expect("Failed to deserialize settings");

    log::info!("Application routes configured");

    // Initialize shared state
    let shared = Arc::new(Mutex::new(SharedState::default()));

    // Initialize on-chain providers with rotation support
    let provider_manager = Arc::new(
        ProviderManager::new(Arc::new(settings.clone()))
            .await
            .expect("ProviderManager initialization failed"),
    );

    // Initialize broadcast channel for WebSocket events
    let (event_tx, _) = tokio::sync::broadcast::channel::<WebSocketEvent>(100);
    let event_tx = web::Data::new(event_tx);

    // Start HTTP/WebSocket server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(Arc::new(settings.clone())))
            .app_data(web::Data::new(Arc::new(provider_manager.clone())))
            .app_data(web::Data::new(Arc::new(matrix2d.clone())))
            .app_data(web::Data::new(Arc::new(shared.clone())))
            .app_data(event_tx.clone())
            .service(web::resource("/ws/matrix2d").to(fusion::api_ws::ws_matrix2d_handler))
            .service(web::resource("/health").to(api::health_check))
            .service(web::resource("/execute_arbitrage").to(api::execute_arbitrage))
            .service(api::get_matrix2d)
            .service(api::post_transfer)
            .service(api::get_wallet_status)
            .service(api::post_connect_wallet)
    })
    .bind((
        std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080)
    ))?
    .run()
    .await
}
