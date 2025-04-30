use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub timestamp: DateTime<Utc>,
    pub protocol: String,
    pub account: String,
    pub debt: f64,
    pub collateral: f64,
    pub success: bool,
    pub profit: f64,
    pub gas_used: Option<u64>,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct ExecutionLog {
    pub records: Arc<Mutex<Vec<ExecutionRecord>>>,
}

impl ExecutionLog {
    pub fn new() -> Self {
        Self { records: Arc::new(Mutex::new(Vec::new())) }
    }
    pub fn log(&self, record: ExecutionRecord) {
        let mut records = self.records.lock().unwrap();
        records.push(record);
    }
    pub fn recent(&self, n: usize) -> Vec<ExecutionRecord> {
        let records = self.records.lock().unwrap();
        records.iter().rev().take(n).cloned().collect()
    }
    pub fn all(&self) -> Vec<ExecutionRecord> {
        let records = self.records.lock().unwrap();
        records.clone()
    }
}
