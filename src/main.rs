// /home/nox/Fusion/src/main.rs

use actix_files::Files;
use actix_web::{App, HttpServer, web};
use config::{Config, Environment, File};
use dotenvy::dotenv;
use fusion::analysis::AnalysisHub;
use fusion::config::Settings;
use fusion::matrix::MatrixManager;
use fusion::providers::ProviderManager;
use log::info;
use std::sync::Arc;
use tokio;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables and logging
    dotenv().ok();
    env_logger::init();
    log::info!("Starting Fusion backend API...");
    // Load configuration
    let settings: Settings = Config::builder()
        .add_source(File::with_name("config/default.toml"))
        .add_source(Environment::with_prefix("APP"))
        .build()
        .expect("Failed to build config")
        .try_deserialize()
        .expect("Failed to deserialize settings");
    let settings = Arc::new(settings);
    // Initialize on-chain providers
    let provider_manager = Arc::new(
        ProviderManager::new(settings.clone())
            .await
            .expect("ProviderManager initialization failed"),
    );
    // Initialize matrix manager
    let matrix_manager = Arc::new(MatrixManager::new());
    // Spawn periodic arbitrage scanning task
    {
        let mm = matrix_manager.clone();
        let cfg = settings.clone();
        // Use ETH provider for scanning
        let eth_chain = provider_manager
            .eth_provider
            .as_ref()
            .expect("ETH provider not configured");
        let client = eth_chain.http_provider.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_millis(
                cfg.matrix_update_interval_ms,
            ));
            loop {
                ticker.tick().await;
                let matrices = mm.all();
                let opps = AnalysisHub::scan_all(&matrices, &cfg, client.clone()).await;
                log::info!("Found {} arbitrage opportunities", opps.len());
            }
        });
    }
    // Start HTTP/WebSocket server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(matrix_manager.clone()))
            .app_data(web::Data::new(provider_manager.clone()))
            .service(fusion::api::get_matrices)
            .service(fusion::api::get_completed_transactions)
            .service(fusion::api::post_transfer)
            .service(fusion::api::get_wallet_status)
            .service(fusion::api::post_connect_wallet)
            .route(
                "/ws/matrices",
                web::get().to(fusion::api::ws_matrices_handler),
            )
            // serve built frontend
            .service(Files::new("/", "frontend/dist").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
