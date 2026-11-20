# 0. Vision and goals

Status: goals decided (the user reviewed `goals.md` and agreed); the
ambition statement and success measures are proposed.

## 0.1 The ambition

We are building **the best language there is for competitive programming**,
and the toolchain around it. It should feel so much better than C++ that a
strong competitor switches for good, without giving up any speed, memory or
judge compatibility.

Concretely, at the end of the plan:

- You write a solution that reads like the editorial. Indexing is typed by
  domains, so the classic off-by-one and wrong-array bugs either don't
  compile or fail loudly in a debug run.
- `cp run d i1` gives you the answer in tens of milliseconds, without
  waiting for g++, and an HTML walkthrough of the whole execution if you ask.
- `cp stress d` finds a failing input, shrinks it to the smallest one while
  you watch, and names the broken part of your solution.
- Before you submit, the compiler tells you which line will be too slow,
  which table won't fit in memory, and which product can overflow.
- `cp build` produces one readable C++17 file, under 64 KB, that runs as
  fast as what you would have written by hand, and every judge accepts it.
- The editor shows types, proven ranges, memory use and live sample values
  as you type.
- A docs site and an online playground with real problems teach the
  language in an afternoon.

The core language stays general-purpose; everything contest-specific lives
in the standard library, written with the same powers a user has.

## 0.2 Goals

Goals are tiered. Each has a test we can run.

**P0: the language fails without these.**

- **G1 – Fast by default.** The obvious code compiles to what a strong
  competitor would write in C++: no hidden copies, allocations, virtual
  calls or boxing, and no bounds checks in hot loops when they can be
  proven. Domains, operation values, generators and optionals cost nothing
  at run time. *Test:* every corpus port runs within 1.1x of the original
  C++, and the benchmark suite has no regressions.
- **G2 – Memory-tight by default.** Values take the fewest bytes their
  types allow: widths from ranges, niche optionals, flat grids, 4-byte
  handles. *Test:* every port uses at most the original's memory, and
  `cp explain` shows bytes per element for every container.
- **G3 – Correct by default.** The easy way is the correct way: typed index
  spaces, overflow caught from ranges, distinct domains, `mut` at call
  sites, exhaustive `match`, optionals instead of sentinels, floor division,
  no uninitialized values or stale per-test state. *Test:* at least 90% of a
  corpus of real past WAs and known pitfalls don't compile, fail a check, or
  can't be written.
- **G4 – Fluent.** You write the algorithm, not the bookkeeping. *Test:*
  ports are shorter than the originals without counting code moved into the
  stdlib, and reviews of the ports find no awkward spots left unfixed.
- **G5 – Great errors and debugging.** Every message is precise and says
  what to do; one root cause gives one error; runtime errors are in source
  terms; debug runs show intermediate values automatically. *Test:* every
  message in a corpus of common mistakes passes the review "would I know
  what to type next?", and `cp locate` names a seeded bug's part and a
  minimal input in under a minute.
- **G6 – Submittable.** One C++17 file that any judge accepts; under 64 KB
  as a hard limit (warning at 48 KB); deterministic. *Test:* every port
  compiles on GCC and Clang at `-std=c++17` and is under 32 KB.
- **G7 – Coherent.** One obvious way to do each thing; features compose
  without special cases. *Test:* each new feature is checked against the
  existing ones, and the spec's list of special cases stays short.

**P1: important.**

- **G8 – General-purpose core, contests in the library.** About 95% of the
  stdlib is ordinary code a user could write with the same performance, on
  top of a small, listed intrinsics layer.
- **G9 – Predictable.** You can tell what code costs by reading it. Nothing
  secret under the hood: an implicit representation choice is allowed only
  if it is never worse on memory or complexity, and every such choice is
  shown by hover and `cp explain`.
- **G10 – Fast tooling.** Front end under 500 ms; debug runs don't wait for
  g++.
- **G11 – Stable source.** Version stamps now; a compatibility promise once
  stable.

**P2: nice to have.** Clean input from types (which the input-types
proposal would promote to P1), editor support beyond the basics, and
learnability for C++ competitors reading a solution cold.

## 0.3 Non-goals

- Production or systems programming, and I/O beyond stdin and stdout.
- Running submissions on anything but compiled C++. (The debug interpreter
  is a local tool only.)
- Dynamic typing, reflection, inheritance, pointers.
- Copying C++ semantics where C++ is a known trap (division, overflow,
  uninitialized memory).
- Fewest keystrokes at the cost of clarity.

## 0.4 When goals conflict

**G3 correct > G1/G2 fast and tight > G4 fluent > G7 coherent > the rest.**
The one exception: a correctness feature that costs measurable time in hot
code must have a zero-cost form, such as being proven at compile time or
checked only in local builds.

## 0.5 How we'll know it worked

- The user solves real contests in it and doesn't want to go back.
- Every corpus port meets G1, G2, G4 and G6.
- The bug corpus meets G3.
- Compile, debug-run and stress-test latency meet G10.
