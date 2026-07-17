Welcome 👋. You've been invited to do a paid coding task for Enki's **AI training data program**. Read this whole file first, then head to [`TASK.md`](./TASK.md) for the actual problem.

---

## What we're doing and why you're here

We're building a dataset of **expert reasoning traces** for training better coding models. What we're paying for is a faithful record of _how_ a strong engineer actually thinks through a real problem: the false starts, the trade-offs weighed out loud, the moment you realise your first plan was wrong and pivot.

The code you ship matters, but the reasoning trace around it matters just as much. A rough-but-honest thinking log paired with an okay solution is worth far more to us than a polished solution with no thinking attached.

Concretely, we care about three artefacts:

1. Your **commits** — frequent, meaningful, explaining _why_.
2. Your **`THINKING.md`** — a running scratchpad filled out as you work.
3. Your **code** — it should run and behave correctly.

In that order of importance.

---

## How to start

1. **Fork this repo.** Work in your fork, that's where you'll submit from.
2. **Read `TASK.md` end to end.** Don't start coding yet.
3. **Open `THINKING.md` and start filling in the "Initial Reaction" section** before you touch any code. First five minutes, gut reaction, what feels hard, what you'd rule out. This is where we see your instincts, don't skip it.
4. **Plan before you build.** Write the plan down in `THINKING.md`. It's fine if you change it later (in fact, please note when you do and why).
5. **Research is welcome.** Docs, Stack Overflow, your own old projects, language references. Jot a quick note in `THINKING.md` about what you looked up and what you took from it.
6. **Then build.** Put your code in a `solution/` directory (or whatever layout makes sense for the language you pick).

---

## No AI assistance — this is the important one

**Do not use Claude, ChatGPT, Copilot, Cursor, Cody, Windsurf, Gemini, or any other AI coding tool while working on this task.** Not for the code, not for the thinking log, not for "just the boilerplate". Your editor's built-in language server and autocomplete are fine.

This isn't about gatekeeping. The entire value of what you produce is that it's a **genuine human reasoning trace**.

If you're used to working with AI and find this hard, that's fair and honest. Note it in `THINKING.md`. That's useful signal too.

Normal tools are fine: language servers, linters, test runners, formatters, your editor's built-in autocomplete, docs, search engines, your own notes.

---

## `THINKING.md` — this is the main deliverable

Treat `THINKING.md` as a **scratchpad**, not a report. Write like you're thinking out loud to a colleague sitting next to you. Rough sentences, fragments, half-formed thoughts you later cross out, all great. Do not polish it. Do not write it retroactively at the end. Polished and performative is worse than messy and real.

The file has four sections. Fill them in **as you go**, not at the end:

- **Initial Reaction** — before any code. Your gut take on the problem. What's the hard part? What would you not do?
- **Plan** — still before any code. How you'll structure it. Key decisions. Things you're unsure about. What you'll build first.
- **Progress Notes** — timestamped entries while you work. Drop a note any time you change direction, hit something unexpected, make a trade-off, realise you were wrong, or finish a chunk and start the next. One or two sentences each is fine. Think of it as: your pair partner just asked "what are you doing?" — answer that.
- **Retrospective** — after you're done. Honest. What's the weakest part of your solution? Where would this break in production? What would you do differently? What surprised you?

**The retrospective is not optional.** It's one of the most valuable sections. If you skip it, the submission is incomplete.

---

## Commits

Commit often. **Your commit messages are part of the data.** We read them.

A good commit message captures _what you decided and why_:

> ✅ `"moved refill from a background ticker to lazy arithmetic on each call — the ticker was simpler but burned CPU on idle keys and drifted at low rates, computing on demand fixes both"`

A bad commit message just names the change:

> ❌ `"update queue"`

Good times to commit:

- Finished the initial plan in `THINKING.md`
- First slice compiles / first test passes
- Changed direction on something
- Finished a requirement
- Refactored for clarity
- Added the test runner

---

## Language choice

**Use whatever you're strongest in.** Python is a great default if you don't have a strong preference, it's widely readable and the standard library gets you everything you need for this kind of task. But Go, Rust, TypeScript/Node, Java, Ruby, C#, Kotlin, Elixir, all fine. Pick the language you'd reach for on a normal workday.

Your solution must actually run. Include a short section in your solution's own README explaining how to install deps and run your tests, e.g. `python3 test.py`, `go run ./cmd/tests`, `npm install && npm test`.

---

## No time limit — seriously

There is **no deadline and no time cap**. We pay hourly, so we actively _want_ you to work at your natural pace and be meticulous. If a sub-problem is interesting, dig into it. If you want to try two approaches and compare, do it, just write about it in `THINKING.md`. Granular and thoughtful is better than fast.

Two things follow from this:

1. **Don't rush to "done".** If you'd normally spend 20 minutes considering a trade-off in real work, spend it here.
2. **Don't pad either.** Work at your real pace. The thinking log makes it obvious either way, so just be yourself.

Track your own time loosely (the timestamps in `THINKING.md` are enough) and let us know roughly how long it took when you submit.

---

## How to know you're done

You're done when all of the following are true:

- [ ] You've met the requirements in `TASK.md` (or clearly documented in `THINKING.md` what you didn't get to and why).
- [ ] Your test run works and its output visibly shows the behaviours `TASK.md` asks for.
- [ ] `THINKING.md` is filled in across all four sections, including the **Retrospective**.
- [ ] Someone could clone your fork, follow your setup instructions, and run your tests without asking you questions.

---

## Submitting

1. Push your final commit to your fork.
2. Email **kirill@enki.com** and **catalin@enki.com** with:
   - A link to your fork
   - Roughly how long you spent
   - Anything you'd want us to know (a caveat, a question, a thing you're proud of)
3. We'll review, confirm receipt, and sort out payment.

---

## Questions

If anything in this README or in `TASK.md` is unclear, or if you hit a blocker you can't reasonably work around, email **kirill@enki.com** and **catalin@enki.com**. Don't stall silently, we'd rather clarify than have you guess.

Ambiguity _inside the task spec itself_ is often intentional. How you interpret an underspecified requirement is valuable reasoning data. When in doubt, make a call, write down in `THINKING.md` why you made it, and keep going.

Now head over to [`TASK.md`](./TASK.md).
