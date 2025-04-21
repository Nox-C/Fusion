use std::time::{SystemTime, UNIX_EPOCH};
use crate::matrix::{DexPrice, ArbitrageOpportunity, Matrix};

/// The AnalysisHub is responsible for running arbitrage analysis on all matrices.
pub struct AnalysisHub;

impl AnalysisHub {
    /// Scan a single matrix for arbitrage opportunities by running down each column (DEX) for the same asset.
    /// Returns all profitable opportunities where the price difference exceeds the threshold (relative to the lower price).
    pub fn scan_matrix(matrix: &Matrix, marginal_optimizer: f64) -> Vec<ArbitrageOpportunity> {
        let mut opps = Vec::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let prices: Vec<&DexPrice> = matrix.dex_prices.values().collect();
        for (i, a) in prices.iter().enumerate() {
            for b in prices.iter().skip(i + 1) {
                let price_diff = (a.price - b.price).abs();
                let min_price = a.price.min(b.price);
                if min_price > 0.0 {
                    let diff_pct = price_diff / min_price;
                    if diff_pct > marginal_optimizer {
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
                        };
                        opps.push(opp);
                    }
                }
            }
        }
        opps
    }

    /// Scan all matrices and return all opportunities.
    pub fn scan_all(matrices: &[Matrix], marginal_optimizer: f64) -> Vec<ArbitrageOpportunity> {
        let mut results = Vec::new();
        for matrix in matrices {
            results.extend(Self::scan_matrix(matrix, marginal_optimizer));
        }
        results
    }
}
