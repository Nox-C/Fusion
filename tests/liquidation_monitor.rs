
use fusion::liquidation_monitor::{LiquidationMonitor, LiquidationEvent, LiquidationExecutor};
use std::sync::Arc;
use fusion::shared_state::SharedState;
use tokio::sync::mpsc;

// Mock protocol helper for testing
struct MockProtocolHelper {
    protocol: &'static str,
    sender: mpsc::Sender<LiquidationEvent>,
}
impl MockProtocolHelper {
    async fn simulate_detection(&self) {
        // Simulate detection of a liquidation opportunity
        let event = LiquidationEvent {
            protocol: self.protocol.to_string(),
            account: "0xliquidate".to_string(),
            debt: 1000.0,
            collateral: 1200.0,
        };
        self.sender.send(event).await.unwrap();
    }
}

// Mock executor for testing

use async_trait::async_trait;

#[derive(Clone)]
struct MockArbitrageExecutor {
    called_with: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
}
impl MockArbitrageExecutor {
    fn new() -> Self {
        Self { called_with: Arc::new(tokio::sync::Mutex::new(vec![])) }
    }
    async fn was_called_with(&self, protocol: &str) -> bool {
        self.called_with.lock().await.contains(&protocol.to_string())
    }
}

#[async_trait]
impl LiquidationExecutor for MockArbitrageExecutor {
    async fn execute(&self, event: &fusion::liquidation_monitor::LiquidationEvent) {
        self.called_with.lock().await.push(event.protocol.clone());
    }
}

#[tokio::test]
async fn test_multi_protocol_liquidation_monitoring() {
    // 1. Setup shared state and communication channel
    let shared = Arc::new(tokio::sync::Mutex::new(SharedState::default()));
    let (tx, rx) = mpsc::channel::<LiquidationEvent>(10);

    // 2. Create helpers for each protocol
    let venus_helper = MockProtocolHelper { protocol: "Venus", sender: tx.clone() };
    let aave_helper = MockProtocolHelper { protocol: "Aave", sender: tx.clone() };

    // 3. Spawn helpers (simulate detection)
    tokio::spawn(async move { venus_helper.simulate_detection().await; });
    tokio::spawn(async move { aave_helper.simulate_detection().await; });

    // 4. Create mock executor and monitor
    let executor = MockArbitrageExecutor::new();
    let monitor = LiquidationMonitor::new_with_rx_and_executor(rx, executor.clone());

    // 5. Run monitor for a short period
    let handle = tokio::spawn(monitor.run(shared.clone()));
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    handle.abort();

    // 6. Assert: executor was called with both protocols
    assert!(executor.was_called_with("Venus").await);
    assert!(executor.was_called_with("Aave").await);
}

