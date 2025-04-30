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
        .add_source(Environment::default())
        .build()
        .expect("Failed to build config")
        .try_deserialize()
        .expect("Failed to deserialize settings");

    // Initialize shared state
    let shared = Arc::new(Mutex::new(SharedState::default()));

    // Initialize on-chain providers with rotation support
    let provider_manager = Arc::new(
        ProviderManager::new(Arc::new(settings.clone()))
            .await
            .expect("ProviderManager initialization failed"),
    );

    // === Multi-Protocol Liquidation Monitoring ===
    // Create async channel for liquidation events
    let (liquidation_tx, liquidation_rx) = mpsc::channel(32);

    // Set up the real executor
    let execution_log = Arc::new(fusion::execution_log::ExecutionLog::new());
    let real_executor = RealArbitrageExecutor {
        abi_path: "out/ArbitrageExecutor.sol/ArbitrageExecutor.json".to_string(),
        profit_wallet: std::env::var("PROFIT_WALLET").unwrap_or_default(),
        execution_log: execution_log.clone(),
    };

    // Spawn Venus protocol helper
    let _venus_helper = VenusHelper {
        client: provider_manager.bsc_provider.as_ref().expect("No BSC provider").http_provider.clone(),
        sender: liquidation_tx.clone(),
    };
    tokio::spawn(_venus_helper.spawn_detection(shared.clone()));

    // Spawn Aave protocol helper
    let _aave_helper = fusion::liquidation_monitor_real::AaveHelper {
        client: provider_manager.eth_provider.as_ref().expect("No ETH provider").http_provider.clone(),
        sender: liquidation_tx.clone(),
    };
    tokio::spawn(_aave_helper.spawn_detection(shared.clone()));

    // Spawn Compound protocol helper
    let _compound_helper = fusion::liquidation_monitor_real::CompoundHelper {
        client: provider_manager.eth_provider.as_ref().expect("No ETH provider").http_provider.clone(),
        sender: liquidation_tx.clone(),
    };
    tokio::spawn(_compound_helper.spawn_detection(shared.clone()));

    // Spawn the liquidation monitor in parallel with the real executor
    tokio::spawn(LiquidationMonitor::new_with_rx_and_executor(liquidation_rx, real_executor.clone()).run(shared.clone()));

    // Spawn the profit-maximizing AI controller
    let execution_log = real_executor.execution_log.clone();
    let shared_ai = shared.clone();
    tokio::spawn(fusion::ai_controller::run_ai_controller(execution_log, shared_ai));

    // (Optional) Spawn the optimizer AI if you want both
    // tokio::spawn(OptimizerAI::new().run(shared.clone()));
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
    // Initialize Matrix2D (shared state)
    // TODO: Select appropriate asset list from settings (e.g. matrix1_tokens or similar)
    let matrix2d = Arc::new(Mutex::new(Matrix2D::new(
        (*settings).dexes.clone(),
        (*settings).matrix1_tokens.clone(), // Default to matrix1_tokens, update as needed
    )));

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

    // Start HTTP/WebSocket server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(settings.clone()))
            .app_data(web::Data::from(provider_manager.clone()))
            .app_data(web::Data::from(matrix2d.clone()))
            .service(api::get_matrix2d)
            .service(api::post_transfer)
            .service(api::get_wallet_status)
            .service(api::post_connect_wallet)
            .route("/ws/matrix2d", web::get().to(fusion::api_ws::ws_matrix2d_handler))
            
    })
    // TODO: Replace with actual host/port fields from Settings. Example uses environment variables or hardcoded values.
    .bind((
        std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080)
    ))?
    .run()
    .await
}
