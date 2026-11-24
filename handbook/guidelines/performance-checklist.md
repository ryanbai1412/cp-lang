# Performance checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | perf |
| Describes | every rule in `handbook/guidelines/performance.md`, one per id |

Each rule has a stable id (`performance/R9`), one sentence, its reason, an
example, and what enforces it. Tests, gates and the perf quality register
cite the id. Read `performance.md` first for the picture.

## Comparing

**performance/R1.** The C++ reference and the port use the same algorithm,
complexity and data layout.
Reason: otherwise the ratio measures the algorithm, not the language.
Example: a segment-tree port is timed against a segment-tree `orig.cpp`,
not a Fenwick one.
Enforced by: review of each reference; unmatched pairs listed in
`bench/unmatched.txt`.

**performance/R2.** Time the C++ that `cpl build` emits, built with the
judge profile's compiler and flags, on inputs at the stated maximum plus a
degenerate case when the algorithm has one.
Reason: the judge runs that binary on those inputs.
Example: `gen.py` emits n = 2·10^5 and an all-equal array for a sort-based
port.
Enforced by: `bench/v03_perf_bench.py`.

**performance/R3.** A timed run counts only if the port and the reference
print the same output on it.
Reason: a fast wrong answer is not a result.
Example: the bench line for a pair whose outputs differ is an error, not a
ratio.
Enforced by: `bench/v03_perf_bench.py`.

**performance/R4.** Every pair is built and timed, or listed with its
reason.
Reason: silently dropped rows hide the worst cases.
Example: a port that fails `cpl build` is a bug or a row in
`bench/unmatched.txt`.
Enforced by: `bench/v03_perf_bench.py` build/run/skip counts;
`bench/sizes.py --check` fails on a bench program that stops building
unless `bench/unbuilt.txt` lists it with why.

## Bars

**performance/R5.** Idiomatic runtime over the perf set is at most 1.1x
matched C++ at p50, 1.2x at p95 and 1.5x at max; idiomatic peak RSS is at
most 1.5x at p95 and 2x at max.
Reason: the natural program must be fast and small enough to submit.
Example: the `p50`, `p95` and `max` lines and the `peak RSS` line of
`bench/v03_perf_bench.py`.
Enforced by: `bench/v03_perf_bench.py`, numbers in
`plan/domains/perf/metrics.toml` (measured, not gated).

**performance/R6.** Each program over 1.1x keeps its idiomatic port and
gets a tuned `perf.cp` beside it that reaches 1.1x on both runtime and
peak RSS using only the language
(widths, ranges, `unchecked:`, layout, reserving), or a logged gap with an
owner.
Reason: a contestant must never need C++ for speed.
Example: `corpus/problems/cf2084h/perf.cp` uses 32-bit range types and
reserves its lists, 1.23x down to 0.95x.
Enforced by: `bench/v03_perf_bench.py` (both `tuned max` figures), gaps in
`plan/domains/perf/PLAN.md`.

**performance/R7.** Each zero-cost pair runs within 1.05x of its
hand-written C++ and of its direct `.cp` form.
Reason: domains, optionals, iterators and operation values must be free.
Example: `bench/zero-cost/optionals/`.
Enforced by: `cpl bench --zero-cost bench/zero-cost`.

**performance/R8.** Emitted C++ stays under 32 KB and never exceeds 64 KB.
Reason: 64 KB is the judges' hard limit; 32 KB leaves room for the
contestant's own code.
Example: a program using three NTT primes must share one NTT body.
Enforced by: `bench/sizes.py --check`, `crates/cpl-cli/tests/budget.rs`.

## Behaviour

**performance/R9.** An optimization never changes output, output order,
the number of times a call with effects runs, whether or where a program
traps, or how often a lazy view is evaluated.
Reason: a faster program that prints something else is a miscompile.
Example: `let m = xs.map(f)` read twice calls an effectful `f` twice per
element.
Enforced by: `crates/cpl-cli/tests/differential.rs`, `cpl-conform diff`,
`crates/cpl-cli/tests/effect_fuzz.rs` (generated programs).

**performance/R10.** Each rewrite that can change evaluation is gated on a
proven fact: `cpl_ir::effects` says the call is pure, or the Uses table says
the difference can't be observed.
Reason: "probably pure" is how R9 gets broken.
Example: materializing a mapped `let` requires `effects(f).pure()`.
Enforced by: `crates/cpl-codegen/src/uses/` and `cpl_ir::effects` as the
only gates; `codegen_effect_guesses` in `scripts/debt-ratchet.py` (at 0)
rejects an effects or purity helper elsewhere in codegen; review.

**performance/R11.** The program that motivated an optimization or a fix
lands as a parity case, with an effect counter when evaluation count
matters.
Reason: the next change to the same code must hit it.
Example: `crates/cpl-cli/tests/parity/view_receiver_runs_once.cp` counts
calls to the head of a view chain.
Enforced by: review.

## Codegen

**performance/R12.** A heap value that is only read lowers to a `const&`;
it is copied only when the program needs an owned value, and moved when the
source is overwritten before it is read again.
Reason: a hidden copy turns O(n) into O(n²) with no sign in the source.
Example: `let row = rows[i]` reads through a reference;
`var cols = m.get(r) ?? []` then `cols.push(x)` copies.
Enforced by: `crates/cpl-cli/tests/read_only_copies.rs` (`deep_copies`),
`crates/cpl-codegen/src/copy_tests.rs`.

**performance/R13.** An operation value or lambda whose callee is known
statically is called directly, never through `std::function`.
Reason: an indirect call per element stops inlining and costs more than
most inner loops.
Example: `SegTree.new(n, 0, max)` calls `max` directly.
Enforced by: `crates/cpl-cli/tests/read_only_copies.rs` (`std_functions`),
`crates/cpl-codegen/src/devirt_tests.rs`.

**performance/R14.** The copy and `std::function` counts and the size
limits only go down; a change that lowers one lowers its bound in the same
commit.
Reason: a raised bound hides the regression it was meant to catch.
Example: a change that adds a copy is fixed, not accommodated.
Enforced by: `crates/cpl-cli/tests/read_only_copies.rs`,
`scripts/debt-ratchet.py`.

**performance/R15.** Codegen picks a representation or lowering on its own
only when it is never worse on any axis, and never picks a different
algorithm than the program names.
Reason: the contestant must be able to predict the cost of what they
wrote.
Example: a map whose order is never observed may become a hash table; a
sort never becomes a counting sort.
Enforced by: review.

**performance/R16.** Each optimization is one general rule in one codegen
module, with a shape test, never keyed to a library or function name.
Reason: a name-keyed rule breaks when the library changes and never helps
user libraries.
Example: closure borrowing follows any hand-off chain, not
`std.seq.bsearch`.
Enforced by: shape tests in `crates/cpl-codegen/src/`;
`scripts/debt-ratchet.py` (`backend_std_names`).

## Visibility and reproducibility

**performance/R17.** Each implicit codegen choice (a copy, an indirect
call, fusion, sinking, a dropped check, a chosen width) appears in
`cpl explain`, text and `--json`, with its source position and reason.
Reason: costs are visible.
Example: `cpl explain` lists `3:9 cols copied: pushed to later`.
Enforced by: `cpl explain` tests in `crates/cpl-cli` (copies and closures
only).

**performance/R18.** Building the same program twice gives the same C++,
byte for byte.
Reason: a submission must be reproducible and diffable.
Example: no hash-ordered walk decides emission order.
Enforced by: `crates/cpl-cli/tests/deterministic_cpp.rs`.

**performance/R19.** Each published number is recorded with its command,
commit, judge profile, input generator and seed, timed one job at a time on
a quiet machine.
Reason: a number nobody can reproduce can't be checked or improved.
Example: an entry in `plan/domains/perf/worklog.md` with the command line.
Enforced by: review.

**performance/R20.** Benchmarking never submits to a real judge.
Reason: judges are shared resources, and the local profile mimics them.
Example: `cpl bench` and `bench/` run locally only.
Enforced by: review.
