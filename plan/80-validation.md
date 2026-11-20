# 8. Validation: tests, corpus, benchmarks

Status: proposed. The user: "super important to have good validation as
well and mostly ensure performance". This doc is about proving the language and its output
are correct, fast and pleasant on real code.

## 8.1 The corpus is the yardstick

`corpus/` holds about 3,800 of the user's accepted submissions from DMOJ,
with metadata in `corpus/submissions/info.json`: problem code, verdict,
time and memory. Most are C++; some are Java, which are useful as problem
references but not as performance baselines.

About 100-500 problems are ported **by rethinking, not translating**
(agent-context.md): the port is what the user would write from scratch in
the language, with the same algorithm and complexity.

### Choosing which to port

Picked to cover the language, not at random:

- every major area: graphs, trees, DP, segment trees and other range
  structures, strings, math, geometry, greedy, interactive, multitest;
- the most performance-sensitive: submissions whose time is closest to the
  limit, and the most memory-heavy;
- the most index-heavy, where domains should help most;
- the largest, which stress the 64 KB budget and program structure;
- recent first (2023-2024), since they reflect the user's current style.

`research/mine.py` and friends find candidates by these criteria.

### The test oracle

The corpus has verdicts but no test data, and DMOJ doesn't publish tests.
So each port is checked by **differential testing** against the original
accepted C++, which is the reference:

1. Write an input generator for the problem, preferably as an input type
   (key features, 1.7) so it comes for free; otherwise a small `gen` fn.
2. Run original and port on thousands of random inputs, small sizes first
   (where bugs show up) and then max sizes.
3. Any difference is a bug in the port or the compiler, shrunk to a
   minimal input by `cp stress`.
4. For problems with several correct answers, compare with a checker
   instead of exact output.
5. Where official data exists (CSES publishes tests; some contests publish
   data packages), it is used too.

Each port stores: the source, the original, the generator, a set of
saved tests (samples, shrunk failures found during porting, max-size
cases), expected outputs, and performance baselines.

### The four bars

Each port must meet:

- **Correctness:** identical output to the original on every saved test and
  on a fresh random run of at least 1,000 inputs.
- **Speed:** within 1.1x of the original's time on max tests, and the
  target is equal; anything over 1.03x gets an explanation (G1).
- **Memory:** at most the original's peak memory (G2).
- **Size and fluency:** bundled output under 32 KB; source shorter than the
  original **not counting code moved into the stdlib** (the user's rule);
  and a review finding no awkward spots (G4, G6).

`cp corpus report` tracks every port against the bars on every compiler
change, and writes a report with the slowest, largest and most awkward
ports at the top.

## 8.2 Proving abstractions are zero cost

Measuring time is necessary but not sufficient: a 5% loss can hide in
noise. So zero cost is also checked statically:

- **Sizes.** A test file asserts `sizeof` for core types in the emitted
  C++: a domain position is 4 bytes, `?V` is 4, `?Int` is 8, `?0..=1e9`
  is 4, a range is 8, a handle is 4, `[V](u: V, v: V, w: 1..=1e9)` is 12
  bytes per element. Any change fails CI.
- **Per-port memory audit.** For each port, bytes per element of every
  array and container, compared with the original's C++ types. An 8-byte
  field where the original stored 4 is flagged.
- **Assembly comparison.** For the hot loops of benchmark kernels (BFS
  over a lambda view, prefix sums on bounds, generator loops, segment tree
  fold with an operation value, `runs`, `sliding`), the emitted C++ and its
  `-O2` assembly are compared with hand-written C++. The check is "same
  loop structure, no extra calls, no extra branches", reviewed by hand once
  and then diffed automatically.
- **No hidden runtime.** The emitted code for a port contains no virtual
  calls, no `std::function`, no heap allocation that the source doesn't
  show, and no bounds checks inside loops unless `cp build --checks` lists
  them.

## 8.3 The bug corpus

A set of real WAs from the user's history and known pitfalls:

- off-by-one errors and prefix-sum shifts;
- the wrong array or the wrong index;
- overflow;
- negative `%`;
- stale multitest state;
- missing cases (an empty input, `n = 1`);
- swapped grid indices;
- sentinel collisions (`-1` that is also a valid value).

Each is written in the language as directly as possible. It must fail to
compile, fail a check in a debug run, or be impossible to write. The
target is at least 90% (G3). Past WA submissions in `info.json` with a
later AC on the same problem are the best source: the diff is the bug.

## 8.4 Error-message review

There is a test file for each common mistake, and the rendered message is
checked into the repository as a golden file. Every message is reviewed
against "would I know what to type next?" (G5). The same files generate
the error-index pages (doc 9), so the gallery and the tests can't drift.

## 8.5 Performance benchmarks

Microbenchmarks in `research/bench/` cover generators, index checks, BFS,
segment trees, DP, FFT, hashing and I/O. Each is compared with hand-written
C++ using g++ at `-O2` (and the judge profile's flags), on a pinned core,
taking the median of at least 7 runs.

- More than 3% slower than the hand-written version is a bug to explain.
- Any regression fails CI.

## 8.6 Compiler tests

The compiler itself needs:

- snapshot tests for each stage;
- run tests on both backends;
- differential testing: interpreter vs C++, optimized vs unoptimized,
  Python prototype vs Rust, ports vs originals;
- a random program generator (fuzzer) whose programs must give the same
  output in the interpreter and the C++ backend;
- budgets for compile time and output size.

## 8.7 Friction

Every awkward spot found while porting is written down in
`research/friction.md` with the submission that showed it, and becomes a
design change, a stdlib addition, or a recorded decision not to change.
Design changes are logged in `decisions.md` with the motivating port.
Friction is the signal the user cares most about: "any awkwardness and
friction is a sign that we should change the design".

## 8.8 Using it for real

The last validation is contests: virtual contests first, then rated ones,
with the friction log kept throughout.
