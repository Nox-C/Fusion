// /home/nox/Fusion/src/providers.rs

use crate::config::Settings; // Use crate:: to refer to config module in the same crate
use crate::providers_round_robin::{ProviderEntry, ProviderRotation};
use ethers::providers::{Http, Middleware, Provider, ProviderError, Ws};
use ethers::signers::{LocalWallet, Signer};
use std::sync::Arc;
use thiserror::Error;
use url::Url; // For custom error types

#[derive(Debug, Error)]
pub enum ProviderManagerError {
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Provider connection error: {0}")]
    ProviderConnection(#[from] ProviderError),
    #[error("Invalid private key: {0}")]
    WalletError(String), // ethers::signers::WalletError doesn't impl Error directly sometimes
    #[error("No valid providers configured or connectable")]
    NoValidProvider,
    #[error("Unsupported provider name: {0}")]
    UnsupportedProvider(String),
}

// Define types for clarity
pub type HttpProvider = Provider<Http>;
pub type WsProvider = Provider<Ws>;
pub type SignerHttpProvider = ethers::middleware::SignerMiddleware<HttpProvider, LocalWallet>;
// pub type SignerWsProvider = ethers::middleware::SignerMiddleware<WsProvider, LocalWallet>; // If needed later

// A simple structure to hold provider connections for a specific chain (e.g., BSC Mainnet)
#[derive(Clone)] // Clone needed if we pass this around
pub struct ChainProvider {
    pub http_provider: Arc<SignerHttpProvider>, // Use Arc for shared ownership
    // pub ws_provider: Option<Arc<WsProvider>>, // Add WebSocket later if needed directly here
    pub chain_id: u64, // Store chain ID for reference
}

pub struct ProviderManager {
    pub bsc_provider: Option<ChainProvider>,
    pub eth_provider: Option<ChainProvider>,
    // Add testnet providers if needed
    // pub bsc_testnet_provider: Option<ChainProvider>,
    // pub eth_sepolia_provider: Option<ChainProvider>,
    // Store the wallet derived from the private key
    wallet: LocalWallet,
    // New: round-robin provider rotation for BSC and ETH
    pub bsc_rotation: Option<ProviderRotation>,
    pub eth_rotation: Option<ProviderRotation>,
}

impl ProviderManager {
    /// Example: Get the next available BSC provider (for future use)
    pub fn next_bsc_provider(&mut self) -> Option<&mut ProviderEntry> {
        self.bsc_rotation.as_mut()?.next_provider()
    }
    /// Example: Mark BSC provider as failed (for future use)
    pub fn mark_bsc_provider_failure(&mut self, name: &str, cooldown: std::time::Duration) {
        if let Some(rotation) = self.bsc_rotation.as_mut() {
            rotation.mark_provider_failure(name, cooldown);
        }
    }
    pub async fn new(settings: Arc<Settings>) -> Result<Self, ProviderManagerError> {
        // Create wallet from private key (robust parsing)
        let raw_key = settings
            .private_key
            .as_ref()
            .expect("PRIVATE_KEY must be set in environment or config")
            .trim()
            .to_string();
        println!("[DEBUG] Raw PRIVATE_KEY: {:?}, length: {}", raw_key, raw_key.len());
        // Try parsing as-is
        let wallet = match raw_key.parse::<LocalWallet>() {
            Ok(w) => Ok(w),
            Err(e1) => {
                // Try with 0x prefix
                let with_0x = if raw_key.starts_with("0x") { raw_key.clone() } else { format!("0x{}", raw_key) };
                match with_0x.parse::<LocalWallet>() {
                    Ok(w) => Ok(w),
                    Err(e2) => Err(ProviderManagerError::WalletError(format!(
                        "Tried parsing PRIVATE_KEY as-is (error: {}), and with 0x prefix (error: {})",
                        e1, e2
                    )))
                }
            }
        }?;

        // --- Try connecting to BSC Mainnet ---
        let bsc_provider = Self::connect_chain(
            &settings.provider_priority_order,
            &settings,
            Chain::BscMainnet,
            wallet.clone(), // Clone wallet for the middleware
        )
        .await;

        // --- Try connecting to ETH Mainnet ---
        let eth_provider = Self::connect_chain(
            &settings.provider_priority_order,
            &settings,
            Chain::EthMainnet,
            wallet.clone(), // Clone wallet for the middleware
        )
        .await;

        // Check if at least one connection succeeded (adjust as needed)
        if bsc_provider.is_err() && eth_provider.is_err() {
            return Err(ProviderManagerError::NoValidProvider);
        }

        // Prepare round-robin rotations for BSC and ETH (future use)
        let bsc_rotation = Some(ProviderRotation::new(
            vec![
                ProviderEntry {
                    name: "BSC-Primary".to_string(),
                    url: settings.bsc_rpc_url.clone(),
                    max_requests_per_minute: 60,
                    monthly_limit: Some(3_000_000), // Example: Infura
                    hourly_limit: Some(3_000_000 / 30 / 24), // ≈4,166/hour
                    daily_limit: Some(3_000_000 / 30), // ≈100,000/day
                    requests_this_window: 0,
                    window_start: None,
                    requests_this_hour: 0,
                    hour_start: None,
                    requests_today: 0,
                    day_start: None,
                    last_used: None,
                    cooldown_until: None,
                },
                // Add more BSC providers here as needed
            ],
            std::time::Duration::from_secs(60),
        ));
        let eth_rotation = Some(ProviderRotation::new(
            vec![
                ProviderEntry {
                    name: "ETH-Primary".to_string(),
                    url: settings.eth_rpc_url.clone(),
                    max_requests_per_minute: 60,
                    monthly_limit: Some(3_000_000), // Example: Infura
                    hourly_limit: Some(3_000_000 / 30 / 24), // ≈4,166/hour
                    daily_limit: Some(3_000_000 / 30), // ≈100,000/day
                    requests_this_window: 0,
                    window_start: None,
                    requests_this_hour: 0,
                    hour_start: None,
                    requests_today: 0,
                    day_start: None,
                    last_used: None,
                    cooldown_until: None,
                },
                // Add more ETH providers here as needed
            ],
            std::time::Duration::from_secs(60),
        ));

        Ok(Self {
            bsc_provider: bsc_provider.ok(), // Store Ok result, None otherwise
            eth_provider: eth_provider.ok(), // Store Ok result, None otherwise
            bsc_rotation,
            eth_rotation,
            wallet, // Store the original wallet
        })
    }

    // Helper to connect to a specific chain based on priority
    async fn connect_chain(
        priority: &[String],
        settings: &Settings,
        chain: Chain,
        wallet: LocalWallet,
    ) -> Result<ChainProvider, ProviderManagerError> {
        type ProviderUrlFn = fn(&Settings, &str) -> Option<String>;
        let (chain_id, get_urls): (u64, ProviderUrlFn) = match chain {
            Chain::BscMainnet => (56, |s, p| match p {
                "Infura" => Some(s.bsc_rpc_url.clone()),
                "Alchemy" => Some(s.alchemy_bsc_rpc_url.clone()),
                "NodeReal" => Some(s.nodereal_bsc_rpc_url.clone()),
                _ => None,
            }),
            Chain::EthMainnet => (1, |s, p| match p {
                "Infura" => Some(s.eth_rpc_url.clone()),
                "Alchemy" => Some(s.alchemy_eth_rpc_url.clone()),
                "NodeReal" => Some(s.nodereal_eth_rpc_url.clone()),
                _ => None,
            }),
            // Add other chains (BscTestnet, EthSepolia) here if needed
        };

        println!(
            "Attempting to connect to {:?} (Chain ID: {})...",
            chain, chain_id
        );

        for provider_name in priority {
            if let Some(mut rpc_url_str) = get_urls(settings, provider_name) {
                // Substitute env vars at runtime
                rpc_url_str = Self::substitute_provider_keys(&rpc_url_str);
                println!(
                    "  Trying provider: {} at URL: {}",
                    provider_name, rpc_url_str
                );
                let url = match Url::parse(&rpc_url_str) {
                    Ok(url) => url,
                    Err(e) => {
                        eprintln!("    Warning: Invalid URL for {}: {}", provider_name, e);
                        continue;
                    }
                };
                let http_client = Http::new(url);
                let provider = Provider::new(http_client);
                // Check connection with a simple call like getting chain ID
                match provider.get_chainid().await {
                    Ok(id) if id.as_u64() == chain_id => {
                        println!("    Successfully connected to {}!", provider_name);
                        // Add wallet middleware
                        let signer_provider = ethers::middleware::SignerMiddleware::new(
                            provider,
                            wallet.with_chain_id(chain_id), // Ensure wallet has correct chain ID
                        );
                        return Ok(ChainProvider {
                            http_provider: Arc::new(signer_provider),
                            chain_id,
                        });
                    }
                    Ok(id) => eprintln!(
                        "    Warning: Connected to {} but chain ID mismatch (Expected: {}, Got: {})",
                        provider_name, chain_id, id
                    ),
                    Err(e) => eprintln!(
                        "    Warning: Failed to verify connection to {}: {}",
                        provider_name, e
                    ),
                }
            } else {
                eprintln!(
                    "    Skipping unsupported provider for this chain: {}",
                    provider_name
                );
            }
        }

        Err(ProviderManagerError::NoValidProvider) // Return error if no provider worked
    }

    /// Substitute ${INFURA_API_KEY}, ${ALCHEMY_API_KEY}, ${NODEREAL_API_KEY} in a URL string
    fn substitute_provider_keys(url: &str) -> String {
        let mut out = url.to_string();
        if let Ok(val) = std::env::var("INFURA_API_KEY") {
            out = out.replace("${INFURA_API_KEY}", &val);
        }
        if let Ok(val) = std::env::var("ALCHEMY_API_KEY") {
            out = out.replace("${ALCHEMY_API_KEY}", &val);
        }
        if let Ok(val) = std::env::var("NODEREAL_API_KEY") {
            out = out.replace("${NODEREAL_API_KEY}", &val);
        }
        out
    }
    pub fn get_wallet(&self) -> &LocalWallet {
        &self.wallet
    }
}

// Enum to represent supported chains easily
#[derive(Debug, Clone, Copy)]
enum Chain {
    BscMainnet,
    EthMainnet,
    // BscTestnet,
    // EthSepolia,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_provider_manager_rotation_integration() {
        let mut pm = ProviderManager {
            bsc_provider: None,
            eth_provider: None,
            wallet: LocalWallet::from_bytes(&[1u8; 32]).unwrap(),
            bsc_rotation: Some(ProviderRotation::new(
                vec![ProviderEntry {
                    name: "BSC-1".to_string(),
                    url: "http://bsc1".to_string(),
                    max_requests_per_minute: 2,
                    monthly_limit: Some(0),
                    hourly_limit: Some(0),
                    daily_limit: Some(0),
                    requests_this_window: 0,
                    window_start: None,
                    requests_this_hour: 0,
                    hour_start: None,
                    requests_today: 0,
                    day_start: None,
                    last_used: None,
                    cooldown_until: None,
                }],
                Duration::from_secs(60),
            )),
            eth_rotation: None,
        };
        assert!(pm.next_bsc_provider().is_some());
        pm.mark_bsc_provider_failure("BSC-1", Duration::from_millis(100));
        // Should be unavailable during cooldown
        assert!(pm.next_bsc_provider().is_none());
    }
}
