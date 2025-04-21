use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::analysis::AnalysisHub;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DexPrice {
    pub dex: String,
    pub price: f64,
    pub timestamp: u64, // unix millis
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ArbitrageOpportunity {
    pub matrix_id: String,
    pub chain: String,
    pub buy_dex: String,
    pub sell_dex: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub marginal_optimizer_pct: f64,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Matrix {
    pub id: String,
    pub name: String,
    pub chain: String, // "ETH" or "BSC"
    pub marginal_optimizer: f64,
    pub dex_prices: HashMap<String, DexPrice>, // key = DEX name
    pub opportunities: Vec<String>, // Replace with real struct if you have one
    pub recent_transactions: Vec<String>, // Replace with real struct if you have one
    pub status: String,
}


use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MatrixManager {
    pub matrices: Arc<Mutex<Vec<Matrix>>>,
}

impl MatrixManager {
    pub fn new() -> Self {
        let mut eth_prices = HashMap::new();
        eth_prices.insert("PancakeSwap".into(), DexPrice { dex: "PancakeSwap".into(), price: 0.0, timestamp: 0 });
        eth_prices.insert("Uniswap".into(), DexPrice { dex: "Uniswap".into(), price: 0.0, timestamp: 0 });
        let mut bsc_prices = HashMap::new();
        bsc_prices.insert("PancakeSwap".into(), DexPrice { dex: "PancakeSwap".into(), price: 0.0, timestamp: 0 });
        bsc_prices.insert("Biswap".into(), DexPrice { dex: "Biswap".into(), price: 0.0, timestamp: 0 });
        let matrices = vec![
            Matrix {
                id: "matrix_eth_1".into(),
                name: "ETH Matrix 1".into(),
                chain: "ETH".into(),
                marginal_optimizer: 123.45,
                dex_prices: eth_prices,
                opportunities: vec!["Arb1".into(), "Arb2".into()],
                recent_transactions: vec!["Tx1".into(), "Tx2".into()],
                status: "Active".into(),
            },
            Matrix {
                id: "matrix_bsc_1".into(),
                name: "BSC Matrix 1".into(),
                chain: "BSC".into(),
                marginal_optimizer: 67.89,
                dex_prices: bsc_prices,
                opportunities: vec!["ArbA".into()],
                recent_transactions: vec!["TxA".into()],
                status: "Paused".into(),
            },
        ];
        Self {
            matrices: Arc::new(Mutex::new(matrices)),
        }
    }

    pub fn all(&self) -> Vec<Matrix> {
        self.matrices.lock().unwrap().clone()
    }

    /// Atomically update a DEX price for a given matrix (by id and dex name)
    pub fn update_dex_price(&self, matrix_id: &str, dex: &str, price: f64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let mut matrices = self.matrices.lock().unwrap();
        if let Some(matrix) = matrices.iter_mut().find(|m| m.id == matrix_id) {
            matrix.dex_prices.insert(
                dex.to_string(),
                DexPrice {
                    dex: dex.to_string(),
                    price,
                    timestamp: now,
                },
            );
        }
    }

    /// Scan for arbitrage opportunities in all matrices using the AnalysisHub
    pub fn scan_for_arbitrage_opportunities(&self, marginal_optimizer: f64) -> Vec<ArbitrageOpportunity> {
        let matrices = self.matrices.lock().unwrap();
        AnalysisHub::scan_all(&matrices, marginal_optimizer)
    }
}
