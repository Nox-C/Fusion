use async_trait::async_trait;

use std::sync::Arc;



use ethers::types::Address;

use log::{info, error};

use std::env;


use crate::liquidation_monitor::{LiquidationEvent, LiquidationExecutor};
use crate::arbitrage_executor_address::ARBITRAGE_EXECUTOR_MAINNET;
use crate::execute_arbitrage::execute_arbitrage_onchain;
use tokio::sync::mpsc;
use tokio;


use crate::execution_log::{ExecutionLog, ExecutionRecord};


#[derive(Clone)]
pub struct RealArbitrageExecutor {
    pub abi_path: String,
    pub profit_wallet: String,
    pub execution_log: Arc<ExecutionLog>,
}

impl RealArbitrageExecutor {
    pub fn new(abi_path: String, profit_wallet: String, execution_log: Arc<ExecutionLog>) -> Self {
        Self { abi_path, profit_wallet, execution_log }
    }
}

#[async_trait]
impl LiquidationExecutor for RealArbitrageExecutor {
    async fn execute(&self, event: &LiquidationEvent) {
        let dry_run = std::env::var("DRY_RUN").unwrap_or_else(|_| "true".to_string()) == "true";
        info!("[RealArbitrageExecutor] {} liquidation for {} on {} (dry_run={})", if dry_run {"Simulating"} else {"Executing"}, event.account, event.protocol, dry_run);

        // Example: Map event fields to contract call params (these should be replaced with real logic)
        let contract_address = ARBITRAGE_EXECUTOR_MAINNET.parse().expect("Invalid contract address");
        let abi_path = &self.abi_path;
        let flashloan_client = "0x0000000000000000000000000000000000000000".parse().unwrap(); // TODO
        let loan_token = "0x0000000000000000000000000000000000000000".parse().unwrap(); // TODO
        let loan_amount = ethers::types::U256::from((event.debt * 1e18) as u128); // Example
        let routers = vec![]; // TODO
        let swap_paths = vec![]; // TODO
        let amounts_in = vec![]; // TODO
        let amounts_out_min = vec![]; // TODO
        let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
        let rpc_url = env::var("BSC_RPC_URL").unwrap_or_else(|_| "https://bsc-dataseed.binance.org/".to_string());

        if dry_run {
            info!("[DRY_RUN] Would call execute_arbitrage_onchain with: account={} debt={} collateral={}", event.account, event.debt, event.collateral);
            self.execution_log.log(ExecutionRecord {
                timestamp: chrono::Utc::now(),
                protocol: event.protocol.clone(),
                account: event.account.clone(),
                debt: event.debt,
                collateral: event.collateral,
                success: true,
                profit: 0.0,
                gas_used: None,
                tx_hash: None,
                error: None,
            });
            return;
        }
        match execute_arbitrage_onchain(
            contract_address,
            abi_path,
            flashloan_client,
            loan_token,
            loan_amount,
            routers,
            swap_paths,
            amounts_in,
            amounts_out_min,
            &private_key,
            &rpc_url,
        ).await {
            Ok(tx_hash) => {
                info!("[RealArbitrageExecutor] Submitted liquidation tx: 0x{:x}", tx_hash);
                self.execution_log.log(ExecutionRecord {
                    timestamp: chrono::Utc::now(),
                    protocol: event.protocol.clone(),
                    account: event.account.clone(),
                    debt: event.debt,
                    collateral: event.collateral,
                    success: true,
                    profit: 0.0,
                    gas_used: None,
                    tx_hash: Some(format!("0x{:x}", tx_hash)),
                    error: None,
                });
            }
            Err(e) => {
                error!("[RealArbitrageExecutor] Error executing liquidation: {}", e);
                self.execution_log.log(ExecutionRecord {
                    timestamp: chrono::Utc::now(),
                    protocol: event.protocol.clone(),
                    account: event.account.clone(),
                    debt: event.debt,
                    collateral: event.collateral,
                    success: false,
                    profit: 0.0,
                    gas_used: None,
                    tx_hash: None,
                    error: Some(format!("{}", e)),
                });
            }
        }
    }
}

pub struct VenusHelper {
    pub sender: mpsc::Sender<LiquidationEvent>,
    pub client: Arc<crate::providers::SignerHttpProvider>,
}

impl VenusHelper {
    pub async fn spawn_detection(self, shared_state: Arc<tokio::sync::Mutex<crate::shared_state::SharedState>>) {
        let mut last_interval = 60u64;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(last_interval));
        loop {
            // Dynamically fetch scan interval from shared state
            let interval_secs = {
                let state = shared_state.lock().await;
                *state.scan_intervals.get("Venus").unwrap_or(&60)
            };
            if interval_secs != last_interval {
                log::info!("[VenusHelper] Scan interval changed: {} -> {}", last_interval, interval_secs);
                interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                last_interval = interval_secs;
            }
            interval.tick().await;
            log::info!("[VenusHelper] Scanning for liquidation opportunities...");
            match fetch_venus_users_from_graph().await {
                Ok(users) => {
                    for user in users {
                        // Placeholder: simulate random opportunity
                        if rand::random::<u8>() % 20 == 0 {
                            let event = LiquidationEvent {
                                protocol: "Venus".to_string(),
                                account: format!("{:?}", user),
                                debt: 100.0,
                                collateral: 200.0,
                            };
                            let _ = self.sender.send(event).await;
                        }
                    }
                },
                Err(e) => log::error!("[VenusHelper] Error fetching users: {}", e),
            }
        }
    }
}


// --- End VenusHelper struct ---
pub struct AaveHelper {
    pub sender: mpsc::Sender<LiquidationEvent>,
    pub client: Arc<crate::providers::SignerHttpProvider>,
}

impl AaveHelper {
    pub async fn spawn_detection(self, shared_state: Arc<tokio::sync::Mutex<crate::shared_state::SharedState>>) {
        let mut last_interval = 60u64;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(last_interval));
        loop {
            // Dynamically fetch scan interval from shared state
            let interval_secs = {
                let state = shared_state.lock().await;
                *state.scan_intervals.get("Aave").unwrap_or(&60)
            };
            if interval_secs != last_interval {
                log::info!("[AaveHelper] Scan interval changed: {} -> {}", last_interval, interval_secs);
                interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                last_interval = interval_secs;
            }
            interval.tick().await;
            log::info!("[AaveHelper] Scanning for liquidation opportunities...");
            match fetch_aave_users_from_graph().await {
                Ok(users) => {
                    for user in users {
                        // Placeholder: simulate random opportunity
                        if rand::random::<u8>() % 20 == 0 {
                            let event = LiquidationEvent {
                                protocol: "Aave".to_string(),
                                account: format!("{:?}", user),
                                debt: 100.0,
                                collateral: 200.0,
                            };
                            let _ = self.sender.send(event).await;
                        }
                    }
                },
                Err(e) => log::error!("[AaveHelper] Error fetching users: {}", e),
            }
        }
    }
}

// --- End AaveHelper struct ---

// --- CompoundHelper struct ---

#[allow(dead_code)]
pub struct CompoundHelper {
    pub sender: mpsc::Sender<LiquidationEvent>,
    pub client: Arc<crate::providers::SignerHttpProvider>,
}

impl CompoundHelper {
    pub async fn spawn_detection(self, shared_state: Arc<tokio::sync::Mutex<crate::shared_state::SharedState>>) {
        let mut last_interval = 60u64;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(last_interval));
        loop {
            // Dynamically fetch scan interval from shared state
            let interval_secs = {
                let state = shared_state.lock().await;
                *state.scan_intervals.get("Compound").unwrap_or(&60)
            };
            if interval_secs != last_interval {
                log::info!("[CompoundHelper] Scan interval changed: {} -> {}", last_interval, interval_secs);
                interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                last_interval = interval_secs;
            }
            interval.tick().await;
            log::info!("[CompoundHelper] Scanning for liquidation opportunities...");
            match fetch_compound_users_from_graph().await {
                Ok(users) => {
                    for user in users {
                        // Placeholder: simulate random opportunity
                        if rand::random::<u8>() % 20 == 0 {
                            let event = LiquidationEvent {
                                protocol: "Compound".to_string(),
                                account: format!("{:?}", user),
                                debt: 100.0,
                                collateral: 200.0,
                            };
                            let _ = self.sender.send(event).await;
                        }
                    }
                },
                Err(e) => log::error!("[CompoundHelper] Error fetching users: {}", e),
            }
        }
    }
}


/*
--- End CompoundHelper struct ---
*/

// Helper: Fetch Aave users from The Graph (paginated)
#[allow(dead_code)]
async fn fetch_aave_users_from_graph() -> Result<Vec<Address>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use std::collections::HashSet;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct GraphUser {
        id: String
    }
    #[derive(Deserialize)]
    struct GraphData {
        users: Vec<GraphUser>
    }
    #[derive(Deserialize)]
    struct GraphResponse {
        data: GraphData
    }
    let client = reqwest::Client::new();
    let mut addresses = HashSet::new();
    let mut last_id = String::from("0x0000000000000000000000000000000000000000");
    let mut scan_round = 0;
    loop {
        scan_round += 1;
        info!("[AaveHelper] Scanning round {} for users after {}", scan_round, last_id);
        let query = format!(r#"{{ users(first: 1000, where: {{borrowedReservesCount_gt: 0, id_gt: "{}"}}) {{ id }} }}"#, last_id);
        let req_body = serde_json::json!({ "query": query });
        // Use The Graph Gateway endpoint for Aave v3 mainnet. Requires THEGRAPH_API_KEY in .env
        let graph_api_key = std::env::var("THEGRAPH_API_KEY").expect("THEGRAPH_API_KEY must be set in .env");
        let aave_v3_subgraph_id = "HB1Z2EAw4rtPRYVb2Nz8QGFLHCpym6ByBX6vbCViuE9F";
        let url = format!("https://gateway.thegraph.com/api/{}/subgraphs/id/{}", graph_api_key, aave_v3_subgraph_id);
        let resp = client.post(&url)
            .json(&req_body)
            .send()
            .await?;
        let resp_text = resp.text().await?;
        log::error!("[GraphQL] Raw response: {}", resp_text);
        let resp_json: GraphResponse = serde_json::from_str(&resp_text)?;
        if resp_json.data.users.is_empty() {
            break;
        }
        for user in resp_json.data.users.iter() {
            if let Ok(addr) = user.id.parse() {
                addresses.insert(addr);
            }
        }
        last_id = resp_json.data.users.last().unwrap().id.clone();
    }
    Ok(addresses.into_iter().collect())
}

// Helper: Fetch Venus users from The Graph (paginated)
#[allow(dead_code)]
async fn fetch_venus_users_from_graph() -> Result<Vec<Address>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use std::collections::HashSet;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct GraphAccount {
        id: String
    }
    #[derive(Deserialize)]
    struct GraphData {
        accounts: Vec<GraphAccount>
    }
    #[derive(Deserialize)]
    struct GraphResponse {
        data: GraphData
    }
    let client = reqwest::Client::new();
    let mut addresses = HashSet::new();
    let mut last_id = String::from("0x0000000000000000000000000000000000000000");
    let mut scan_round = 0;
    loop {
        scan_round += 1;
        info!("[VenusHelper] Scanning round {} for accounts after {}", scan_round, last_id);
        let query = format!(r#"{{ accounts(first: 1000, where: {{hasBorrowed: true, id_gt: "{}"}}) {{ id }} }}"#, last_id);
        let req_body = serde_json::json!({ "query": query });
        // TODO: Update to a working Venus subgraph endpoint. See: https://thegraph.com/explorer/subgraphs?query=venus
        log::warn!("Venus subgraph endpoint may be deprecated. Please update to a working endpoint or use direct on-chain event log scraping as a fallback. See https://docs.venus.io/ for more info.");
        let resp = client.post("https://api.thegraph.com/subgraphs/name/venusprotocol/venus")
            .json(&req_body)
            .send()
            .await?;
        let resp_text = resp.text().await?;
        log::error!("[GraphQL] Raw response: {}", resp_text);
        let resp_json: GraphResponse = serde_json::from_str(&resp_text)?;
        if resp_json.data.accounts.is_empty() {
            break;
        }
        for acc in resp_json.data.accounts.iter() {
            if let Ok(addr) = acc.id.parse() {
                addresses.insert(addr);
            }
        }
        last_id = resp_json.data.accounts.last().unwrap().id.clone();
    }
    Ok(addresses.into_iter().collect())
}

// Helper: Fetch Compound users from The Graph (paginated)
#[allow(dead_code)]
async fn fetch_compound_users_from_graph() -> Result<Vec<Address>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use std::collections::HashSet;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct GraphAccount {
        id: String
    }
    #[derive(Deserialize)]
    struct GraphData {
        accounts: Vec<GraphAccount>
    }
    #[derive(Deserialize)]
    struct GraphResponse {
        data: GraphData
    }
    let client = reqwest::Client::new();
    let mut addresses = HashSet::new();
    let mut last_id = String::from("0x0000000000000000000000000000000000000000");
    let mut scan_round = 0;
    loop {
        scan_round += 1;
        info!("[VenusHelper] Scanning round {} for accounts after {}", scan_round, last_id);
        let query = format!(r#"{{ accounts(first: 1000, where: {{hasBorrowed: true, id_gt: "{}"}}) {{ id }} }}"#, last_id);
        let req_body = serde_json::json!({ "query": query });
        // TODO: Update to a working Compound subgraph endpoint. See: https://thegraph.com/explorer/subgraphs?query=compound
        log::warn!("Compound subgraph endpoint may be deprecated. Please update to a working endpoint.");
        let resp = client.post("https://api.thegraph.com/subgraphs/name/graphprotocol/compound-v2")
            .json(&req_body)
            .send()
            .await?;
        let resp_text = resp.text().await?;
        log::error!("[GraphQL] Raw response: {}", resp_text);
        let resp_json: GraphResponse = serde_json::from_str(&resp_text)?;
        if resp_json.data.accounts.is_empty() {
            break;
        }
        for acc in resp_json.data.accounts.iter() {
            if let Ok(addr) = acc.id.parse() {
                addresses.insert(addr);
            }
        }
        last_id = resp_json.data.accounts.last().unwrap().id.clone();
    }
    Ok(addresses.into_iter().collect())
}
// --- End Helper functions ---
