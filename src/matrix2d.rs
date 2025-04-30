use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Matrix2D {
    pub dexes: Vec<String>,            // DEX names (rows)
    pub assets: Vec<String>,           // Asset symbols (columns)
    pub prices: Vec<Vec<PriceCell>>,   // [dex][asset] = PriceCell
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PriceCell {
    pub price: f64,
    pub timestamp: u64, // unix millis
}

impl Matrix2D {
    pub fn new(dexes: Vec<String>, assets: Vec<String>) -> Self {
        let prices = vec![vec![PriceCell { price: 0.0, timestamp: 0 }; assets.len()]; dexes.len()];
        Self { dexes, assets, prices }
    }

    pub fn update_price(&mut self, dex: &str, asset: &str, price: f64) {
        if let (Some(dex_idx), Some(asset_idx)) = (
            self.dexes.iter().position(|d| d == dex),
            self.assets.iter().position(|a| a == asset),
        ) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            self.prices[dex_idx][asset_idx] = PriceCell { price, timestamp: now };
        }
    }

    pub fn get_price(&self, dex: &str, asset: &str) -> Option<&PriceCell> {
        let dex_idx = self.dexes.iter().position(|d| d == dex)?;
        let asset_idx = self.assets.iter().position(|a| a == asset)?;
        self.prices.get(dex_idx)?.get(asset_idx)
    }
}
