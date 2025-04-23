use serde::{Deserialize, Serialize};

use crate::analysis::AnalysisHub;
use crate::config::Settings;
use ethers::middleware::Middleware;
use ethers::types::{Address, U256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub flashloan_provider: Address,
    pub flashloan_amount: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Matrix {
    pub id: String,
    pub name: String,
    pub chain: String, // "ETH" or "BSC"
    pub marginal_optimizer: f64,
    pub dex_prices: HashMap<String, DexPrice>, // key = DEX name
    pub opportunities: Vec<ArbitrageOpportunity>,
    pub recent_transactions: Vec<Transaction>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub status: String,
}

#[derive(Clone)]
pub struct MatrixManager {
    pub matrices: Arc<Mutex<Vec<Matrix>>>,
}

impl MatrixManager {
    /// Create a new MatrixManager with a default ETH matrix. In production, matrices should be loaded from config or blockchain.
    pub fn new() -> Self {
        // Initialize with a default ETH matrix so manager.all() is not empty
        let mut initial_matrices = Vec::new();
        initial_matrices.push(Matrix {
            id: "ETH".to_string(),
            name: "ETH Matrix".to_string(),
            chain: "ETH".to_string(),
            marginal_optimizer: 0.0,
            dex_prices: HashMap::new(),
            opportunities: Vec::new(),
            recent_transactions: Vec::new(),
            status: "Active".to_string(),
        });
        Self {
            matrices: Arc::new(Mutex::new(initial_matrices)),
        }
    }
    // Add a method to load matrices from config or blockchain as needed.
}

impl Default for MatrixManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MatrixManager {
    pub fn all(&self) -> Vec<Matrix> {
        self.matrices.lock().unwrap().clone()
    }

    /// Atomically update a DEX price for a given matrix (by id and dex name)
    pub fn update_dex_price(&self, matrix_id: &str, dex: &str, price: f64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
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

    /// Async scan for arbitrage opportunities in all matrices with flashloan integration.
    pub async fn scan_for_arbitrage_opportunities<M: Middleware + 'static>(
        &self,
        settings: &Settings,
        client: Arc<M>,
    ) -> Vec<ArbitrageOpportunity> {
        let matrices = self.matrices.lock().unwrap();
        AnalysisHub::scan_all(&matrices, settings, client).await
    }
}
