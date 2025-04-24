use crate::config::Settings;
use ethers::abi::Abi;
use ethers::middleware::Middleware;
use ethers::prelude::*;
use log::{error, warn};
use once_cell::sync::Lazy;
use serde_json;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FlashloanProvider {
    pub name: String,
    pub address: Address,
}

impl FlashloanProvider {
    pub fn from_config(name: &str, address: &str) -> Option<Self> {
        address.parse::<Address>().ok().map(|addr| Self {
            name: name.to_string(),
            address: addr,
        })
    }
    /// Construct provider from a config entry of format "name:address" or just address.
    pub fn from_entry(entry: &str) -> Option<Self> {
        let parts: Vec<&str> = entry.splitn(2, ':').collect();
        if parts.len() == 2 {
            let name = parts[0].to_string();
            let addr = parts[1].parse::<Address>().ok()?;
            Some(Self {
                name,
                address: addr,
            })
        } else {
            let addr = entry.parse::<Address>().ok()?;
            Some(Self {
                name: entry.to_string(),
                address: addr,
            })
        }
    }
}

/// Query the available liquidity for a given asset from a flashloan provider contract.
pub async fn query_liquidity<M: Middleware + 'static>(
    provider: &FlashloanProvider,
    asset: Address,
    client: Arc<M>,
) -> Option<U256> {
    // Lazy-loaded ERC20 ABI for flashloan queries
    static ERC20_ABI: Lazy<Abi> = Lazy::new(|| {
        serde_json::from_str(include_str!("abi/ERC20.json")).expect("ABI parse error")
    });

    // Query liquidity using cached ERC20 ABI
    let erc20 = Contract::new(asset, ERC20_ABI.clone(), client.clone());
    match erc20.method::<_, U256>("balanceOf", provider.address) {
        Ok(call) => match call.call().await {
            Ok(v) => Some(v),
            Err(e) => {
                error!("Error querying liquidity for {}: {}", provider.name, e);
                None
            }
        },
        Err(e) => {
            error!(
                "Error building balanceOf method for {}: {}",
                provider.name, e
            );
            None
        }
    }
}

/// Pure helper: choose best flashloan provider based on available liquidity and usage percentage.
pub fn choose_best_provider(
    liquidities: &[(Address, U256)],
    usage_percent: f64,
) -> Option<(Address, U256)> {
    if liquidities.is_empty() {
        warn!("No liquidity data provided to choose_best_provider");
        return None;
    }
    let mut best: Option<(Address, U256)> = None;
    for (addr, liquidity) in liquidities {
        let liq_f64 = liquidity.as_u128() as f64;
        let usage_amount = U256::from((usage_percent * liq_f64) as u128);
        if best.is_none() || usage_amount > best.as_ref().unwrap().1 {
            best = Some((*addr, usage_amount));
        }
    }
    best
}

/// Select the best flashloan provider and amount (using liquidity_usage_percentage).
pub async fn select_best_flashloan_provider<M: Middleware + 'static>(
    settings: &Settings,
    client: Arc<M>,
    asset: Address,
) -> Option<(Address, U256)> {
    let mut best: Option<(Address, U256)> = None;
    for entry in &settings.flash_loan_providers {
        if let Some(provider) = FlashloanProvider::from_entry(entry) {
            if let Some(liquidity) = query_liquidity(&provider, asset, client.clone()).await {
                let liq_f64 = liquidity.as_u128() as f64;
                let usage = U256::from((settings.liquidity_usage_percentage * liq_f64) as u128);
                if best.is_none() || usage > best.as_ref().unwrap().1 {
                    best = Some((provider.address, usage));
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::{Address, U256};

    fn addr_from_u64(n: u64) -> Address {
        Address::from_low_u64_be(n)
    }

    #[test]
    fn test_choose_best_provider_empty() {
        assert!(choose_best_provider(&[], 0.45).is_none());
    }

    #[test]
    fn test_choose_best_provider_single() {
        let addr = addr_from_u64(1);
        let res = choose_best_provider(&[(addr, U256::from(100u64))], 0.45).unwrap();
        assert_eq!(res.0, addr);
        assert_eq!(res.1, U256::from(45u64));
    }

    #[test]
    fn test_choose_best_provider_multiple() {
        let a1 = addr_from_u64(1);
        let a2 = addr_from_u64(2);
        let data = vec![(a1, U256::from(100u64)), (a2, U256::from(200u64))];
        let res = choose_best_provider(&data, 0.45).unwrap();
        assert_eq!(res.0, a2);
        assert_eq!(res.1, U256::from(90u64));
    }
}
