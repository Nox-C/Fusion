use std::collections::HashMap;

pub struct SharedState {
    pub min_profit_threshold: f64,
    pub protocol_weights: HashMap<String, f64>,
    pub scan_intervals: HashMap<String, u64>, // in seconds
    // Add shared fields for coordination between modules
    // e.g., pub liquidation_events: Mutex<Vec<LiquidationEvent>>,
}

impl Default for SharedState {
    fn default() -> Self {
        let mut scan_intervals = HashMap::new();
        scan_intervals.insert("Venus".to_string(), 60);
        scan_intervals.insert("Aave".to_string(), 60);
        scan_intervals.insert("Compound".to_string(), 60);
        Self {
            min_profit_threshold: 0.0,
            protocol_weights: HashMap::new(),
            scan_intervals,
        }
    }
}
