// Round-robin rotation for DEX websocket endpoints
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct DexWebSocketEntry {
    pub name: String,
    pub url: String,
    pub max_reconnects_per_minute: u32,
    pub last_used: Option<Instant>,
    pub cooldown_until: Option<Instant>,
    pub reconnects_this_window: u32,
    pub window_start: Option<Instant>,
}

impl DexWebSocketEntry {
    pub fn is_available(&self) -> bool {
        match self.cooldown_until {
            Some(until) => Instant::now() >= until,
            None => true,
        }
    }
}

pub struct DexWebSocketRotation {
    endpoints: VecDeque<DexWebSocketEntry>,
    window: Duration,
}

impl DexWebSocketRotation {
    pub fn new(endpoints: Vec<DexWebSocketEntry>, window: Duration) -> Self {
        Self {
            endpoints: VecDeque::from(endpoints),
            window,
        }
    }

    pub fn next_endpoint(&mut self) -> Option<&mut DexWebSocketEntry> {
        let now = Instant::now();
        let len = self.endpoints.len();
        for _ in 0..len {
            if let Some(mut entry) = self.endpoints.pop_front() {
                // Reset window if needed
                if let Some(start) = entry.window_start {
                    if now.duration_since(start) > self.window {
                        entry.window_start = Some(now);
                        entry.reconnects_this_window = 0;
                    }
                } else {
                    entry.window_start = Some(now);
                    entry.reconnects_this_window = 0;
                }

                if entry.is_available()
                    && entry.reconnects_this_window < entry.max_reconnects_per_minute
                {
                    entry.last_used = Some(now);
                    entry.reconnects_this_window += 1;
                    self.endpoints.push_back(entry.clone());
                    return self.endpoints.back_mut();
                } else {
                    self.endpoints.push_back(entry);
                }
            }
        }
        None
    }

    pub fn mark_endpoint_failure(&mut self, name: &str, cooldown: Duration) {
        for entry in self.endpoints.iter_mut() {
            if entry.name == name {
                entry.cooldown_until = Some(Instant::now() + cooldown);
            }
        }
    }
}
