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

## General fixes owed

Local fixes that landed, and the mechanism that would remove the whole class.

| Instance | General fix | Owner |
|---|---|---|
| `(0..b).start` emits `0`, not `Range::make(0, b).lo` (`expr/builtin.rs`; noi17p2 34530 -> 32521 B) | lowering stops building `Range::make(0, 0 + len).lo + i + 0` around view indexes; failing that, one IR simplifier pass (pure projections of literals, `x + 0`) in `lib.rs::prepared` beside `prune` | compiler (lowering), perf (pass) |
| `if` on a known `same(op, op)` pruned in `prune.rs`, but only when `on:` is passed | `same` folds for a default-valued op parameter too (request in compiler's worklog) | compiler |
| Bitset fusion keyed on the field list (`is_bitset_ty`, `scope::rows`), stale twice | recognise std types by declaration, one predicate shared by codegen | perf |
| unused runtime fns found by name in the text (`runtime::unused_lines`), now skipping comments and members | runtime pieces declare the fns they define, and reachability follows the emitted calls, as it does for program fns | perf |
| cf2147c timed on a derived input that breaks the statement (`n` = 1, 200000-letter `s`) | `cpl bench` derives only inputs the port's declared types allow, and ports tie a string's length to its count (`s: Str` of length `n`) | tools (`cpl bench`), idioms (port) |
| `s.runs(link: f).all(...)` builds every run and calls `link` through `std::function` | `.all`/`.any` over a `yields` fn feed the body as `cpl_each_` loops do; a `?Fn` argument given a lambda is devirtualised like a `Fn` one | compiler (lowering of `.all`), perf (devirt) |
| two forwarding closures for one fn named twice as a value (cf2147f `op: merge`), merged in `devirt::same_closures` | lowering makes one closure per (fn, kept params) and reuses it at every place the fn is named (`lower/mono.rs::fn_as_closure`) | compiler |
| cf2147f needs a hand-written cover tree to reach orig.cpp: `SegTree` with an `Action` always pushes its tags down and keeps `t`, `lz`, `has_lz`, `span` in four arrays | a commutative `Action` whose effect a node can recompute from its own tag (cover counts) skips the push; one node record instead of parallel arrays | stdlib (`std.ds` SegTree) |
| cf2124h tuned by writing constant bounds (`0..=30_002`) where port.cp has `0..=2 * n` | width inference bounds a range through names and `*` (`lower/range.rs::expr_limits`), so a bound written from the statement's variables narrows as a constant does | compiler |
| reduction sinking re-derives which fns are pure (`sink::pure_fns`), as fusion re-derives trap-freedom (`fuse::trap_free_with`) | one effects summary per fn in the IR (traps, reads, writes, recursion), computed once and read by every codegen pass | compiler |
| cf2237g tuned by hand into one queue, since `[y for x in layer for y in grow(x)]` builds a list per call of `grow` | the reverse of reduction sinking: a fn that builds a fresh list and returns it, iterated once by its caller, gets a variant that pushes each element to the caller's loop body | perf |
| cf2183e has no tuned variant: orig.cpp's loop order and 0-based tables leave it at 0.53-0.56 s against orig.cpp's 0.39 s; hand-writing `Mod.add` in 32 bits (`v = a - (P - b); v < 0 ? v + P : v`) takes it to 0.48 s, but std's `ll(a) + ll(b) >= P` form is all codegen emits, and the 32-bit form written in std gets a `check_range`, since the range of `a - (P - b)` under `a >= P - b` is not proven | codegen evaluates an expression in the narrowest width its operands' ranges prove (here `int`), and range facts follow an `if`'s condition into its branches | perf (width), compiler (branch facts) |
| cf2124g tuned by rewriting each prefix-minimum block in one `mins` row and one suffix-sum row: port.cp's per-block `scan`, `suffix_fold` and two `[Blk] t => xs[t.l]` copies cost about six mallocs a block (malloc and free 25% of the profile on a constant input) | `scope::reused` also keeps a `let` list built inside a non-reentrant loop (a comprehension, or a std fn's fresh result written into the caller's buffer), and a re-indexing comprehension such as `[D] t => xs[t.l]` lowers to a view, not a copy | perf |
| tuned variants found under three names (`perf.cp`, `<stem>.perf.cp`, dry-run `<stem>_w.cp`) | one name, `perf.cp` / `<stem>.perf.cp`; rename the dry-run files | perf |

## Next

Deadline about 09:30 PST 2026-11-23; worklog checkpoint every 2 hours.

1. Tail until about 03:00: `p06_labyrinth` (done: 77e70afe0, 69.3 -> 40.4 ms), `cf1146h`, `p10_matching`, `cf2178h`.
2. `ucup5s05f` and `noi17p2` under 32 KB (about 1 h).
3. Full perf set and sizes re-run after each fix (`bench/compiles.py` is 1 failure, routed).
