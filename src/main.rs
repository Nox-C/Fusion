// /home/nox/Fusion/src/main.rs

use actix_files::Files;
use actix_web::{App, HttpServer, web};
use config::{Config, Environment, File};
use dotenvy::dotenv;
use fusion::analysis::AnalysisHub;
use fusion::config::Settings;
use fusion::matrix::MatrixManager;
use fusion::providers::ProviderManager;
use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;

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

    // === SECURITY CHECK: Warn if secrets are not set via environment variables ===
    #[cfg(not(debug_assertions))]
    {
        use std::process::exit;
        let s = &settings;
        let mut missing = Vec::new();
        let check = |val: &Option<String>, name: &str| {
            match val {
                None => true,
                Some(v) if v.trim().is_empty() => true,
                Some(v) if v.starts_with("${") && v.ends_with("}") => true,
                _ => false,
            }
        };
        if check(&s.private_key, "PRIVATE_KEY") { missing.push("PRIVATE_KEY"); }
        if check(&s.profit_wallet, "PROFIT_WALLET") { missing.push("PROFIT_WALLET"); }
        if check(&s.infura_api_key, "INFURA_API_KEY") { missing.push("INFURA_API_KEY"); }
        if check(&s.alchemy_api_key, "ALCHEMY_API_KEY") { missing.push("ALCHEMY_API_KEY"); }
        if check(&s.nodereal_api_key, "NODEREAL_API_KEY") { missing.push("NODEREAL_API_KEY"); }
        if !missing.is_empty() {
            eprintln!("\n[SECURITY ERROR] The following secrets are missing or invalid: {:?}\n--> Do NOT run in production without setting these securely via env vars.\n", missing);
            exit(1);
        }
    }
    // Initialize on-chain providers with rotation support
    let provider_manager = Arc::new(Mutex::new(
        ProviderManager::new(settings.clone())
            .await
            .expect("ProviderManager initialization failed"),
    ));
    // Initialize matrix manager with ETH and BSC matrices from config
    let matrix_manager = Arc::new(MatrixManager::with_settings(&settings));
    // Spawn periodic arbitrage scanning task
    {
        let mm = matrix_manager.clone();
        let cfg = settings.clone();
        // Use ETH provider for scanning
        let provider_guard = provider_manager.lock().await;
        let eth_chain = provider_guard.eth_provider.as_ref().expect("ETH provider not configured");
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
            .service(fusion::api::ws_matrices_handler)
            // serve built frontend
            .service(Files::new("/", "frontend/dist").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
