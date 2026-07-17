# Task: Keyed Rate Limiter

> Read `README.md` first. It covers the working process, how to use `THINKING.md`, and the no-AI rule.

## Context

You are building the rate-limiting layer of a backend service. The service handles requests from many clients, each identified by an API key, and before doing any work for a request it asks the limiter: is this client allowed to proceed right now?

This is what the limiter is for:

- Stopping a misbehaving client (a runaway script, a retry loop) from overloading the service.
- Keeping one client from consuming capacity that is shared with all the others.
- Pacing the service's own outbound calls so it stays inside a third-party API's quota.

You need to build the limiter as an in-process library: the service imports it and calls it on the hot path of every request. There is no separate server and no external store; all state lives in the process's memory. Use only the standard library for the limiter itself (test frameworks and dev tooling are fine).

Use the language you are strongest in. Python is a reasonable default if you have no preference.

## What to hand in

1. The limiter library.
2. Tests that prove the eight requirements below. You can set up the tests however you want as long as you follow the format below.
3. Setup instructions: how to install and the one command that runs everything.

Alongside these, `THINKING.md` and your commit history, as the README describes. Those matter more than the code.

## Output format

Running your tests with one command (e.g. `python3 test.py`) should print a labeled section per requirement, with the log lines that prove the behavior underneath:

```
Requirement 1: steady client is never throttled
[14:03:07.512] client-a request 1: ALLOW
[14:03:08.014] client-a request 2: ALLOW
...

Requirement 2: burst up to capacity, then refill-paced
[14:03:09.021] client-b request 1: ALLOW
...
```

Start each section with `Requirement <n>`; the title after it is yours. Timestamp the log lines to the millisecond, since most of the behavior here happens at sub-second scale. Everything else about how you organize the tests is up to you.

## A note on timing

Run everything against the real clock: use your language's normal sleep and monotonic time functions (e.g. `time.sleep` and `time.monotonic` in Python). Do not mock or fake time. When you assert on a duration, allow a tolerance of around ±50 ms so the checks pass reliably run after run.

## Requirements

Each requirement comes with a code example showing the interface we expect. Treat these as sketches of the shape rather than exact signatures, and adapt them to your language's idioms.

---

### 1. The happy path: a steady client is not throttled

A client whose request rate stays below the refill rate sees no denials.

```python
limiter = RateLimiter(capacity=5, refill_per_sec=2)

limiter.try_acquire("client-a")   # allowed
```

Show us: a client making requests at a steady pace below the refill rate, with every request in the run allowed.

---

### 2. Token bucket: burst, refill, and cost

Each key has a bucket holding up to `capacity` permits, refilling continuously at `refill_per_sec`. A full bucket absorbs a burst of `capacity` requests; after that, requests are paced by the refill. A request can also declare a cost: `cost=5` consumes 5 permits at once.

```python
limiter.try_acquire("client-a")           # cost 1
limiter.try_acquire("client-a", cost=5)   # consumes 5 permits
```

Show us:

- 8 requests in the same instant with `capacity=5`: 5 allowed, 3 denied.
- After exhaustion, recovery paced by the refill: roughly one permit every 500 ms at `refill_per_sec=2`.
- A `cost=5` request visibly draining the budget in one step.

---

### 3. Denial tells the caller when to come back

`try_acquire` never blocks. When it denies a request, the result carries how long until the same request would succeed.

```python
result = limiter.try_acquire("client-a")
result.allowed          # False
result.retry_after_ms   # e.g. 496
```

Show us: a denial printing its `retry_after_ms`, then the same request succeeding once that much time has passed, with the predicted and actual waits side by side.

---

### 4. Blocking acquire with timeout

A caller can choose to wait for a permit instead of handling a denial, up to a deadline.

```python
limiter.acquire("client-a", timeout_ms=2000)   # blocks; returns success or failure
```

Show us: against an exhausted key, one call that waits and gets its permit (the wait visible in the timestamps), and one call with a short timeout that gives up at its deadline: not before it, and past it by no more than the tolerance you stated.

---

### 5. Per-key isolation

One client exhausting its budget must not affect any other client.

Show us: `"alice"` fully exhausted and denied while, in the same instant, a request for `"bob"` is allowed.

---

### 6. Second strategy: sliding window

The same interface, with the algorithm selected at construction. The rule: at most `limit` grants inside **any** trailing window of `window_ms`, measured backwards from the current moment. A grant only stops counting against the client once it is more than `window_ms` old.

```python
limiter = RateLimiter(strategy="sliding_window", limit=3, window_ms=1000)
```

Concretely, with `limit=3, window_ms=1000`: three requests granted at t=900 ms mean a request at t=1100 ms is denied, and requests keep being denied until those grants age out, just after t=1900 ms. (A counter that resets at fixed boundaries would have said yes at t=1001 ms, admitting six requests in a fraction of a second. That is the hole this strategy closes; you do not need to implement the broken version.)

Both strategies support the full interface: `cost`, `retry_after_ms`, blocking `acquire`, and idle-key eviction. What `cost=5` means for a window is your call to make; write it down in `THINKING.md`.

Show us: the timeline above. Grants at t≈900 ms, a denial at t≈1100 ms, a grant again shortly after t≈1900 ms.

---

### 7. Exact admission under concurrency

The limiter must be safe to call from many threads, goroutines, or async tasks at once: no over-admission, no crashes, no lost state.

Show us: 20 concurrent callers against one key holding 10 permits, all released at the same moment (a barrier or your language's equivalent), and the tally: 10 granted, 10 denied, over-admission 0. A fresh bucket with `capacity=10` and `refill_per_sec=0` keeps the race clean, since nothing refills mid-test.

The callers need to be genuinely concurrent in whatever model your language uses; a sequential loop does not count. If over-admission is structurally impossible in your model (a single-threaded event loop, for example), demonstrate the scenario anyway and explain in `THINKING.md` why it cannot happen in your design.

---

### 8. Idle-key cleanup

The limiter keeps per-key state (the bucket or the window log) for every key it has ever seen. That must not grow without bound: keys with no activity for `idle_ttl_ms` get evicted. Whether eviction runs on an explicit call or automatically in the background is your choice; document it. What counts as activity (do denied requests reset the idle timer?) is also your call.

```python
limiter = RateLimiter(capacity=5, refill_per_sec=2, idle_ttl_ms=60_000)

limiter.evict_idle()
```

Show us, as one sequence: keys `"a"` and `"b"` in use; both idle past the TTL while `"c"` stays active; after cleanup, `"a"` and `"b"` are no longer tracked and `"c"` still is (a tracked-key count before and after is enough).

One consequence to take a position on in `THINKING.md`: a key evicted with an empty bucket comes back with a full one if the client returns. Decide whether that is acceptable for your limiter, and why.

---

## Things to reason about

These are real design decisions without a single right answer. Engage with them in `THINKING.md` as you work:

- Refill accounting: computed arithmetically on each call, or advanced by a background tick? Compare precision, lock contention, and the cost of idle keys.
- When a permit becomes available and several callers are blocked, who wakes up, and how?
- Sliding window implementation: a log of grant timestamps is exact but costs memory per key; a bucketed approximation is cheap but inaccurate at boundaries. Which do you pick, and for which users is that the right call?
- Your timing checks run on the real clock. What would it take to make them deterministic instead? Is an injectable clock worth the extra API surface?
- Clock choice inside the limiter: what happens to your arithmetic if wall-clock time jumps backward while the process is running?
- Is `retry_after_ms` a guarantee or an estimate? Under contention, another caller can take the permit before the denied caller comes back.
- What should happen when a request's `cost` exceeds `capacity`, so it can never succeed?
- Locking: one lock around the whole key registry, or a lock per key? At what request rate does the difference start to matter?

If you make a call and later change your mind, write that down. Those moments are some of the most valuable data in the submission.

## You're done when

- [ ] All eight requirements are implemented, or `THINKING.md` documents what you did not get to and why.
- [ ] One command runs your tests, the output is labeled `Requirement 1` through `Requirement 8`, and each behavior can be verified from the log lines under its label.
- [ ] `THINKING.md` is filled in across all four sections, including the retrospective.
- [ ] Someone can clone your fork, follow your setup instructions, and run everything without asking you anything.

Now open `THINKING.md` and write down your first reaction.
