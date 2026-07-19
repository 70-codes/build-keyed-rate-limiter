use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
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

    pub fn deny(retry_after: Option<Duration>) -> Self {
        Outcome {
            allowed: false,
            retry_after,
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

impl Strategy {
    fn fresh_state(&self, now: Instant) -> KeyState {
        match self {
            Strategy::TokenBucket { capacity, .. } => KeyState::TokenBucket {
                tokens: *capacity,
                last_refill: now,
            },

            Strategy::SlidingWindow { .. } => KeyState::SlidingWindow {
                grants: VecDeque::new(),
            },
        }
    }
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
            strategy: Strategy::SlidingWindow { limit, window },
            idle_ttl: None,
            keys: RwLock::new(HashMap::new()),
        }
    }

    pub fn try_acquire(&self, key: &str) -> Outcome {
        self.try_acquire_cost(key, 1)
    }

    pub fn try_acquire_cost(&self, key: &str, cost: u64) -> Outcome {
        let now = Instant::now();

        let entry = self.entry_for(key, now);
        let mut e = entry.lock().unwrap();
        e.last_activity = now;

        match self.strategy {
            Strategy::TokenBucket {
                capacity,
                refill_per_sec,
            } => {
                if let KeyState::TokenBucket {
                    tokens,
                    last_refill,
                } = &mut e.state
                {
                    token_bucket_try(
                        tokens,
                        last_refill,
                        capacity,
                        refill_per_sec,
                        now,
                        cost as f64,
                    )
                } else {
                    unreachable!("state/strategy mismatch")
                }
            }
            Strategy::SlidingWindow { limit, window } => {
                if let KeyState::SlidingWindow { grants } = &mut e.state {
                    sliding_window_try(grants, limit, window, now, cost)
                } else {
                    unreachable!("state/strategy mismatch")
                }
            }
        }
    }

    pub fn acquire(&self, key: &str, timeout: Duration) -> Outcome {
        self.acquire_cost(key, 1, timeout)
    }

    pub fn acquire_cost(&self, key: &str, cost: u64, timeout: Duration) -> Outcome {
        let deadline = Instant::now() + timeout;
        loop {
            let out = self.try_acquire_cost(key, cost);
            if out.allowed {
                return out;
            }
            let wait = match out.retry_after {
                None => return out,
                Some(w) => w,
            };
            let now = Instant::now();
            if now >= deadline {
                return out;
            }
            let remaining = deadline - now;
            thread::sleep(wait.min(remaining));
        }
    }

    pub fn with_idle_ttl(mut self, ttl: Duration) -> Self {
        self.idle_ttl = Some(ttl);
        self
    }

    pub fn tracked_keys(&self) -> usize {
        self.keys.read().unwrap().len()
    }

    pub fn is_tracked(&self, key: &str) -> bool {
        self.keys.read().unwrap().contains_key(key)
    }

    pub fn evict_idle(&self) -> usize {
        let ttl = match self.idle_ttl {
            Some(t) => t,
            None => return 0,
        };

        let now = Instant::now();

        let mut map = self.keys.write().unwrap();
        let before = map.len();

        map.retain(|_, entry| {
            let e = entry.lock().unwrap();
            now.duration_since(e.last_activity) <= ttl
        });
        before - map.len()
    }

    fn entry_for(&self, key: &str, now: Instant) -> Arc<Mutex<Entry>> {
        if let Some(e) = self.keys.read().unwrap().get(key) {
            return e.clone();
        }

        let mut map = self.keys.write().unwrap();
        map.entry(key.to_string())
            .or_insert_with(|| {
                Arc::new(Mutex::new(Entry {
                    state: self.strategy.fresh_state(now),
                    last_activity: now,
                }))
            })
            .clone()
    }
}

fn token_bucket_try(
    tokens: &mut f64,
    last_refill: &mut Instant,
    capacity: f64,
    refill_per_sec: f64,
    now: Instant,
    cost: f64,
) -> Outcome {
    let elapsed = now.duration_since(*last_refill).as_secs_f64();
    *tokens = (*tokens + elapsed * refill_per_sec).min(capacity);
    *last_refill = now;

    if cost > capacity {
        return Outcome::deny(None);
    }
    if *tokens >= cost {
        *tokens -= cost;
        Outcome::allow()
    } else {
        let deficit = cost - *tokens;
        let retry = if refill_per_sec > 0.0 {
            Some(Duration::from_secs_f64(deficit / refill_per_sec))
        } else {
            None
        };
        Outcome::deny(retry)
    }
}

fn sliding_window_try(
    grants: &mut VecDeque<Instant>,
    limit: u64,
    window: Duration,
    now: Instant,
    cost: u64,
) -> Outcome {
    while let Some(&front) = grants.front() {
        if front + window <= now {
            grants.pop_front();
        } else {
            break;
        }
    }

    if cost > limit {
        return Outcome::deny(None);
    }

    let count = grants.len() as u64;
    if count + cost <= limit {
        for _ in 0..cost {
            grants.push_back(now);
        }
        Outcome::allow()
    } else {
        let k = (count + cost - limit) as usize;
        let target = grants[k - 1];
        let retry = (target + window).saturating_duration_since(now);
        Outcome::deny(Some(retry))
    }
}
