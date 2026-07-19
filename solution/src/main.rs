use keyed_rate_limiter::RateLimiter;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TOL: Duration = Duration::from_millis(80);

fn main() {
    req1_steady_client();
    req2_token_bucket();
    req3_retry_after();
    req4_blocking_acquire();
    req5_per_key_isolation();
    req6_sliding_window();
    req7_concurrency();
    req8_idle_cleanup();

    println!("\nAll requirements passed.");
}

fn stamp() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = d.as_secs();

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        (secs / 3600) % 24, // hours
        (secs / 60) % 60,   // minutes
        secs % 60,          // seconds
        d.subsec_millis(),
    )
}

fn log(msg: impl AsRef<str>) {
    println!("[{}] {}", stamp(), msg.as_ref());
}

fn header(n: u32, title: &str) {
    println!("\nRequirement {n}: {title}")
}

fn verdict(o: &keyed_rate_limiter::Outcome) -> &'static str {
    if o.allowed {
        "ALLOW"
    } else {
        "DENY"
    }
}

// FIrst requirement
fn req1_steady_client() {
    header(
        1,
        "a steady client below the refill rate is never throttled",
    );
    // capacity 5, refill 2/sec => one permit every 500ms. Requesting every
    // 600ms stays comfortably under that, so the bucket never empties.
    let rl = RateLimiter::token_bucket(5.0, 2.0);
    for i in 1..=6 {
        let out = rl.try_acquire("client-a");
        log(format!("client-a request {}: {}", i, verdict(&out)));
        assert!(out.allowed, "steady client should never be denied");
        if i < 6 {
            thread::sleep(Duration::from_millis(600));
        }
    }
}

// Second requirement

fn req2_token_bucket() {
    header(
        2,
        "token bucket: burst to capacity, refill-paced recovery, cost",
    );
    // (a) 8 requests in the same instant, capacity 5 => 5 allow, 3 deny.
    println!("  (a) burst of 8 against capacity 5:");
    let rl = RateLimiter::token_bucket(5.0, 2.0);
    let mut allowed = 0;
    let mut denied = 0;
    for i in 1..=8 {
        let out = rl.try_acquire("client-b");
        log(format!("client-b request {}: {}", i, verdict(&out)));
        if out.allowed {
            allowed += 1
        } else {
            denied += 1
        }
    }
    assert_eq!(
        (allowed, denied),
        (5, 3),
        "full bucket absorbs exactly capacity"
    );

    // (b) after exhaustion, recovery is paced one permit / 500ms at refill 2.
    println!("  (b) recovery paced by refill (~500ms/permit at refill=2):");
    let rl = RateLimiter::token_bucket(5.0, 2.0);
    for _ in 0..5 {
        rl.try_acquire("client-c");
    }
    let mut last = Instant::now();
    for i in 1..=3 {
        // waits out the denial, then takes the permit that just refilled
        let out = loop {
            let o = rl.try_acquire("client-c");
            if o.allowed {
                break o;
            }
            thread::sleep(o.retry_after.unwrap());
        };
        let gap = last.elapsed();
        last = Instant::now();
        log(format!(
            "client-c refill {}: {} (+{}ms since previous)",
            i,
            verdict(&out),
            gap.as_millis()
        ));
        let off = gap.as_millis() as i64 - 500;
        assert!(
            off.unsigned_abs() as u128 <= TOL.as_millis(),
            "pacing ~500ms, got {}ms",
            gap.as_millis()
        );
    }

    // (c) a cost=5 request drains the whole budget in one step.
    println!("  (c) cost=5 drains the budget in a single call:");
    let rl = RateLimiter::token_bucket(5.0, 0.0);
    let big = rl.try_acquire_cost("client-d", 5);
    log(format!(
        "client-d cost=5: {} (bucket now empty)",
        verdict(&big)
    ));
    let after = rl.try_acquire("client-d");
    log(format!(
        "client-d cost=1: {} (nothing left)",
        verdict(&after)
    ));
    assert!(big.allowed && !after.allowed);
}

// Third requirement
fn req3_retry_after() {
    header(
        3,
        "a denial says when to come back, and that prediction holds",
    );
    let rl = RateLimiter::token_bucket(1.0, 2.0);
    assert!(rl.try_acquire("client-e").allowed);

    let denied = rl.try_acquire("client-e");
    let predicted = denied.retry_after_ms().unwrap();
    log(format!(
        "client-e: {} retry_after_ms={}",
        verdict(&denied),
        predicted
    ));
    assert!(!denied.allowed);

    let start = Instant::now();
    thread::sleep(Duration::from_millis(predicted as u64));
    let retry = rl.try_acquire("client-e");
    let actual = start.elapsed().as_millis();
    log(format!(
        "client-e after waiting: {} (predicted {}ms, actual waited {}ms)",
        verdict(&retry),
        predicted,
        actual
    ));
    assert!(
        retry.allowed,
        "the request should succeed once the predicted wait elapses"
    );
}

// Fourth requirement
fn req4_blocking_acquire() {
    header(
        4,
        "blocking acquire: one waits and wins, one gives up at its deadline",
    );
    let rl = RateLimiter::token_bucket(1.0, 2.0);

    assert!(rl.try_acquire("client-f").allowed); // exhaust
    log("client-f exhausted; calling acquire(timeout=2000ms)...");
    let start = Instant::now();
    let out = rl.acquire("client-f", Duration::from_millis(2000));
    let waited = start.elapsed();
    log(format!(
        "client-f: {} after waiting {}ms",
        verdict(&out),
        waited.as_millis()
    ));
    assert!(out.allowed, "2s is plenty for a 500ms refill");
    assert!(
        waited >= Duration::from_millis(500) - TOL,
        "it really did wait for the refill"
    );

    let timeout = Duration::from_millis(100);
    log("client-f exhausted again; calling acquire(timeout=100ms)...");
    let start = Instant::now();
    let out = rl.acquire("client-f", timeout);
    let waited = start.elapsed();
    log(format!(
        "client-f: {} after waiting {}ms (deadline was 100ms)",
        verdict(&out),
        waited.as_millis()
    ));
    assert!(
        !out.allowed,
        "next permit (~500ms) is past the 100ms deadline"
    );
    assert!(
        waited >= timeout - TOL,
        "must not give up before the deadline"
    );
    assert!(
        waited <= timeout + TOL,
        "must not overshoot beyond tolerance"
    );
}

// Fifth requirement
fn req5_per_key_isolation() {
    header(
        5,
        "per-key isolation: one exhausted key does not affect another",
    );
    let rl = RateLimiter::token_bucket(3.0, 0.0); // no refill, so exhaustion sticks
    for i in 1..=3 {
        let out = rl.try_acquire("alice");
        log(format!("alice request {}: {}", i, verdict(&out)));
        assert!(out.allowed);
    }
    let alice = rl.try_acquire("alice");
    log(format!("alice request 4: {} (exhausted)", verdict(&alice)));
    let bob = rl.try_acquire("bob");
    log(format!(
        "bob   request 1: {} (unaffected, same instant)",
        verdict(&bob)
    ));
    assert!(!alice.allowed && bob.allowed);
}

// Requirement 6
fn req6_sliding_window() {
    header(
        6,
        "sliding window: grants age out continuously, not on a fixed reset",
    );
    let rl = RateLimiter::sliding_window(3, Duration::from_millis(1000));
    let t0 = Instant::now();
    let at = |t0: Instant| t0.elapsed().as_millis();

    thread::sleep(Duration::from_millis(900));
    for i in 1..=3 {
        let out = rl.try_acquire("client-g");
        log(format!("t≈{}ms  grant {}: {}", at(t0), i, verdict(&out)));
        assert!(out.allowed);
    }

    thread::sleep(Duration::from_millis(200));
    let denied = rl.try_acquire("client-g");
    log(format!(
        "t≈{}ms  grant 4: {} retry_after_ms={} (window still full)",
        at(t0),
        verdict(&denied),
        denied.retry_after_ms().unwrap()
    ));
    assert!(!denied.allowed);

    thread::sleep(Duration::from_millis(1000));
    let out = rl.try_acquire("client-g");
    log(format!(
        "t≈{}ms  grant 5: {} (oldest grants aged out)",
        at(t0),
        verdict(&out)
    ));
    assert!(out.allowed);
}

// Seventh requirement
fn req7_concurrency() {
    header(7, "exact admission under concurrency: no over-admission");
    let rl = Arc::new(RateLimiter::token_bucket(10.0, 0.0));
    let n = 20;
    let barrier = Arc::new(Barrier::new(n));
    let allowed = Arc::new(AtomicU64::new(0));
    let denied = Arc::new(AtomicU64::new(0));

    let handles: Vec<_> = (0..n)
        .map(|_| {
            let rl = Arc::clone(&rl);
            let barrier = Arc::clone(&barrier);
            let allowed = Arc::clone(&allowed);
            let denied = Arc::clone(&denied);
            thread::spawn(move || {
                barrier.wait(); // all 20 threads slam the key at once
                if rl.try_acquire("shared").allowed {
                    allowed.fetch_add(1, Ordering::Relaxed);
                } else {
                    denied.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let a = allowed.load(Ordering::Relaxed);
    let d = denied.load(Ordering::Relaxed);
    let over = a.saturating_sub(10);
    log(format!(
        "20 concurrent callers, capacity 10 => granted={} denied={} over-admission={}",
        a, d, over
    ));
    assert_eq!(
        (a, d, over),
        (10, 10, 0),
        "exactly capacity granted, no over-admission"
    );
}

// 8Th requirement
fn req8_idle_cleanup() {
    header(
        8,
        "idle-key cleanup: untouched keys are evicted, active ones kept",
    );
    // 200ms TTL keeps the demo quick; in production this would be tens of seconds.
    let rl = RateLimiter::token_bucket(5.0, 2.0).with_idle_ttl(Duration::from_millis(200));

    rl.try_acquire("a");
    rl.try_acquire("b");
    rl.try_acquire("c");
    log(format!(
        "touched a, b, c => tracked_keys={}",
        rl.tracked_keys()
    ));
    assert_eq!(rl.tracked_keys(), 3);

    // Keep "c" active across two intervals while a and b go idle.
    for _ in 0..2 {
        thread::sleep(Duration::from_millis(150));
        rl.try_acquire("c");
    }
    log("kept 'c' active for ~300ms; 'a' and 'b' left idle past the 200ms TTL");

    let removed = rl.evict_idle();
    log(format!(
        "evict_idle removed {} key(s) => tracked_keys={} (a tracked={}, b tracked={}, c tracked={})",
        removed,
        rl.tracked_keys(),
        rl.is_tracked("a"),
        rl.is_tracked("b"),
        rl.is_tracked("c"),
    ));
    assert_eq!(removed, 2);
    assert!(!rl.is_tracked("a") && !rl.is_tracked("b"));
    assert!(rl.is_tracked("c"), "the active key survives");
}
