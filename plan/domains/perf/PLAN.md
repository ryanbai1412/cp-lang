# perf plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Workstreams, in order

1. **Land what is in flight** (phase 2, first hour).
   - `devin/wip-codegen-perf` (4 commits on `0fc84cf7`): stream/iterator loop fusion (`try_fuse_producer`, `FusedPush`), power-of-two Euclidean `/` `%` as shift/mask, `bench/v03_perf_bench.py`, v0.3 fixes to two zero-cost sources. Rebase, squash the two fixups, move fusion into `crates/cpl-codegen/src/fuse.rs` (stmt.rs would reach 2108 lines, over the 2000 cap), gate, land.
   - `devin/wip-cpp-size-main` (1 commit): `scripts/sizes.py` + gate stage, `src/compact.rs`, fit order. Rebase and land. It touches `scripts/check.sh`, `crates/cpl-cli`, `crates/cpl-judge`, `crates/cpl-wasm` (infra and tools paths): note it in their worklogs.
   - `devin/wip-cpp-size` and `devin/wip-cpp-size-report` are earlier versions of the same change (same files, older base); `git cherry` shows them unmerged by patch id only. Delete after `wip-cpp-size-main` lands, noting it in the worklog.
2. **Measure and ratchet** (M1). One perf-set runner, JSON out, a committed baseline in `bench/`, and a ratchet: a program > 10% slower than baseline (median of 5) fails. Sizes in the fast gate (deterministic, cheap); runtime in a nightly/`check-full` stage, because VM timings move 5-30% run to run (zero-cost `domains` read 1.37x at 5 reps and 1.02x at 11). Hand the gate wiring to infra as a request.
3. **Zero-cost** (M2), each a general codegen rule with a shape test in `src/tests.rs`:
   - local fn bindings in `devirt.rs` (`Home::Local`);
   - bounds checks from range facts: `proven()` consults `ranges.rs`/loop bounds (`for i in 0..n` over a list of length `n` never written to a different length);
   - `optionals` once Q2 is answered.
4. **Matched-C++ tail** (M3): `ranges/mo` (2.43x: 64-bit storage where C++ has `int`, unreserved `push_back`, dynamic-divisor `%`), `cf2178h`, `math/comb`, `z_prefix`, `p14_elevator`, `sorted_set`. Profile each with `cpl profile --native` and `perf`, fix the cause generally, re-run the set.
5. **Debt alongside**: split `expr.rs` (2900) and `emit.rs` (2233) under the cap by concern (indexing, calls, maps; includes/types/fns); fix `bench/rust_latency_audit.py`; fold the Python harnesses into `cpl bench` once the suite list is stable.

## Why these milestones

- M1 first because nothing perf-related is in the gate: every number in this folder can regress silently today, and the last full report (`bench/corpus-p95-report.md`) is a 0.2 run. Without a ratchet, M2/M3 wins don't stick.
- M2 before M3 because zero-cost pairs isolate one abstraction each; the tail programs are mostly the same causes (indirect calls, checks, allocation) at scale.
- M3 is the AGENTS.md goal 3 test, on programs that matter for v0.3.

## Alternatives considered

| Choice | Picked | Rejected, and why |
|---|---|---|
| Runtime gate | nightly ratchet with 10% slack | in the fast gate: too noisy on 2 cores, adds minutes |
| Perf set | v0.3 programs with matched `_ref.cpp`/`orig.cpp`, >= 10 ms | 0.2 corpus sweep: being deleted; tiny inputs measure process start |
| Harness | land Python `v03_perf_bench.py` now, fold into `cpl bench` later | rewrite in Rust first: delays M1 by a day for no new number |
| Bounds checks | prove from range facts in codegen now, move to `cpl-ir` facts with compiler (B3 step 6) | wait for B3: blocks the two worst offenders on another domain |
| Tail bar | max <= 1.5x | p95 <= 1.5x: hides the worst programs, which are what a contestant hits |

## Risks

- Optimisations that change output or traps. Guard: differential tests with sanitizers on every program, shape tests, majorize-only rule.
- Pattern-matching lowered IR (`map_remove_do`) breaks silently when lowering changes. Guard: contract tests in `cpl-ir` (request to compiler) or explicit IR nodes.
- Noise faking wins: report medians of >= 5 runs, cite the command, quote only programs >= 10 ms (or >= 100 ms for headline claims).
- Readability loss from minifying and fusion (Q3).
- Overlap with compiler on facts (`ranges.rs` vs `prover`): agree the move in `plan/domains/compiler/worklog.md` before building more on `ranges.rs`.

## First in phase 2

1. Rebase, split `fuse.rs`, land `devin/wip-codegen-perf`; re-run zero-cost (expect 5/7: iterators ~1.05x, segtree_prefix ~1.01x).
2. Rebase and land `devin/wip-cpp-size-main`; delete the two superseded size branches.
3. Commit the perf-set baseline JSON and add the ratchet; request gate wiring from infra.
4. `Home::Local` devirtualization with its shape test; re-measure `p10_matching`.
