use async_trait::async_trait;
use ethers::abi::Abi;
use ethers::types::Address;
use futures::stream::{FuturesUnordered, StreamExt};
use log::{info, error};
use std::env;
use std::fs::File;
use std::str::FromStr;
use crate::liquidation_monitor::{LiquidationEvent, LiquidationExecutor, ProtocolHelper};
use crate::arbitrage_executor_address::ARBITRAGE_EXECUTOR_MAINNET;
use crate::execute_arbitrage::execute_arbitrage_onchain;
use tokio::sync::mpsc;
use tokio;


use crate::execution_log::{ExecutionLog, ExecutionRecord};
use std::sync::Arc;

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
        let flashloan_provider = "0x0000000000000000000000000000000000000000".parse().unwrap(); // TODO
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
            flashloan_provider,
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
}

impl ProtocolHelper for VenusHelper {
    fn protocol_name(&self) -> &'static str { "Venus" }
    fn spawn_detection(&self) {
        let _sender = self.sender.clone();
        
        
        tokio::spawn(async move {
            info!("[VenusHelper] Starting Venus mainnet liquidation monitoring");
            use ethers::prelude::*;
            use std::sync::Arc;
            use std::time::Duration;
            let venus_comptroller_address = match env::var("VENUS_COMPTROLLER") {
                Ok(addr) => addr,
                Err(_) => {
                    error!("[VenusHelper] VENUS_COMPTROLLER env var required");
                    return;
                }
            };
            let venus_comptroller: Address = match venus_comptroller_address.parse() {
                Ok(addr) => addr,
                Err(_) => {
                    error!("[VenusHelper] Invalid Venus Comptroller address");
                    return;
                }
            };
            let abi: Abi = match File::open("src/abi/VenusComptroller.json") {
                Ok(f) => match serde_json::from_reader(f) {
                    Ok(a) => a,
                    Err(_) => {
                        error!("[VenusHelper] VenusComptroller ABI parse error");
                        return;
                    }
                },
                Err(_) => {
                    error!("[VenusHelper] VenusComptroller ABI file missing");
                    return;
                }
            };
            let provider = Provider::try_from("http://localhost:8545").unwrap();
            let client = provider.clone();
            let contract = Contract::new(venus_comptroller, abi, Arc::new(client));
            loop {
                match contract.method::<(), Vec<Address>>("getAllMarkets", ()).unwrap().call().await {
                    Ok(accounts) => {
                        use futures::stream::FuturesUnordered;
                        use tokio::sync::Semaphore;
                        use std::sync::Arc as StdArc;
                        let semaphore = StdArc::new(Semaphore::new(20));
                        let mut tasks = FuturesUnordered::new();
                        for account in accounts {
                            let contract = contract.clone();
                            let sender = _sender.clone();
                            let provider = provider.clone();
                            let semaphore = semaphore.clone();
                            tasks.push(tokio::spawn(async move {
                                let _permit = semaphore.acquire().await;
                                (account, contract.method::<Address, (U256, U256, U256)>("getAccountLiquidity", account).unwrap().call().await, contract.clone(), sender, provider)
                            }));
                        }
                        while let Some(Ok((account, liquidity_res, contract, sender, provider))) = tasks.next().await {
                            match liquidity_res {
                                Ok(_liquidity) => {
                                    if _liquidity.1 > U256::zero() { // shortfall > 0
                                        // Get all assets user is involved in
                                        match contract.method::<Address, Vec<Address>>("getAssetsIn", account).unwrap().call().await {
                                            Ok(assets) => {
                                                let mut total_debt_usd = 0.0;
                                                let mut total_collateral_usd = 0.0;
                                                let price_oracle_addr: Address = match env::var("VENUS_PRICE_ORACLE") {
                                                    Ok(addr) => addr.parse().unwrap(),
                                                    Err(_) => {
                                                        error!("[VenusHelper] VENUS_PRICE_ORACLE env var required");
                                                        continue;
                                                    }
                                                };
                                                let price_oracle_abi: Abi = match File::open("src/abi/VenusPriceOracle.json") {
                                                    Ok(f) => match serde_json::from_reader(f) {
                                                        Ok(a) => a,
                                                        Err(_) => {
                                                            error!("[VenusHelper] VenusPriceOracle ABI parse error");
                                                            continue;
                                                        }
                                                    },
                                                    Err(_) => {
                                                        error!("[VenusHelper] VenusPriceOracle ABI file missing");
                                                        continue;
                                                    }
                                                };
                                                let oracle = Contract::new(price_oracle_addr, price_oracle_abi, Arc::new(provider.clone()));
                                                let ctoken_abi: Abi = match File::open("src/abi/VenusCToken.json") {
                                                    Ok(f) => match serde_json::from_reader(f) {
                                                        Ok(a) => a,
                                                        Err(_) => {
                                                            error!("[VenusHelper] VenusCToken ABI parse error");
                                                            continue;
                                                        }
                                                    },
                                                    Err(_) => {
                                                        error!("[VenusHelper] VenusCToken ABI file missing");
                                                        continue;
                                                    }
                                                };
                                                for ctoken in assets {
                                                    let ctoken_contract = Contract::new(ctoken, ctoken_abi.clone(), Arc::new(provider.clone()));
                                                    match ctoken_contract.method::<Address, U256>("borrowBalanceStored", account).unwrap().call().await {
                                                        Ok(debt) => {
                                                            match ctoken_contract.method::<Address, U256>("balanceOfUnderlying", account).unwrap().call().await {
                                                                Ok(supply) => {
                                                                    match ctoken_contract.method::<(), Address>("underlying", ()).unwrap().call().await {
                                                                        Ok(_underlying_addr) => {
                                                                            match oracle.method::<Address, U256>("getUnderlyingPrice", ctoken).unwrap().call().await {
                                                                                Ok(price) => {
                                                                                    // Venus price is scaled by 1e18, debt/supply by token decimals
                                                                                    let debt_usd = debt.as_u128() as f64 * price.as_u128() as f64 / 1e36;
                                                                                    let supply_usd = supply.as_u128() as f64 * price.as_u128() as f64 / 1e36;
                                                                                    total_debt_usd += debt_usd;
                                                                                    total_collateral_usd += supply_usd;
                                                                                }
                                                                                Err(_) => {
                                                                                    error!("[VenusHelper] Error getting underlying price");
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(_) => {
                                                                            error!("[VenusHelper] Error getting underlying address");
                                                                        }
                                                                    }
                                                                }
                                                                Err(_) => {
                                                                    error!("[VenusHelper] Error getting supply");
                                                                }
                                                            }
                                                        }
                                                        Err(_) => {
                                                            error!("[VenusHelper] Error getting debt");
                                                        }
                                                    }
                                                }
                                                let event = LiquidationEvent {
                                                    protocol: "Venus".to_string(),
                                                    account: format!("{:?}", account), // Use Debug formatting for Address
                                                    debt: total_debt_usd,
                                                    collateral: total_collateral_usd,
                                                };
                                                if let Err(e) = sender.send(event).await {
                                                    error!("[VenusHelper] Failed to send liquidation event: {}", e);
                                                }
                                            }
                                            Err(_) => {
                                                error!("[VenusHelper] Error getting assets");
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    error!("[VenusHelper] Error getting liquidity");
                                }
                            }
                        }
                    }
                    Err(_) => {
                        error!("[VenusHelper] Error getting all markets");
                    }
                }
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }
}

pub struct AaveHelper {
    pub sender: mpsc::Sender<LiquidationEvent>,
}

impl ProtocolHelper for AaveHelper {
    fn protocol_name(&self) -> &'static str {
        "Aave"
    }
    fn spawn_detection(&self) {
        let _sender = self.sender.clone();
        
        
        tokio::spawn(async move {
            info!("[AaveHelper] Starting Aave mainnet liquidation monitoring");
            use ethers::prelude::*;
            use std::sync::Arc;
            use std::time::Duration;
            use reqwest::Client;
            use serde::Deserialize;
            let aave_lending_pool_address = match env::var("AAVE_LENDING_POOL") {
                Ok(addr) => addr,
                Err(_) => {
                    error!("[AaveHelper] AAVE_LENDING_POOL env var required");
                    return;
                }
            };
            let aave_lending_pool: Address = match aave_lending_pool_address.parse() {
                Ok(addr) => addr,
                Err(_) => {
                    error!("[AaveHelper] Invalid Aave LendingPool address");
                    return;
                }
            };
            let abi: Abi = match File::open("src/abi/AaveLendingPool.json") {
                Ok(f) => match serde_json::from_reader(f) {
                    Ok(a) => a,
                    Err(_) => {
                        error!("[AaveHelper] AaveLendingPool ABI parse error");
                        return;
                    }
                },
                Err(_) => {
                    error!("[AaveHelper] AaveLendingPool ABI file missing");
                    return;
                }
            };
            let provider = Provider::try_from("http://localhost:8545").unwrap();
            let client = provider.clone();
            let contract = Contract::new(aave_lending_pool, abi, Arc::new(client));
            loop {
                // Fetch users from The Graph (all pages)
                let accounts = match fetch_aave_users_from_graph().await {
                    Ok(users) => users,
                    Err(e) => {
                        log::error!("[AaveHelper] Error fetching users from The Graph: {}", e);
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        continue;
                    }
                };

                use futures::stream::{FuturesUnordered, StreamExt};
                use tokio::sync::Semaphore;
                use std::sync::Arc as StdArc;
                let semaphore = StdArc::new(Semaphore::new(20)); // Limit concurrency to 20
                let mut tasks = FuturesUnordered::new();
                for account in accounts {
                    let contract = contract.clone();
                    let sender = _sender.clone();
                    let provider = provider.clone();
                    let semaphore = semaphore.clone();
                    tasks.push(tokio::spawn(async move {
                        let _permit = semaphore.acquire().await;
                        let result = contract.method::<Address, (ethers::types::U256, ethers::types::U256, ethers::types::U256, ethers::types::U256, ethers::types::U256, ethers::types::U256, bool)>("getUserAccountData", account).unwrap().call().await;
                        (account, result, sender, provider)
                    }));
                }
                while let Some(Ok((account, result, sender, provider))) = tasks.next().await {
                    match result {
                        Ok(data) => {
                            let total_collateral_eth = data.0;
                            let total_debt_eth = data.1;
                            let health = data.5;
                            if health < U256::from(1_000_000_000_000_000_000u64) { // health factor < 1.0
                                // Optionally convert ETH to USD using price oracle
                                let price_oracle_addr: Address = match env::var("AAVE_PRICE_ORACLE") {
                                    Ok(addr) => addr.parse().unwrap(),
                                    Err(_) => {
                                        error!("[AaveHelper] AAVE_PRICE_ORACLE env var required");
                                        continue;
                                    }
                                };
                                let price_oracle_abi: Abi = match File::open("src/abi/AavePriceOracle.json") {
                                    Ok(f) => match serde_json::from_reader(f) {
                                        Ok(a) => a,
                                        Err(_) => {
                                            error!("[AaveHelper] AavePriceOracle ABI parse error");
                                            continue;
                                        }
                                    },
                                    Err(_) => {
                                        error!("[AaveHelper] AavePriceOracle ABI file missing");
                                        continue;
                                    }
                                };
                                let oracle = Contract::new(price_oracle_addr, price_oracle_abi, StdArc::new(provider.clone()));
                                match oracle.method::<String, ethers::types::U256>("getAssetPrice", String::from("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE")).unwrap().call().await {
                                    Ok(eth_usd) => {
                                        let collateral_usd = total_collateral_eth.as_u128() as f64 * eth_usd.as_u128() as f64 / 1e36;
                                        let debt_usd = total_debt_eth.as_u128() as f64 * eth_usd.as_u128() as f64 / 1e36;
                                        let event = LiquidationEvent {
                                            protocol: "Aave".to_string(),
                                            account: format!("{:?}", account),
                                            debt: debt_usd,
                                            collateral: collateral_usd,
                                        };
                                        if let Err(e) = sender.send(event).await {
                                            log::error!("[AaveHelper] Failed to send liquidation event: {}", e);
                                        }
                                    }
                                    Err(_) => {
                                        log::error!("[AaveHelper] Error getting ETH price");
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            log::error!("[AaveHelper] Error getting user account data");
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }
}

// Helper: Fetch Aave users from The Graph (paginated)
async fn fetch_aave_users_from_graph() -> Result<Vec<Address>, Box<dyn std::error::Error + Send + Sync>> {
    // ... existing code ...
}

// Helper: Fetch Venus users from The Graph (paginated)
async fn fetch_venus_users_from_graph() -> Result<Vec<Address>, Box<dyn std::error::Error + Send + Sync>> {
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
    loop {
        let query = format!(r#"{{ accounts(first: 1000, where: {{hasBorrowed: true, id_gt: "{}"}}) {{ id }} }}"#, last_id);
        let req_body = serde_json::json!({ "query": query });
        let resp = client.post("https://api.thegraph.com/subgraphs/name/venusprotocol/venus")
            .json(&req_body)
            .send()
            .await?;
        let resp_json: GraphResponse = resp.json().await?;
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
async fn fetch_compound_users_from_graph() -> Result<Vec<Address>, Box<dyn std::error::Error + Send + Sync>> {
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
    loop {
        let query = format!(r#"{{ accounts(first: 1000, where: {{hasBorrowed: true, id_gt: "{}"}}) {{ id }} }}"#, last_id);
        let req_body = serde_json::json!({ "query": query });
        let resp = client.post("https://api.thegraph.com/subgraphs/name/graphprotocol/compound-v2")
            .json(&req_body)
            .send()
            .await?;
        let resp_json: GraphResponse = resp.json().await?;
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
    // (function body already defined above, remove this duplicate)

pub struct CompoundHelper {
    pub sender: mpsc::Sender<LiquidationEvent>,
}

impl ProtocolHelper for CompoundHelper {
    fn protocol_name(&self) -> &'static str {
        "Compound"
    }
    fn spawn_detection(&self) {
        let _sender = self.sender.clone();
        
        
        tokio::spawn(async move {
            info!("[CompoundHelper] Starting Compound mainnet liquidation monitoring");
            use ethers::prelude::*;
            use std::sync::Arc;
            use std::time::Duration;
            let compound_comptroller_address = match env::var("COMPOUND_COMPTROLLER") {
                Ok(addr) => addr,
                Err(_) => {
                    error!("[CompoundHelper] COMPOUND_COMPTROLLER env var required");
                    return;
                }
            };
            let compound_comptroller: Address = match compound_comptroller_address.parse() {
                Ok(addr) => addr,
                Err(_) => {
                    error!("[CompoundHelper] Invalid Compound Comptroller address");
                    return;
                }
            };
            let abi: Abi = match File::open("src/abi/CompoundComptroller.json") {
                Ok(f) => match serde_json::from_reader(f) {
                    Ok(a) => a,
                    Err(_) => {
                        error!("[CompoundHelper] CompoundComptroller ABI parse error");
                        return;
                    }
                },
                Err(_) => {
                    error!("[CompoundHelper] CompoundComptroller ABI file missing");
                    return;
                }
            };
            let provider = Provider::try_from("http://localhost:8545").unwrap();
            let client = provider.clone();
            let contract = Contract::new(compound_comptroller, abi, Arc::new(client));
            loop {
                match contract.method::<(), Vec<Address>>("getAllMarkets", ()).unwrap().call().await {
                    Ok(accounts) => {
                        use futures::stream::FuturesUnordered;
                        use tokio::sync::Semaphore;
                        use std::sync::Arc as StdArc;
                        let semaphore = StdArc::new(Semaphore::new(20));
                        let mut tasks = FuturesUnordered::new();
                        for account in accounts {
                            let contract = contract.clone();
                            let sender = _sender.clone();
                            let provider = provider.clone();
                            let semaphore = semaphore.clone();
                            tasks.push(tokio::spawn(async move {
                                let _permit = semaphore.acquire().await;
                                (account, contract.method::<Address, (U256, U256, U256)>("getAccountLiquidity", account).unwrap().call().await, contract.clone(), sender, provider)
                            }));
                        }
                        while let Some(Ok((account, liquidity_res, contract, sender, provider))) = tasks.next().await {
                            match liquidity_res {
                                Ok(liquidity) => {
                                    // TODO: Add logic for processing liquidity
                                }
                                Err(e) => {
                                    log::error!("[CompoundHelper] Error getting liquidity: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[CompoundHelper] Error getting markets: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }
}
