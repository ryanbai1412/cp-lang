**CP Lang** is a programming language for competitive programming that compiles to fast, human-readable **C++17** you can submit directly to Codeforces, AtCoder, DMOJ, Kattis, and ICPC judges.

Keep the algorithms and data structures you already know. Replace index arithmetic, `1e18` sentinels, `((a % m) + m) % m`, and repetitive container boilerplate with typed domains, Tropical Optionals, `@memo` DP, and an algebraic standard library — all compiling down to flat, single-file C++17.

- **Learn from scratch:** work through the [interactive course](course.html) — {{chapter_count}} progressive chapters with {{exercise_count}} checked exercises and live in-browser WebAssembly sandboxes.
- **Try it in your browser:** open the [playground](playground.html) to run CP Lang, inspect the generated C++17, or press **Trace** to step through variables and `@brute(main)` comparisons visually in the [Trace debugger](trace.html).
- **Keep a contest reference open:** print or bookmark the [one-page cheat sheet](cheat-sheet.html), browse the [standard & local library reference](stdlib.html), or jump straight in from the [C++17 migration guide](cpp-guide.html).

## Why CP Lang?

Most Wrong Answers in contests do not come from the algorithm idea — they come from off-by-one boundaries, mixing up vertex and edge indices, forgetting to clear a DP table between test cases, overflowing `INF + w`, or erasing every copy from a `std::multiset`. CP Lang catches those bugs at compile time or eliminates the boilerplate that causes them:

- **Typed domains & boundaries:** declare `domain V = 1..=n` once, read `let a: [V]Int = read()`, and write prefix sums with `pre[v.r] = pre[v.l] + a[v]` — zero off-by-one shifts and zero runtime bounds overhead.
- **Tropical Optionals (`?T`):** `?Int` uses `LLONG_MIN` as a zero-byte niche tag, `+`/`-` absorb `none`, and `best min= dist[u] + w` skips unreachable states automatically — no `1e18` sentinels.
- **Symmetric `@!multitest`:** put sieves and factorials above `@!multitest`, put per-case logic below it, use bare `return` to end the current case, and let `@memo` tables reset automatically between cases.
- **Built-in verification & Trace:** put `@brute(main)`, `@checker`, or `@interactor` right next to your solution in the same file, run `cpl stress` to shrink failing cases automatically, and strip all verification code cleanly on `cpl build`.

## Real contest solutions: C++17 vs. CP Lang

Below are five real accepted competitive programming solutions (from COCI, NOIP, NOI, and Cheerio), comparing the original C++17 submission side by side with its idiomatic CP Lang 0.2.2 rewrite.

Line counts are shown two ways: **raw** (the entire accepted C++17 file) and **stripped** (removing personal `#include` and debug-macro boilerplate so you see the pure algorithm savings). Click **Show all lines** on any C++17 panel to inspect the complete solution body.

```showcase
```

Across hundreds of ported competitive programming problems, CP Lang solutions average roughly two-thirds fewer total lines than raw C++17 — and nearly half as many lines even after stripping C++ template and library boilerplate.

## Complete documentation map

Everything you need to learn CP Lang fluently and look up any detail during a contest is organized into two tracks:

- **Interactive course ({{chapter_count}} chapters):** starts with [Your first program](course-first-program.html) and progresses through [domains & boundaries](course-domains.html), [types, enums & strings](course-types-strings.html), [collections & binary search](course-collections.html), [Index & slices](course-index-slices.html), [ownership & `lends`](course-borrowing.html), [DP, optionals & bitmasks](course-dp.html), [Fenwick & segment trees](course-algebra.html), [graphs, trees, math & geometry](course-graphs-math.html), and [stress testing & Trace](course-verification.html).
- **Language & library reference:** [Getting started & CLI](getting-started.html), [One-page cheat sheet](cheat-sheet.html), [Language tour](tour.html), [Types, numbers & control flow](types-and-control.html), [I/O, formatting & contest modes](io-and-formatting.html), [Index spaces & slices](indices.html), [Ownership & borrowing](borrowing.html), [Strings & ASCII text](strings.html), [Memoization, stress testing & Trace](testing.html), [Standard & local libraries](stdlib.html), and [From C++17 & performance](cpp-guide.html).

> **Version 0.2.2 status.** CP Lang 0.2.2 is in active development. The browser WebAssembly [playground](playground.html), interactive course exercises, [Trace debugger](trace.html), and local `cpl` CLI run the core language and emit standalone C++17; see [Getting started & CLI](getting-started.html#what-works-today) for what runs today versus upcoming CLI features.
