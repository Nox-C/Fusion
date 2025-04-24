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
    /// Create a new MatrixManager with ETH and BSC matrices pre-configured from settings.
    pub fn with_settings(settings: &Settings) -> Self {
        let mut matrices = Vec::new();
        // ETH matrix (placeholder for future use)
        matrices.push(Matrix {
            id: "ETH".to_string(),
            name: "ETH Matrix".to_string(),
            chain: "ETH".to_string(),
            marginal_optimizer: settings.marginal_optimizer,
            dex_prices: HashMap::new(),
            opportunities: Vec::new(),
            recent_transactions: Vec::new(),
            status: "Active".to_string(),
        });
        // BSC matrix configured with all DEXes
        let mut bsc_matrix = Matrix {
            id: "BSC".to_string(),
            name: "BSC Matrix".to_string(),
            chain: "BSC".to_string(),
            marginal_optimizer: settings.marginal_optimizer,
            dex_prices: HashMap::new(),
            opportunities: Vec::new(),
            recent_transactions: Vec::new(),
            status: "Active".to_string(),
        };
        for dex in &settings.dexes {
            bsc_matrix.dex_prices.insert(
                dex.clone(),
                DexPrice { dex: dex.clone(), price: 0.0, timestamp: 0 },
            );
        }
        matrices.push(bsc_matrix);
        Self { matrices: Arc::new(Mutex::new(matrices)) }
    }
}

impl Default for MatrixManager {
    fn default() -> Self {
        Self::with_settings(&Settings::default())
    }
}

// Add `new` alias to default constructor for tests
impl MatrixManager {
    /// Construct a new MatrixManager using default settings
    pub fn new() -> Self {
        MatrixManager::default()
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
        let matrices_vec = {
            let matrices = self.matrices.lock().unwrap();
            matrices.clone()
        };
        AnalysisHub::scan_all(&matrices_vec, settings, client).await
    }
}
