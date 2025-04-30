use std::sync::Arc;
use tokio::sync::Mutex;
use crate::shared_state::SharedState;

#[derive(Debug, Clone)]
pub struct LiquidationEvent {
    pub protocol: String,
    pub account: String,
    pub debt: f64,
    pub collateral: f64,
}

use tokio::sync::mpsc;
use async_trait::async_trait;



#[async_trait]
pub trait LiquidationExecutor: Clone + Send + Sync + 'static {
    async fn execute(&self, event: &LiquidationEvent);
}
// Note: async_trait ensures the returned future is Send if the implementor is Send.

pub struct LiquidationMonitor<E> {
    receiver: mpsc::Receiver<LiquidationEvent>,
    executor: E,
}

impl<E: LiquidationExecutor> LiquidationMonitor<E> {
    pub fn new_with_rx_and_executor(receiver: mpsc::Receiver<LiquidationEvent>, executor: E) -> Self {
        Self { receiver, executor }
    }

    pub async fn run(mut self, _shared: Arc<Mutex<SharedState>>) {
        while let Some(event) = self.receiver.recv().await {
            log::info!("[LiquidationMonitor] Received liquidation opportunity from {}: account={}, debt={}, collateral={}", event.protocol, event.account, event.debt, event.collateral);
            // Call the executor (must be async)
            // For the mock, just call executor.execute(&event).await
            #[allow(unused_must_use)]
            {
                let exec = self.executor.clone();
                tokio::spawn(async move {
                    exec.execute(&event).await;
                });
            }
        }
    }
}

