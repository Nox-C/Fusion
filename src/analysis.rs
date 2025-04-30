use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::Settings;
// Legacy matrix scanning removed. Only Matrix2D is used.

use crate::matrix2d::{Matrix2D, PriceCell};

/// Scan the Matrix2D for arbitrage opportunities
pub fn scan_matrix2d(matrix: &Matrix2D, profit_threshold_pct: f64) -> Vec<(String, String, String, f64, String, f64, f64, u64)> {
    use log::info;
    // Print the full price matrix for visibility
    info!("[DEX SCAN] Current price matrix:");
    for (dex_idx, dex) in matrix.dexes.iter().enumerate() {
        let prices: Vec<String> = matrix.prices[dex_idx]
            .iter()
            .map(|cell| format!("{}@{}", cell.price, cell.timestamp))
            .collect();
        info!("[DEX SCAN] {}: {}", dex, prices.join(", "));
    }
    
    let mut opps = Vec::new();
    for (asset_idx, asset) in matrix.assets.iter().enumerate() {
        // Find best (lowest) buy and best (highest) sell price for this asset
        let mut best_buy: Option<(usize, &PriceCell)> = None;
        let mut best_sell: Option<(usize, &PriceCell)> = None;
        for (dex_idx, _dex) in matrix.dexes.iter().enumerate() {
            let cell = &matrix.prices[dex_idx][asset_idx];
            if cell.price > 0.0 {
                if best_buy.is_none() || cell.price < best_buy.as_ref().unwrap().1.price {
                    best_buy = Some((dex_idx, cell));
                }
                if best_sell.is_none() || cell.price > best_sell.as_ref().unwrap().1.price {
                    best_sell = Some((dex_idx, cell));
                }
            }
        }
        if let (Some((buy_idx, buy_cell)), Some((sell_idx, sell_cell))) = (best_buy, best_sell) {
            if sell_cell.price > buy_cell.price {
                let profit_pct = (sell_cell.price - buy_cell.price) / buy_cell.price * 100.0;
                if profit_pct >= profit_threshold_pct {
                    opps.push((
                        matrix.dexes[buy_idx].clone(),
                        asset.clone(),
                        matrix.dexes[sell_idx].clone(),
                        buy_cell.price,
                        asset.clone(),
                        sell_cell.price,
                        profit_pct,
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
                    ));
                }
            }
        }
    }
    for (buy_dex, asset, sell_dex, buy_price, _, sell_price, profit_pct, ts) in &opps {
        info!("[ARBITRAGE OPP] Buy {} on {} at {} | Sell on {} at {} | Profit: {:.2}% @ {}", asset, buy_dex, buy_price, sell_dex, sell_price, profit_pct, ts);
    }
    opps
}

use ethers::middleware::Middleware;
use std::sync::Arc;
/// The AnalysisHub is responsible for running arbitrage analysis on all matrices.
pub struct AnalysisHub;

impl AnalysisHub {
    /// Scan a single matrix for arbitrage opportunities by running down each column (DEX) for the same asset.
    /// Returns all profitable opportunities where the price difference exceeds the threshold (relative to the lower price).
    /// Scan a single Matrix2D for arbitrage opportunities (async version for interface compatibility)
    pub async fn scan_matrix2d_async<M: Middleware + 'static>(
        matrix: &Matrix2D,
        profit_threshold_pct: f64,
        _settings: &Settings,
        _client: Arc<M>,
    ) -> Vec<(String, String, String, f64, String, f64, f64, u64)> {
        // Just call the sync version for now
        scan_matrix2d(matrix, profit_threshold_pct)
    }

    /// Scan all matrices and return all opportunities.
    /// Scan all Matrix2D instances and return all arbitrage opportunities (async version for interface compatibility)
    pub async fn scan_all_matrix2d_async<M: Middleware + 'static>(
        matrices: &[Matrix2D],
        profit_threshold_pct: f64,
        settings: &Settings,
        client: Arc<M>,
    ) -> Vec<(String, String, String, f64, String, f64, f64, u64)> {
        let mut all_opps = Vec::new();
        for matrix in matrices {
            let mut opps = Self::scan_matrix2d_async(matrix, profit_threshold_pct, settings, client.clone()).await;
            all_opps.append(&mut opps);
        }
        // Only collect and return tuple-based arbitrage opportunities for Matrix2D
        all_opps
    }
}
