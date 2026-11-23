# perf plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Workstreams, in order (Milestone 1 first)

Milestone 1 steps 6 and 7 (Submit, Compete) set the order; see the README's Milestone 1 section. Landed in phase 2: the in-flight branches, the perf and size harnesses, zero-cost builds, compaction (decision 271), the no-trap judge build (decision 299).

1. **Tail over 1.5x, then tuned to 1.1x** (M1 criterion 3). Idiomatic max < 1.5x and p95 < 1.2x; tuned max <= 1.1x, with a tuned in-language variant next to each outlier over 1.1x and a logged language gap for any that cannot get there. `p06_labyrinth` 3.90x, `cf1146h` 3.47x and RSS 10.5x, `wtf26b`, `p10_matching` 2.23x, `cf2178h` 1.87x, `ranges/mo` 1.47x, `flow/costflow` 1.26x. Profile each with `perf`, find the cause (codegen, stdlib body, or port algorithm), fix it generally or route it, re-run the set.
2. **Fit 32 KB** (criterion 1). `ucup5s05f/port.cp` and `noi17p2/spec.cp`; `sizes.py --check` in the gate (request to infra).
3. **Compiles** (criterion 2). Every built program through g++ as C++17 (`bench/compiles.py`); fix codegen failures, route the rest. No new judge-profile or Windows g++ work in Milestone 1 (Ryan: language quality first); the existing profiles keep working.
4. **Perf set completeness** (M1 of the old list): matched references, the three `bench/` pairs that do not build (`math/ntt`, `seq/scan_in_place`, `seq/slice_nth_element`, all stdlib), corpus blockers routed.
5. **Ratchet** so wins stick: committed baseline, nightly runtime check (request to infra).
6. **Deferred**: new judge-profile work (Codeforces/AtCoder flags, Windows g++), zero-cost pairs past the tail (`iterators`, `optionals` waits on parameter ranges in the IR), SIMD, the `expr.rs`/`emit.rs` splits, harnesses into `cpl bench`.

## Why these milestones

- M1 first because nothing perf-related is in the gate: every number in this folder can regress silently today, and the last full report (`research/perf/corpus-p95-report.md`) is a 0.2 run. Without a ratchet, M2/M3 wins don't stick.
- M2 before M3 because zero-cost pairs isolate one abstraction each; the tail programs are mostly the same causes (indirect calls, checks, allocation) at scale.
- M3 is the AGENTS.md goal 3 test, on programs that matter for v0.3.

## Alternatives considered

| Choice | Picked | Rejected, and why |
|---|---|---|
| Runtime gate | nightly ratchet with 10% slack | in the fast gate: too noisy on 2 cores, adds minutes |
| Perf set | v0.3 programs with matched `_ref.cpp`/`orig.cpp`, >= 10 ms | 0.2 corpus sweep: being deleted; tiny inputs measure process start |
| Harness | land Python `v03_perf_bench.py` now, fold into `cpl bench` later | rewrite in Rust first: delays M1 by a day for no new number |
| Bounds checks | prove from range facts in codegen now, move to `cpl-ir` facts with compiler (B3 step 6) | wait for B3: blocks the two worst offenders on another domain |
| Tail bar | idiomatic max <= 1.5x, and tuned max <= 1.1x (goal 3: every port reaches 1.1x by tuning in the language) | p95 <= 1.5x: hides the worst programs, which are what a contestant hits; idiomatic bar alone: hides whether tuning can close the gap |

## Risks

- Optimisations that change output or traps. Guard: differential tests with sanitizers on every program, shape tests, majorize-only rule.
- Pattern-matching lowered IR (`map_remove_do`) breaks silently when lowering changes. Guard: contract tests in `cpl-ir` (request to compiler) or explicit IR nodes.
- Noise faking wins: report medians of >= 5 runs, cite the command, quote only programs >= 10 ms (or >= 100 ms for headline claims).
- Readability loss from minifying and fusion (Q3).
- Overlap with compiler on facts (`ranges.rs` vs `prover`): agree the move in `plan/domains/compiler/worklog.md` before building more on `ranges.rs`.

## Next

Deadline about 09:30 PST 2026-11-23; worklog checkpoint every 2 hours.

1. Tail until about 03:00: `p06_labyrinth` (done: 77e70afe0, 69.3 -> 40.4 ms), `cf1146h`, `p10_matching`, `cf2178h`.
2. `ucup5s05f` and `noi17p2` under 32 KB (about 1 h).
3. Full perf set and sizes re-run after each fix (`bench/compiles.py` is 1 failure, routed).
