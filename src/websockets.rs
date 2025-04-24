use crate::config::Settings;
use ethers::providers::{Provider, Ws, PubsubClient, StreamExt};
use crate::websockets_round_robin::{DexWebSocketEntry, DexWebSocketRotation};
use futures_util::stream::FuturesUnordered;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use tokio::task::JoinHandle;
use url::Url;

#[derive(Debug, Error)]
pub enum WebSocketError {
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("WebSocket connection failed for URL {0}: {1}")]
    ConnectionFailed(String, String), // url, error message
    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Subscription error: {0}")]
    SubscriptionError(String),
}

// Structure to manage multiple WebSocket connections and their listeners
pub struct WebSocketManager {
    settings: Arc<Settings>,
    // Store active connections (Provider<Ws>) and their listener tasks
    connections: HashMap<String, (Arc<Provider<Ws>>, JoinHandle<()>)>,
    // Add round-robin rotation for each DEX
    pancakeswap_rotation: DexWebSocketRotation,
    biswap_rotation: DexWebSocketRotation,
    mdex_rotation: DexWebSocketRotation,
    babyswap_rotation: DexWebSocketRotation,
    apeswap_rotation: DexWebSocketRotation,
    kokoswap_rotation: DexWebSocketRotation,
    thena_rotation: DexWebSocketRotation,
    waultswap_rotation: DexWebSocketRotation,
    dodo_rotation: DexWebSocketRotation,
    ellipsis_rotation: DexWebSocketRotation,
}

impl WebSocketManager {
    pub fn new(settings: Arc<Settings>) -> Self {
        // Helper to create a single-entry rotation for each DEX (can expand to multiple URLs)
        let dex_rotation = |name: &str, url: &str| {
            DexWebSocketRotation::new(
                vec![DexWebSocketEntry {
                    name: name.to_string(),
                    url: url.to_string(),
                    max_reconnects_per_minute: 60,
                    last_used: None,
                    cooldown_until: None,
                    reconnects_this_window: 0,
                    window_start: None,
                }],
                std::time::Duration::from_secs(60),
            )
        };
        Self {
            settings: settings.clone(),
            connections: HashMap::new(),
            pancakeswap_rotation: dex_rotation("PancakeSwap", &settings.websocket_pancakeswap),
            biswap_rotation: dex_rotation("Biswap", &settings.websocket_biswap),
            mdex_rotation: dex_rotation("MDEX", &settings.websocket_mdex),
            babyswap_rotation: dex_rotation("BabySwap", &settings.websocket_babyswap),
            apeswap_rotation: dex_rotation("ApeSwap", &settings.websocket_apeswap),
            kokoswap_rotation: dex_rotation("KokoSwap", &settings.websocket_kokoswap),
            thena_rotation: dex_rotation("Thena", &settings.websocket_thena),
            waultswap_rotation: dex_rotation("WaultSwap", &settings.websocket_waultswap),
            dodo_rotation: dex_rotation("DODO", &settings.websocket_dodo),
            ellipsis_rotation: dex_rotation("Ellipsis", &settings.websocket_ellipsis),
        }
    }

    // Connect to primary chain WebSockets (BSC, ETH) based on priority
    // and also connect to DEX-specific WebSockets
    pub async fn connect_all(&mut self) {
        let mut futures = FuturesUnordered::new();

        // --- BSC Primary WebSocket ---
        if let Some(url) = self.get_chain_ws_url(Chain::BscMainnet) {
            futures.push(Self::connect_and_listen("BSC_Primary".to_string(), url));
        } else {
             log::warn!("Warning: Could not determine primary WebSocket URL for BSC Mainnet.");
        }

        // --- ETH Primary WebSocket ---
         if let Some(url) = self.get_chain_ws_url(Chain::EthMainnet) {
            futures.push(Self::connect_and_listen("ETH_Primary".to_string(), url));
        } else {
             log::warn!("Warning: Could not determine primary WebSocket URL for ETH Mainnet.");
        }

        // --- DEX Specific WebSockets ---
        // Create a helper function or macro to reduce repetition if desired
        self.add_dex_ws_future(&mut futures, "PancakeSwap");
        self.add_dex_ws_future(&mut futures, "Biswap");
        self.add_dex_ws_future(&mut futures, "MDEX");
        self.add_dex_ws_future(&mut futures, "BabySwap");
        self.add_dex_ws_future(&mut futures, "ApeSwap");
        self.add_dex_ws_future(&mut futures, "KokoSwap");
        self.add_dex_ws_future(&mut futures, "Thena");
        self.add_dex_ws_future(&mut futures, "WaultSwap");
        self.add_dex_ws_future(&mut futures, "DODO");
        self.add_dex_ws_future(&mut futures, "Ellipsis");

        log::warn!("Attempting to establish all WebSocket connections...");

        // Process connection results as they complete
        while let Some(result) = futures.next().await {
             match result {
                Ok((name, ws_provider, handle)) => {
                    log::warn!("--> Successfully connected WebSocket for: {}", name);
                    self.connections.insert(name, (ws_provider, handle));
                }
                Err(e) => {
                    // Log the error but continue trying other connections
                    log::warn!("--> WebSocket connection error: {}", e);
                }
            }
        }
         log::warn!("Finished attempting all WebSocket connections. Active count: {}", self.connections.len());
    }

    // Helper to add DEX WebSocket connection future
    fn add_dex_ws_future<'a>(
        &mut self,
        futures: &'a mut FuturesUnordered<impl futures_util::Future<Output = Result<(String, Arc<Provider<Ws>>, JoinHandle<()>) , WebSocketError>> + Unpin>,
        name: &str,
    ) {
        let rotation = match name {
            "PancakeSwap" => &mut self.pancakeswap_rotation,
            "Biswap" => &mut self.biswap_rotation,
            "MDEX" => &mut self.mdex_rotation,
            "BabySwap" => &mut self.babyswap_rotation,
            "ApeSwap" => &mut self.apeswap_rotation,
            "KokoSwap" => &mut self.kokoswap_rotation,
            "Thena" => &mut self.thena_rotation,
            "WaultSwap" => &mut self.waultswap_rotation,
            "DODO" => &mut self.dodo_rotation,
            "Ellipsis" => &mut self.ellipsis_rotation,
            _ => return,
        };
        if let Some(entry) = rotation.next_endpoint() {
            if entry.url.is_empty() {
                log::warn!("Warning: Skipping DEX WebSocket for {} due to empty URL.", name);
                return;
            }
            futures.push(Self::connect_and_listen(name.to_string(), entry.url.clone()));
        } else {
            log::warn!("No available endpoint for {} (all on cooldown or exhausted)", name);
        }
    }


    // Helper to get the prioritized WebSocket URL for a chain
    fn get_chain_ws_url(&self, chain: Chain) -> Option<String> {
        let get_urls: fn(&Settings, &str) -> Option<String> = match chain {
            Chain::BscMainnet => |s, p| match p {
                "Infura" => Some(s.infura_bsc_websocket_url.clone()),
                "Alchemy" => Some(s.alchemy_bsc_websocket_url.clone()),
                "NodeReal" => Some(s.nodereal_bsc_websocket_url.clone()),
                _ => None,
            },
            Chain::EthMainnet => |s, p| match p {
                "Infura" => Some(s.infura_eth_websocket_url.clone()),
                "Alchemy" => Some(s.alchemy_eth_websocket_url.clone()),
                "NodeReal" => Some(s.nodereal_eth_websocket_url.clone()),
                _ => None,
            },
            // Add other chains if needed
        };

        log::warn!("Searching for WebSocket URL for {:?}...", chain);
        for provider_name in &self.settings.provider_priority_order {
            if let Some(url) = get_urls(&self.settings, provider_name) {
                // Basic check: is the URL non-empty?
                if !url.is_empty() {
                     log::warn!("   Found URL via {}: {}", provider_name, url);
                    return Some(url);
                } else {
                     log::warn!("   Provider {} has empty URL for {:?}, skipping.", provider_name, chain);
                }
            } else {
                 // This case should ideally not happen if priority list matches available settings
                 log::warn!("   Provider {} not configured for {:?} WebSocket.", provider_name, chain);
            }
        }
        None // No suitable URL found
    }

    // Connects to a single WebSocket URL and spawns a listener task
    async fn connect_and_listen(
        name: String,
        url_str: String,
    ) -> Result<(String, Arc<Provider<Ws>>, JoinHandle<()>), WebSocketError> {
        log::warn!("   Attempting WebSocket connection to: {} ({})", name, url_str);
        let ws = Ws::connect(url_str.clone())
            .await
            .map_err(|e| WebSocketError::ConnectionFailed(url_str.clone(), e.to_string()))?;
        let provider = Arc::new(Provider::new(ws));

        // Spawn a task to listen for messages (e.g., new blocks)
        let provider_clone = provider.clone();
        let name_clone = name.clone();
        let handle = tokio::spawn(async move {
            log::warn!("   Listener spawned for: {}", name_clone);
            match provider_clone.subscribe_blocks().await {
                Ok(mut stream) => {
                     log::warn!("   Subscribed to new blocks on: {}", name_clone);
                    while let Some(block) = stream.next().await {
                        // --- Simulate a DEX price update and arbitrage scan ---
                        // In real code, you would extract price data from the DEX feed here
                        let mut rng = rand::thread_rng();
                        let price = rng.gen_range(1.0..100.0); // Simulate a price
                        // Example: matrix_id and dex name are stubbed for now
                        let matrix_id = if name_clone.contains("ETH") { "matrix_eth_1" } else { "matrix_bsc_1" };
                        let dex = if name_clone.contains("Pancake") { "PancakeSwap" } else { "Biswap" };
                        // TODO: Pass matrix_manager and marginal_optimizer via closure or context
                        // matrix_manager.update_dex_price(matrix_id, dex, price);
                        // let opps = matrix_manager.scan_for_arbitrage_opportunities(marginal_optimizer);
                        log::warn!(
                            "[{}] New Block: Number={:?}, Hash={:?}, Simulated {} price: {}",
                            name_clone,
                            block.number.map(|n| n.as_u64()),
                            block.hash.map(|h| format!("{:.8}", h)),
                            dex,
                            price
                        );
                        // For each opportunity, print or trigger downstream logic
                        // for opp in opps { log::warn!("Found arbitrage: {}", opp); }
                    }
                     log::warn!("Block stream ended unexpectedly for: {}", name_clone); // Should ideally not happen unless WS disconnects
                }
                Err(e) => {
                    log::warn!("Error subscribing to blocks for {}: {}", name_clone, e);
                }
            }
             log::warn!("WebSocket listener task finished for: {}", name_clone);
        });

        Ok((name, provider, handle))
    }

    // Method to gracefully shut down listeners
    pub async fn shutdown(&mut self) {
        log::warn!("Shutting down WebSocket listeners...");
        let count = self.connections.len();
        for (name, (_, handle)) in self.connections.drain() {
             log::warn!("   Aborting listener for: {}", name);
            handle.abort();
        }
         log::warn!("{} WebSocket listeners shut down.", count);
    }
}

// Re-use Chain enum, maybe move to a shared `common` module later?
#[derive(Debug, Clone, Copy)]
enum Chain {
    BscMainnet,
    EthMainnet,
}
