use std::sync::Arc;
use tokio::sync::Mutex;
use crate::execution_log::ExecutionLog;
use crate::shared_state::SharedState;
use std::collections::HashMap;


pub async fn run_ai_controller(
    execution_log: Arc<ExecutionLog>,
    shared_state: Arc<Mutex<SharedState>>,
) {
    loop {
        // Analyze the last 100 executions (or all if fewer)
        let recent = execution_log.recent(100);
        if recent.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            continue;
        }
        // Compute per-protocol profit
        let mut protocol_profit: HashMap<String, f64> = HashMap::new();
        let mut _total_profit = 0.0;
        for rec in &recent {
            *protocol_profit.entry(rec.protocol.clone()).or_insert(0.0) += rec.profit;
            _total_profit += rec.profit;
        }
        // Find the best protocol
        let (best_protocol, best_profit) = protocol_profit.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(p, v)| (p.clone(), *v)).unwrap_or(("".to_string(), 0.0));
        // Dynamically adjust min_profit_threshold
        let mut new_min_profit = 0.0;
        for rec in &recent {
            if rec.success && rec.profit > new_min_profit {
                new_min_profit = rec.profit;
            }
        }
        // AI logic: adjust scan intervals based on profitability
        let mut new_scan_intervals = HashMap::new();
        for (protocol, profit) in &protocol_profit {
            let base = 60u64;
            let interval = if *profit > 0.0 {
                (base as f64 / (profit.abs().sqrt() + 1.0)).max(10.0) as u64
            } else {
                base * 2
            };
            new_scan_intervals.insert(protocol.clone(), interval);
        }
        // Update shared state
        {
            let mut state = shared_state.lock().await;
            state.min_profit_threshold = new_min_profit;
            // Weight best protocol highest, others lower
            for (protocol, _profit) in &protocol_profit {
                state.protocol_weights.insert(protocol.clone(), if *protocol == best_protocol { 1.0 } else { 0.5 });
            }
            // Update scan intervals if changed
            for (protocol, interval) in &new_scan_intervals {
                let prev = state.scan_intervals.get(protocol).cloned().unwrap_or(60);
                if *interval != prev {
                    log::info!("[AIController] Adjusted scan_interval for {}: {} -> {}", protocol, prev, interval);
                    state.scan_intervals.insert(protocol.clone(), *interval);
                }
            }
        }
        log::info!("[AIController] Set min_profit_threshold to {}. Best protocol: {} (profit {}). Scan intervals: {:?}", new_min_profit, best_protocol, best_profit, new_scan_intervals);
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    }
}
