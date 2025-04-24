// /home/nox/Fusion/src/main.rs

use actix_files::Files;
use actix_web::{App, HttpServer, web};
use config::{Config, Environment, File};
use dotenvy::dotenv;
use fusion::analysis::AnalysisHub;
use fusion::config::Settings;
use fusion::matrix::MatrixManager;
mod dex_price_fetch;
use dex_price_fetch::{fetch_price, Dex};
use fusion::providers::ProviderManager;
use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;

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
    // DEBUG: Print private key length before ProviderManager
    if let Some(pk) = &settings.private_key {
        println!("[DEBUG] PRIVATE_KEY length: {}", pk.len());
        println!("[DEBUG] PRIVATE_KEY raw: {:?}", pk);
    } else {
        println!("[DEBUG] PRIVATE_KEY is None");
    }
    // Initialize on-chain providers with rotation support
    // Construct provider URLs with actual API keys
    let infura_api_key = settings.infura_api_key.clone().unwrap_or_default();
    let alchemy_api_key = settings.alchemy_api_key.clone().unwrap_or_default();
    let nodereal_api_key = settings.nodereal_api_key.clone().unwrap_or_default();

    let bsc_infura_url = format!("https://bsc-mainnet.infura.io/v3/{}", infura_api_key);
    let eth_infura_url = format!("https://mainnet.infura.io/v3/{}", infura_api_key);
    let bsc_alchemy_url = format!("https://bnb-mainnet.g.alchemy.com/v2/{}", alchemy_api_key);
    let eth_alchemy_url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_api_key);
    let bsc_nodereal_url = format!("https://bsc-mainnet.nodereal.io/v1/{}", nodereal_api_key);
    let eth_nodereal_url = format!("https://eth-mainnet.nodereal.io/v1/{}", nodereal_api_key);

    println!("[DEBUG] BSC Infura URL: {}", bsc_infura_url);
    println!("[DEBUG] ETH Infura URL: {}", eth_infura_url);
    println!("[DEBUG] BSC Alchemy URL: {}", bsc_alchemy_url);
    println!("[DEBUG] ETH Alchemy URL: {}", eth_alchemy_url);
    println!("[DEBUG] BSC NodeReal URL: {}", bsc_nodereal_url);
    println!("[DEBUG] ETH NodeReal URL: {}", eth_nodereal_url);

    let provider_manager = Arc::new(Mutex::new(
        ProviderManager::new(settings.clone())
            .await
            .expect("ProviderManager initialization failed"),
    ));
    // Initialize matrix manager with ETH and BSC matrices from config
    let matrix_manager = Arc::new(MatrixManager::with_settings(&settings));
    // Spawn periodic price updater and arbitrage scanning task
    {
        let mm = matrix_manager.clone();
        let cfg = settings.clone();
        // Build tokens, pairs, and dex_map as owned values before spawning tasks
        let mut tokens = std::collections::HashMap::new();
        tokens.insert("WBNB".to_string(), cfg.token_wbnb.clone());
        tokens.insert("BUSD".to_string(), cfg.token_busd.clone());
        tokens.insert("USDT".to_string(), cfg.token_usdt.clone());
        tokens.insert("USDC".to_string(), cfg.token_usdc.clone());
        tokens.insert("CAKE".to_string(), cfg.token_cake.clone());
        for symbol in cfg.matrix1_tokens.clone() {
            if !tokens.contains_key(&symbol) {
                let key = format!("token_{}", symbol.to_lowercase());
                if let Ok(val) = std::env::var(&key.to_uppercase()) {
                    tokens.insert(symbol, val);
                }
            }
        }
        let pairs: Vec<(String, String)> = cfg.matrix1_pairs.clone().into_iter().filter_map(|pair| {
            let parts: Vec<&str> = pair.split('/').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        }).collect();
        let dex_map: Vec<(String, Dex)> = cfg.dexes.clone().into_iter().filter_map(|dex_name| {
            match dex_name.to_lowercase().as_str() {
                "pancakeswap" => Some(("PancakeSwap".to_string(), Dex::PancakeSwap)),
                "biswap" => Some(("Biswap".to_string(), Dex::Biswap)),
                "apeswap" => Some(("ApeSwap".to_string(), Dex::ApeSwap)),
                "babyswap" => Some(("BabySwap".to_string(), Dex::BabySwap)),
                "mdex" => Some(("MDEX".to_string(), Dex::MDEX)),
                "dodo" => Some(("DODO".to_string(), Dex::DODO)),
                "thena" => Some(("Thena".to_string(), Dex::Thena)),
                "ellipsis" => Some(("Ellipsis".to_string(), Dex::Ellipsis)),
                "waultswap" => Some(("WaultSwap".to_string(), Dex::WaultSwap)),
                _ => None,
            }
        }).collect();
        for (dex_name, dex_enum) in dex_map {
            for (base, quote) in &pairs {
                let mm = mm.clone();
                let dex_name = dex_name.clone();
                let dex_enum = dex_enum.clone();
                let base = base.clone();
                let quote = quote.clone();
                let tokens = tokens.clone();
                tokio::spawn(async move {
                    loop {
                        let base_addr = tokens.get(&base);
                        let quote_addr = tokens.get(&quote);
                        if let (Some(base_addr), Some(_quote_addr)) = (base_addr, quote_addr) {
                            if let Some(price) = fetch_price(dex_enum.clone(), base_addr).await {
                                mm.update_dex_price("BSC", &dex_name, price);
                                log::info!("[PRICE UPDATE] {} {}-{}: {}", dex_name, base, quote, price);
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                    }
                });
            }
        }
        // Arbitrage scan task
        let mm = matrix_manager.clone();
        let cfg = settings.clone();
        let provider_guard = provider_manager.lock().await;
        let eth_chain = provider_guard.eth_provider.as_ref().expect("ETH provider not configured");
        let client = eth_chain.http_provider.clone();
        // Run arbitrage scan in a tight loop (no artificial delay)
        tokio::spawn(async move {
            loop {
                let matrices = mm.all();
                let opps = AnalysisHub::scan_all(&matrices, &cfg, client.clone()).await;
                log::info!("Found {} arbitrage opportunities", opps.len());
                // Minimal sleep to avoid 100% CPU, but maximize scan frequency
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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
