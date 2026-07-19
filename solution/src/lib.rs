use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub allowed: bool,
    pub retry_after: Option<Duration>,
}

impl Outcome {
    // retry_after in whole milliseconds rounded up
    // The contract is - wait atleast for this long where we
    // have a caller who sleeps the reported time and retries should find the request satisfiable

    pub fn retry_after_ms(&self) -> Option<u128> {
        self.retry_after.map(|d| d.as_nanos().div_ceil(1_000_000))
    }
    pub fn allow() -> Self {
        Outcome {
            allowed: true,
            retry_after: None,
        }
    }

    pub fn deny(retry_after: Duration) -> Self {
        Outcome {
            allowed: false,
            retry_after: Some(retry_after),
        }
    }
}

enum KeyState {
    TokenBucket { tokens: f64, last_refill: Instant },
    SlidingWindow { grants: VecDeque<Instant> },
}

#[derive(Debug, Clone, Copy)]
enum Strategy {
    TokenBucket { capacity: f64, refill_per_sec: f64 },
    SlidingWindow { limit: u64, window: Duration },
}

struct Entry {
    state: KeyState,
    last_activity: Instant,
}

pub struct RateLimiter {
    strategy: Strategy,
    idle_ttl: Option<Duration>,
    keys: RwLock<HashMap<String, Arc<Mutex<Entry>>>>,
}


impl RateLimiter {
    pub fn token_bucket(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            strategy: Strategy::TokenBucket {
                capacity,
                refill_per_sec,
            },
            idle_ttl: None,
            keys: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn sliding_window(limit: u64, window: Duration) -> Self {
        Self {
            strategy: Strategy::SlidingWindow {
                limit,
                window,
            },
            idle_ttl: None,
            keys: RwLock::new(HashMap::new()),
        }
    }

    pub fn try_acquire(&self, key: &str) -> Outcome {
        todo!()
    }

    pub fn try_acquire_cost(&self, key: &str, cost: u64) -> Outcome {
        todo!()
    }

    pub fn acquire(&self, key: &str, timeout: Duration) -> Outcome {
        todo!()
    }

    pub fn with_idle_ttl(mut self, ttl: Duration) -> Self {
        self.idle_ttl = Some(ttl);
        self
    }

    pub fn tracked_keys(&self) -> usize {
        todo!()
    }

    pub fn is_tracked(&self, key: &str) -> bool {
        todo!()
    }

    pub fn evict_idle(&self) -> usize {
        todo!()
    }
}

