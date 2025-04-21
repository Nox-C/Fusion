// Robust round-robin provider rotation with rate limiting and cooldowns
// This module is intended to be used by ProviderManager for safe, production-grade provider selection

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ProviderEntry {
    pub name: String,
    pub url: String,
    pub max_requests_per_minute: u32,
    pub last_used: Option<Instant>,
    pub cooldown_until: Option<Instant>,
    pub requests_this_window: u32,
    pub window_start: Option<Instant>,
}

impl ProviderEntry {
    pub fn is_available(&self) -> bool {
        match self.cooldown_until {
            Some(until) => Instant::now() >= until,
            None => true,
        }
    }
}

pub struct ProviderRotation {
    providers: VecDeque<ProviderEntry>,
    window: Duration,
}

impl ProviderRotation {
    pub fn new(providers: Vec<ProviderEntry>, window: Duration) -> Self {
        Self {
            providers: VecDeque::from(providers),
            window,
        }
    }

    pub fn next_provider(&mut self) -> Option<&mut ProviderEntry> {
        let now = Instant::now();
        let len = self.providers.len();
        for _ in 0..len {
            if let Some(mut entry) = self.providers.pop_front() {
                // Reset window if needed
                if let Some(start) = entry.window_start {
                    if now.duration_since(start) > self.window {
                        entry.window_start = Some(now);
                        entry.requests_this_window = 0;
                    }
                } else {
                    entry.window_start = Some(now);
                    entry.requests_this_window = 0;
                }

                if entry.is_available() && entry.requests_this_window < entry.max_requests_per_minute {
                    entry.last_used = Some(now);
                    entry.requests_this_window += 1;
                    self.providers.push_back(entry.clone());
                    return self.providers.back_mut();
                } else {
                    self.providers.push_back(entry);
                }
            }
        }
        None // All providers exhausted or on cooldown
    }

    pub fn mark_provider_failure(&mut self, name: &str, cooldown: Duration) {
        for entry in self.providers.iter_mut() {
            if entry.name == name {
                entry.cooldown_until = Some(Instant::now() + cooldown);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_round_robin_rotation_and_rate_limit() {
        let mut rotation = ProviderRotation::new(
            vec![
                ProviderEntry {
                    name: "A".into(),
                    url: "urlA".into(),
                    max_requests_per_minute: 2,
                    last_used: None,
                    cooldown_until: None,
                    requests_this_window: 0,
                    window_start: None,
                },
                ProviderEntry {
                    name: "B".into(),
                    url: "urlB".into(),
                    max_requests_per_minute: 2,
                    last_used: None,
                    cooldown_until: None,
                    requests_this_window: 0,
                    window_start: None,
                },
            ],
            Duration::from_secs(60),
        );
        // Each provider can be used twice per window
        assert_eq!(rotation.next_provider().unwrap().name, "A");
        assert_eq!(rotation.next_provider().unwrap().name, "B");
        assert_eq!(rotation.next_provider().unwrap().name, "A");
        assert_eq!(rotation.next_provider().unwrap().name, "B");
        // Now both are exhausted
        assert!(rotation.next_provider().is_none());
        // Simulate window reset
        sleep(Duration::from_millis(10));
        for entry in rotation.providers.iter_mut() {
            entry.window_start = Some(Instant::now() - Duration::from_secs(61));
        }
        assert!(rotation.next_provider().is_some());
    }

    #[test]
    fn test_provider_failure_and_cooldown() {
        let mut rotation = ProviderRotation::new(
            vec![
                ProviderEntry {
                    name: "A".into(),
                    url: "urlA".into(),
                    max_requests_per_minute: 2,
                    last_used: None,
                    cooldown_until: None,
                    requests_this_window: 0,
                    window_start: None,
                },
            ],
            Duration::from_secs(60),
        );
        assert!(rotation.next_provider().is_some());
        rotation.mark_provider_failure("A", Duration::from_secs(1));
        assert!(rotation.next_provider().is_none());
        sleep(Duration::from_secs(1));
        assert!(rotation.next_provider().is_some());
    }
}
