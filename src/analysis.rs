use crate::config::Settings;
use crate::flashloan::{FlashloanProvider, choose_best_provider, query_liquidity};
use crate::matrix::{ArbitrageOpportunity, DexPrice, Matrix};
use ethers::middleware::Middleware;
use ethers::types::{Address, U256};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

/// The AnalysisHub is responsible for running arbitrage analysis on all matrices.
pub struct AnalysisHub;

impl AnalysisHub {
    /// Scan a single matrix for arbitrage opportunities by running down each column (DEX) for the same asset.
    /// Returns all profitable opportunities where the price difference exceeds the threshold (relative to the lower price).
    pub async fn scan_matrix<M: Middleware + 'static>(
        matrix: &Matrix,
        settings: &Settings,
        client: Arc<M>,
    ) -> Vec<ArbitrageOpportunity> {
        info!("[SCAN] Matrix: {} | Chain: {} | DEXes: {}", matrix.name, matrix.chain, matrix.dex_prices.keys().cloned().collect::<Vec<_>>().join(", "));
        for (dex, price) in &matrix.dex_prices {
            info!("[SCAN]   DEX: {} | Price: {} | Timestamp: {}", dex, price.price, price.timestamp);
        }
        let mut opps = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let prices: Vec<&DexPrice> = matrix.dex_prices.values().collect();
        for (i, a) in prices.iter().enumerate() {
            for b in prices.iter().skip(i + 1) {
                let price_diff = (a.price - b.price).abs();
                let min_price = a.price.min(b.price);
                if min_price > 0.0 {
                    let diff_pct = price_diff / min_price;
                    if diff_pct > settings.marginal_optimizer {
                        let (buy_dex, buy_price, sell_dex, sell_price) = if a.price < b.price {
                            (a.dex.clone(), a.price, b.dex.clone(), b.price)
                        } else {
                            (b.dex.clone(), b.price, a.dex.clone(), a.price)
                        };
                        let opp = ArbitrageOpportunity {
                            matrix_id: matrix.id.clone(),
                            chain: matrix.chain.clone(),
                            buy_dex,
                            sell_dex,
                            buy_price,
                            sell_price,
                            marginal_optimizer_pct: diff_pct * 100.0,
                            timestamp: now,
                            flashloan_provider: {
                                let asset_address =
                                    matrix.id.parse::<Address>().unwrap_or(Address::zero());
                                let mut candidates = Vec::new();
                                for entry in &settings.flash_loan_providers {
                                    if let Some(provider) = FlashloanProvider::from_entry(entry) {
                                        if let Some(liq) = query_liquidity(
                                            &provider,
                                            asset_address,
                                            client.clone(),
                                        )
                                        .await
                                        {
                                            candidates.push((provider.address, liq));
                                        }
                                    }
                                }
                                choose_best_provider(
                                    &candidates,
                                    settings.liquidity_usage_percentage,
                                )
                                .map(|(addr, _)| addr)
                                .unwrap_or(Address::zero())
                            },
                            flashloan_amount: {
                                let asset_address =
                                    matrix.id.parse::<Address>().unwrap_or(Address::zero());
                                let mut candidates = Vec::new();
                                for entry in &settings.flash_loan_providers {
                                    if let Some(provider) = FlashloanProvider::from_entry(entry) {
                                        if let Some(liq) = query_liquidity(
                                            &provider,
                                            asset_address,
                                            client.clone(),
                                        )
                                        .await
                                        {
                                            candidates.push((provider.address, liq));
                                        }
                                    }
                                }
                                choose_best_provider(
                                    &candidates,
                                    settings.liquidity_usage_percentage,
                                )
                                .map(|(_, amount)| amount)
                                .unwrap_or(U256::zero())
                            },
                        };
                        opps.push(opp);
                    }
                }
            }
        }
        opps
    }

    /// Scan all matrices and return all opportunities.
    pub async fn scan_all<M: Middleware + 'static>(
        matrices: &[Matrix],
        settings: &Settings,
        client: Arc<M>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut results = Vec::new();
        for matrix in matrices {
            results.extend(Self::scan_matrix(matrix, settings, client.clone()).await);
        }
        results
    }
}
