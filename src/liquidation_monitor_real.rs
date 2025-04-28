
use tokio::sync::mpsc;
use async_trait::async_trait;
use ethers::abi::Abi;
use crate::execute_arbitrage::execute_arbitrage_onchain;
use crate::liquidation_monitor::{LiquidationEvent, LiquidationExecutor, ProtocolHelper};
use crate::arbitrage_executor_address::ARBITRAGE_EXECUTOR_MAINNET;

#[derive(Clone)]
pub struct RealArbitrageExecutor {
    pub abi_path: String,
    pub profit_wallet: String,
}

#[async_trait]
impl LiquidationExecutor for RealArbitrageExecutor {
    async fn execute(&self, event: &LiquidationEvent) {
        let dry_run = std::env::var("DRY_RUN").unwrap_or_else(|_| "true".to_string()) == "true";
        log::info!("[RealArbitrageExecutor] {} liquidation for {} on {} (dry_run={})", if dry_run {"Simulating"} else {"Executing"}, event.account, event.protocol, dry_run);

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
        let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
        let rpc_url = std::env::var("BSC_RPC_URL").unwrap_or_else(|_| "https://bsc-dataseed.binance.org/".to_string());

        use chrono::Utc;
        if dry_run {
            log::info!("[DRY_RUN] Would call execute_arbitrage_onchain with: account={} debt={} collateral={}", event.account, event.debt, event.collateral);
            
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
                log::info!("[RealArbitrageExecutor] Submitted liquidation tx: 0x{:x}", tx_hash);
                
            }
            Err(e) => {
                log::error!("[RealArbitrageExecutor] Error executing liquidation: {}", e);
                
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
        let sender = self.sender.clone();
        
        
        tokio::spawn(async move {
            log::info!("[VenusHelper] Starting Venus mainnet liquidation monitoring");
            use ethers::prelude::*;
            use std::sync::Arc;
            use std::time::Duration;
            use crate::config::Settings;
            let venus_comptroller_address = match std::env::var("VENUS_COMPTROLLER") {
                Ok(addr) => addr,
                Err(_) => {
                    log::error!("[VenusHelper] VENUS_COMPTROLLER env var required");
                    return;
                }
            };
            let venus_comptroller: Address = match venus_comptroller_address.parse() {
                Ok(addr) => addr,
                Err(_) => {
                    log::error!("[VenusHelper] Invalid Venus Comptroller address");
                    return;
                }
            };
            let abi: Abi = match std::fs::File::open("src/abi/VenusComptroller.json") {
                Ok(f) => match serde_json::from_reader(f) {
                    Ok(a) => a,
                    Err(_) => {
                        log::error!("[VenusHelper] VenusComptroller ABI parse error");
                        return;
                    }
                },
                Err(_) => {
                    log::error!("[VenusHelper] VenusComptroller ABI file missing");
                    return;
                }
            };
            let provider = ethers::providers::Provider::try_from("http://localhost:8545").unwrap();
            let client = provider.clone();
            let contract = Contract::new(venus_comptroller, abi, std::sync::Arc::new(client));
            loop {
                match contract.method::<(), Vec<Address>>("getAllMarkets", ()).unwrap().call().await {
                    Ok(accounts) => {
                        for account in accounts {
                            match contract.method::<Address, (U256, U256)>("getAccountLiquidity", account).unwrap().call().await {
                                Ok(liquidity) => {
                                    if liquidity.1 > U256::zero() { // shortfall > 0
                                        // Get all assets user is involved in
                                        match contract.method::<Address, Vec<Address>>("getAssetsIn", account).unwrap().call().await {
                                            Ok(assets) => {
                                                let mut total_debt_usd = 0.0;
                                                let mut total_collateral_usd = 0.0;
                                                let price_oracle_addr: Address = match std::env::var("VENUS_PRICE_ORACLE") {
                                                    Ok(addr) => addr.parse().unwrap(),
                                                    Err(_) => {
                                                        log::error!("[VenusHelper] VENUS_PRICE_ORACLE env var required");
                                                        continue;
                                                    }
                                                };
                                                let price_oracle_abi: Abi = match std::fs::File::open("src/abi/VenusPriceOracle.json") {
                                                    Ok(f) => match serde_json::from_reader(f) {
                                                        Ok(a) => a,
                                                        Err(_) => {
                                                            log::error!("[VenusHelper] VenusPriceOracle ABI parse error");
                                                            continue;
                                                        }
                                                    },
                                                    Err(_) => {
                                                        log::error!("[VenusHelper] VenusPriceOracle ABI file missing");
                                                        continue;
                                                    }
                                                };
                                                let oracle = Contract::new(price_oracle_addr, price_oracle_abi, std::sync::Arc::new(provider.clone()));
                                                let ctoken_abi: Abi = match std::fs::File::open("src/abi/VenusCToken.json") {
                                                    Ok(f) => match serde_json::from_reader(f) {
                                                        Ok(a) => a,
                                                        Err(_) => {
                                                            log::error!("[VenusHelper] VenusCToken ABI parse error");
                                                            continue;
                                                        }
                                                    },
                                                    Err(_) => {
                                                        log::error!("[VenusHelper] VenusCToken ABI file missing");
                                                        continue;
                                                    }
                                                };
                                                for ctoken in assets {
                                                    let ctoken_contract = Contract::new(ctoken, ctoken_abi.clone(), std::sync::Arc::new(provider.clone()));
                                                    match ctoken_contract.method("borrowBalanceStored", account).unwrap().call().await {
                                                        Ok(debt) => {
                                                            match ctoken_contract.method("balanceOfUnderlying", account).unwrap().call().await {
                                                                Ok(supply) => {
                                                                    match ctoken_contract.method::<_, Address>("underlying", ()).unwrap().call().await {
                                                                        Ok(underlying_addr) => {
                                                                            match oracle.method("getUnderlyingPrice", ctoken).unwrap().call().await {
                                                                                Ok(price) => {
                                                                                    // Venus price is scaled by 1e18, debt/supply by token decimals
                                                                                    let debt_usd = debt.as_u128() as f64 * price.as_u128() as f64 / 1e36;
                                                                                    let supply_usd = supply.as_u128() as f64 * price.as_u128() as f64 / 1e36;
                                                                                    total_debt_usd += debt_usd;
                                                                                    total_collateral_usd += supply_usd;
                                                                                }
                                                                                Err(_) => {
                                                                                    log::error!("[VenusHelper] Error getting underlying price");
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(_) => {
                                                                            log::error!("[VenusHelper] Error getting underlying address");
                                                                        }
                                                                    }
                                                                }
                                                                Err(_) => {
                                                                    log::error!("[VenusHelper] Error getting supply");
                                                                }
                                                            }
                                                        }
                                                        Err(_) => {
                                                            log::error!("[VenusHelper] Error getting debt");
                                                        }
                                                    }
                                                }
                                                let event = LiquidationEvent {
                                                    protocol: "Venus".to_string(),
                                                    account: format!("0x{:x}", account),
                                                    debt: total_debt_usd,
                                                    collateral: total_collateral_usd,
                                                };
                                                if let Err(e) = sender.send(event).await {
                                                    log::error!("[VenusHelper] Failed to send liquidation event: {}", e);
                                                }
                                            }
                                            Err(_) => {
                                                log::error!("[VenusHelper] Error getting assets");
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    log::error!("[VenusHelper] Error getting liquidity");
                                }
                            }
                        }
                    }
                    Err(_) => {
                        log::error!("[VenusHelper] Error getting all markets");
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
        let sender = self.sender.clone();
        
        
        tokio::spawn(async move {
            log::info!("[AaveHelper] Starting Aave mainnet liquidation monitoring");
            use ethers::prelude::*;
            use std::sync::Arc;
            use std::time::Duration;
            let aave_lending_pool_address = match std::env::var("AAVE_LENDING_POOL") {
                Ok(addr) => addr,
                Err(_) => {
                    log::error!("[AaveHelper] AAVE_LENDING_POOL env var required");
                    return;
                }
            };
            let aave_lending_pool: Address = match aave_lending_pool_address.parse() {
                Ok(addr) => addr,
                Err(_) => {
                    log::error!("[AaveHelper] Invalid Aave LendingPool address");
                    return;
                }
            };
            let abi: Abi = match std::fs::File::open("src/abi/AaveLendingPool.json") {
                Ok(f) => match serde_json::from_reader(f) {
                    Ok(a) => a,
                    Err(_) => {
                        log::error!("[AaveHelper] AaveLendingPool ABI parse error");
                        return;
                    }
                },
                Err(_) => {
                    log::error!("[AaveHelper] AaveLendingPool ABI file missing");
                    return;
                }
            };
            let provider = ethers::providers::Provider::try_from("http://localhost:8545").unwrap();
            let client = provider.clone();
            let contract = Contract::new(aave_lending_pool, abi, std::sync::Arc::new(client));
            loop {
                match contract.method::<(), Vec<Address>>("getUsers", ()).unwrap().call().await {
                    Ok(accounts) => {
                        for account in accounts {
                            match contract.method::<Address, (U256, U256, U256, U256, U256, U256)>("getUserAccountData", account).unwrap().call().await {
                                Ok(data) => {
                                    let total_collateral_eth = data.0;
                                    let total_debt_eth = data.1;
                                    let health = data.5;
                                    if health < U256::from(1_000_000_000_000_000_000u64) { // health factor < 1.0
                                        // Optionally convert ETH to USD using price oracle
                                        let price_oracle_addr: Address = match std::env::var("AAVE_PRICE_ORACLE") {
                                            Ok(addr) => addr.parse().unwrap(),
                                            Err(_) => {
                                                log::error!("[AaveHelper] AAVE_PRICE_ORACLE env var required");
                                                continue;
                                            }
                                        };
                                        let price_oracle_abi: Abi = match std::fs::File::open("src/abi/AavePriceOracle.json") {
                                            Ok(f) => match serde_json::from_reader(f) {
                                                Ok(a) => a,
                                                Err(_) => {
                                                    log::error!("[AaveHelper] AavePriceOracle ABI parse error");
                                                    continue;
                                                }
                                            },
                                            Err(_) => {
                                                log::error!("[AaveHelper] AavePriceOracle ABI file missing");
                                                continue;
                                            }
                                        };
                                        let oracle = Contract::new(price_oracle_addr, price_oracle_abi, std::sync::Arc::new(provider.clone()));
                                        match oracle.method("getAssetPrice", String::from("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE")).unwrap().call().await {
                                            Ok(eth_usd) => {
                                                let collateral_usd = total_collateral_eth.as_u128() as f64 * eth_usd.as_u128() as f64 / 1e36;
                                                let debt_usd = total_debt_eth.as_u128() as f64 * eth_usd.as_u128() as f64 / 1e36;
                                                let event = LiquidationEvent {
                                                    protocol: "Aave".to_string(),
                                                    account: format!("0x{:x}", account),
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
                    }
                    Err(_) => {
                        log::error!("[AaveHelper] Error getting users");
                    }
                }
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }
}

pub struct CompoundHelper {
    pub sender: mpsc::Sender<LiquidationEvent>,
}

impl ProtocolHelper for CompoundHelper {
    fn protocol_name(&self) -> &'static str {
        "Compound"
    }
    fn spawn_detection(&self) {
        let sender = self.sender.clone();
        
        
        tokio::spawn(async move {
            log::info!("[CompoundHelper] Starting Compound mainnet liquidation monitoring");
            use ethers::prelude::*;
            use std::sync::Arc;
            use std::time::Duration;
            let compound_comptroller_address = match std::env::var("COMPOUND_COMPTROLLER") {
                Ok(addr) => addr,
                Err(_) => {
                    log::error!("[CompoundHelper] COMPOUND_COMPTROLLER env var required");
                    return;
                }
            };
            let compound_comptroller: Address = match compound_comptroller_address.parse() {
                Ok(addr) => addr,
                Err(_) => {
                    log::error!("[CompoundHelper] Invalid Compound Comptroller address");
                    return;
                }
            };
            let abi: Abi = match std::fs::File::open("src/abi/CompoundComptroller.json") {
                Ok(f) => match serde_json::from_reader(f) {
                    Ok(a) => a,
                    Err(_) => {
                        log::error!("[CompoundHelper] CompoundComptroller ABI parse error");
                        return;
                    }
                },
                Err(_) => {
                    log::error!("[CompoundHelper] CompoundComptroller ABI file missing");
                    return;
                }
            };
            let provider = ethers::providers::Provider::try_from("http://localhost:8545").unwrap();
            let client = provider.clone();
            let contract = Contract::new(compound_comptroller, abi, std::sync::Arc::new(client));
            loop {
                match contract.method::<(), Vec<Address>>("getAllMarkets", ()).unwrap().call().await {
                    Ok(accounts) => {
                        for account in accounts {
                            match contract.method::<Address, (U256, U256)>("getAccountLiquidity", account).unwrap().call().await {
                                Ok(liquidity) => {
                                    if liquidity.1 > U256::zero() { // shortfall > 0
                                        let mut total_debt_usd = 0.0;
                                        let mut total_collateral_usd = 0.0;
                                        let price_oracle_addr: Address = std::env::var("COMPOUND_PRICE_ORACLE").expect("COMPOUND_PRICE_ORACLE env var required").parse().unwrap();
                                        let price_oracle_abi: Abi = serde_json::from_reader(std::fs::File::open("abi/CompoundPriceOracle.json").unwrap()).unwrap();
                                        let oracle = Contract::new(price_oracle_addr, price_oracle_abi, std::sync::Arc::new(provider.clone()));
                                        let ctoken_abi: Abi = serde_json::from_reader(std::fs::File::open("abi/CompoundCToken.json").unwrap()).unwrap();
                                        match contract.method::<Address, Vec<Address>>("getAssetsIn", account).unwrap().call().await {
                                            Ok(assets) => {
                                                for ctoken in assets {
                                                    let ctoken_contract = Contract::new(ctoken, ctoken_abi.clone(), std::sync::Arc::new(provider.clone()));
                                                    match ctoken_contract.method("borrowBalanceStored", account).unwrap().call().await {
                                                        Ok(debt) => {
                                                            match ctoken_contract.method("balanceOfUnderlying", account).unwrap().call().await {
                                                                Ok(supply) => {
                                                                    match ctoken_contract.method::<_, Address>("underlying", ()).unwrap().call().await {
                                                                        Ok(underlying_addr) => {
                                                                            match oracle.method("getUnderlyingPrice", ctoken).unwrap().call().await {
                                                                                Ok(price) => {
                                                                                    // Compound price is scaled by 1e18, debt/supply by token decimals
                                                                                    let debt_usd = debt.as_u128() as f64 * price.as_u128() as f64 / 1e36;
                                                                                    let supply_usd = supply.as_u128() as f64 * price.as_u128() as f64 / 1e36;
                                                                                    total_debt_usd += debt_usd;
                                                                                    total_collateral_usd += supply_usd;
                                                                                }
                                                                                Err(_) => {
                                                                                    log::error!("[CompoundHelper] Error getting underlying price");
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(_) => {
                                                                            log::error!("[CompoundHelper] Error getting underlying address");
                                                                        }
                                                                    }
                                                                }
                                                                Err(_) => {
                                                                    log::error!("[CompoundHelper] Error getting supply");
                                                                }
                                                            }
                                                        }
                                                        Err(_) => {
                                                            log::error!("[CompoundHelper] Error getting debt");
                                                        }
                                                    }
                                                }
                                                let event = LiquidationEvent {
                                                    protocol: "Compound".to_string(),
                                                    account: format!("0x{:x}", account),
                                                    debt: total_debt_usd,
                                                    collateral: total_collateral_usd,
                                                };
                                                if let Err(e) = sender.send(event).await {
                                                    log::error!("[CompoundHelper] Failed to send liquidation event: {}", e);
                                                }
                                            }
                                            Err(_) => {
                                                log::error!("[CompoundHelper] Error getting assets");
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    log::error!("[CompoundHelper] Error getting liquidity");
                                }
                            }
                        }
                        };
                        if let Err(e) = sender.send(event).await {
                            log::error!("[CompoundHelper] Failed to send liquidation event: {}", e);
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }
}

