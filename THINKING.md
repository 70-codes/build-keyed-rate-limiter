# Thinking Log

<!-- This is your scratchpad. Fill it in AS YOU GO, not at the end.
     Rough, fragmentary, honest. Don't polish it.
     Read the README for guidance on how to use this file. -->

## Initial Reaction

I need two algorithms behind the same API and also check up the logic used by the algorithms for a deeper understanding
I have to read on suitability between SystemTime and instant
The concurrency requirement will probably be the trickiest part. I need to guarantee that multiple callers can't over-admit permits while also avoiding unnecessary contention between unrelated API keys
Timing is also important. I already know I don't want to use wall-clock time because adjustments to the system clock would affect refill calculations

## Plan

<!-- Still before coding (or right at the start).
     - How will you structure this? Files, types, main components.
     - What are the key design decisions you're making up front?
     - What are you deliberately deferring?
     - What will you build FIRST — the smallest slice that proves something useful? -->
I'll create a folder named solution and inside the folder i'll have the src folder which will contain the library crate as lib.rs and an entry point for implementing the library as main.rs which is where the library will be consumed
I'll use instant for refill calculations instead of SystemTime because refill math shouldn't depend on wall clock. (instant documentation - https://doc.rust-lang.org/std/time/struct.Instant.html)

- get API shape right first
- implement token bucket
- then retry_after + blocking acquire
- sliding window after that
- cleanup last

Not touching concurrency until basic limiter works.

## Progress Notes

<!-- Drop an entry any time you:
     - change direction from your plan
     - hit something unexpected
     - make a trade-off
     - realise you were wrong about something
     - finish a chunk and start the next

     One or two sentences each is fine. Timestamp each one.
     Imagine your pair partner just asked "what are you doing?" — answer that.
     Add as many entries as you need. -->

### [08:26]
Initialized the repo by creating the files and filled up some section of the thinking.md

### [09:21]
Started with lib.rs. Want to settle the public API before implementing the algorithms.

### [10:01]
Finished designing sketching the public API and set the contructors in place as well so the library can already create limiter instances, behavior methods are todo's for now, will implement the alogorithms at a time

### [11:06]
Added the per-key lookup helper instead of duplicating the map logic everywhere. Ended up matching on the strategy after locking the entry so both algorithms can share the same flow
I also changed the implementation order slightly since both algorithms share the same entry path hance it was easire to get both non-blocking strategies working first and then build blocking acquire on top
Had to change deny to take Option<Duration> — my first version forced a concrete Duration, but a request that can never succeed (cost > capacity, or refill=0) has no retry time to give. None = "don't bother waiting.

### [11:36]
Was thinking acquire() would be its own implementation but it's really just retry + sleep around try_acquire_cost() finished the cleanup helpers toothe lib now feels feature complete, now I can move on to validating behaviour

## Research / References

<!-- Optional. Any docs, articles, past code, or language references you looked at.
     A one-line note on what you took from each is enough. -->
Instant- Documentation url (https://doc.rust-lang.org/std/time/struct.Instant.html) - Contained what i exactly needed which is a monotonic clock

Duration - Documentation url (https://doc.rust-lang.org/std/time/struct.Duration.html) - Mostly checking helper methods.

VecDeque - https://doc.rust-lang.org/std/collections/struct.VecDeque.html - Needed a queue where old timestamps come off the front and new grants are appended to the back, for the sliding window

## Retrospective

<!-- After you're done. This section is NOT optional — it's one of the most
     valuable parts of the submission. Be honest.

     - What's the weakest part of your solution? Where's the duct tape?
     - Where would this break in production?
     - What would you do differently with more time?
     - What surprised you about this problem?
     - Anything you tried and threw away? Why? -->
