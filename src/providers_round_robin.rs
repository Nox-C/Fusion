// Robust round-robin provider rotation with rate limiting and cooldowns
// This module is intended to be used by ProviderManager for safe, production-grade provider selection

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ProviderEntry {
    pub name: String,
    pub url: String,
    pub max_requests_per_minute: u32,
    pub monthly_limit: Option<u64>, // e.g., 3_000_000 for Infura
    pub hourly_limit: Option<u64>,  // calculated from monthly_limit
    pub daily_limit: Option<u64>,   // calculated from monthly_limit
    pub requests_this_window: u32,
    pub window_start: Option<Instant>,
    pub requests_this_hour: u64,
    pub hour_start: Option<chrono::DateTime<chrono::Utc>>,
    pub requests_today: u64,
    pub day_start: Option<chrono::DateTime<chrono::Utc>>,
    pub last_used: Option<Instant>,
    pub cooldown_until: Option<Instant>,
}

impl ProviderEntry {
    pub fn enforce_steady_limits(&mut self) -> bool {
        use chrono::Utc;
        let now = Utc::now();
        // Reset hour if needed
        if let Some(hour_start) = self.hour_start {
            let elapsed = now.timestamp() - hour_start.timestamp();
            if elapsed >= 3600 {
                self.requests_this_hour = 0;
                self.hour_start = Some(now);
            }
        } else {
            self.hour_start = Some(now);
            self.requests_this_hour = 0;
        }
        // Reset day if needed
        if let Some(day_start) = self.day_start {
            if now.date_naive() != day_start.date_naive() {
                self.requests_today = 0;
                self.day_start = Some(now);
            }
        } else {
            self.day_start = Some(now);
            self.requests_today = 0;
        }
        // Enforce hourly limit
        if let Some(hourly) = self.hourly_limit {
            if self.requests_this_hour >= hourly {
                return false;
            }
        }
        // Enforce daily limit
        if let Some(daily) = self.daily_limit {
            if self.requests_today >= daily {
                return false;
            }
        }
        true
    }
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

                if entry.is_available()
                    && entry.requests_this_window < entry.max_requests_per_minute
                {
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
                    monthly_limit: Some(0),
                    hourly_limit: Some(0),
                    daily_limit: Some(0),
                    requests_this_window: 0,
                    window_start: None,
                    requests_this_hour: 0,
                    hour_start: Some(chrono::Utc::now()),
                    requests_today: 0,
                    day_start: Some(chrono::Utc::now()),
                    last_used: None,
                    cooldown_until: None,
                },
                ProviderEntry {
                    name: "B".into(),
                    url: "urlB".into(),
                    max_requests_per_minute: 2,
                    monthly_limit: Some(0),
                    hourly_limit: Some(0),
                    daily_limit: Some(0),
                    requests_this_window: 0,
                    window_start: None,
                    requests_this_hour: 0,
                    hour_start: Some(chrono::Utc::now()),
                    requests_today: 0,
                    day_start: Some(chrono::Utc::now()),
                    last_used: None,
                    cooldown_until: None,
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
            vec![ProviderEntry {
    monthly_limit: Some(0),
    hourly_limit: Some(0),
    daily_limit: Some(0),
    requests_this_hour: 0,
    hour_start: Some(chrono::Utc::now()),
    requests_today: 0,
    day_start: Some(chrono::Utc::now()),
                name: "A".into(),
                url: "urlA".into(),
                max_requests_per_minute: 2,
                last_used: None,
                cooldown_until: None,
                requests_this_window: 0,
                window_start: None,
            }],
            Duration::from_secs(60),
        );
        assert!(rotation.next_provider().is_some());
        rotation.mark_provider_failure("A", Duration::from_secs(1));
        assert!(rotation.next_provider().is_none());
        sleep(Duration::from_secs(1));
        assert!(rotation.next_provider().is_some());
    }
}
