# perf

Segment: B. Metric. Owns: `crates/cpl-codegen`, `cpl bench` (`crates/cpl-contest/src/{bench,zero_cost,corpus_bench}.rs`), `bench/`, generated C++ size. Status 2026-11-22: zero-cost bench 3/7 pass on main, two in-flight branches fix most of it; nothing perf-related is in the gate.
Detail: [PLAN.md](PLAN.md) · [ARCHITECTURE.md](ARCHITECTURE.md) · [metrics.toml](metrics.toml) · [worklog.md](worklog.md).

## Mission

The submitted C++ is as fast and as small as the C++ a strong contestant writes by hand, for the same algorithm and layout, so nobody ever drops to C++ for speed. Domains, optionals, iterators and operation values cost nothing; any cost left is visible and tunable in the language.

## Covers / not covered

| Covered here | Not here (owner) |
|---|---|
| C++17 codegen: lowering, devirtualization, bounds-check elimination, fusion, narrowing, niches | IR and lowering, range proofs in `prover` (compiler) |
| `cpl bench` (zero-cost pairs, corpus), `bench/` suites and reports | Gate wiring, CI, cache (infra); we hand them the command |
| Emitted C++ size and fitting under 32 KB | Judge profiles' contents, `cpl-judge` (tools) |
| Runtime vs matched C++, peak RSS, compile time of the C++ | Interpreter speed (compiler), front end speed (compiler; we track it) |
| Perf of stdlib code paths through codegen | Stdlib bodies and their algorithms (stdlib) |
| | Choice of algorithm in ports (idioms) |

## Milestone 1

[Milestone 1](../../product/MILESTONE-1.md), due about 09:30 PST 2026-11-23, needs perf for step 6 (Submit) and step 7 (Compete): what CPL submits is one C++17 file that compiles under a standard g++, stays well under the judges' 64 KB, and is never slower than the C++ the friend would write. Ryan: language quality over judge-specific features, so no new Milestone 1 time goes into judge profiles; the existing profiles and their tests keep working. Checkpoints go in [worklog.md](worklog.md) every 2 hours.

Exit criteria, each one command:
1. **Fits.** Every program that builds is under 32 KB (`python3 bench/sizes.py --judge generic --check`). Today (debug `cpl` at `058ce6f3f`): 753 programs build; 2 are over, `ucup5s05f/port.cp` at 37,078 B and `noi17p2/spec.cp` at 34,484 B.
2. **Compiles.** Every program that builds compiles as C++17 with g++ (`python3 bench/compiles.py`). Today: 753 build, 1 does not compile (`etsr23r4p2/spec.cp`: `Action.new`'s default `compose: (+)` lowers to nothing; routed to compiler).
3. **No performance disadvantage, tail first.** On the matched perf set (`bench/v03_perf_bench.py`): idiomatic max < 1.5x and p95 < 1.2x; tuned bar: max <= 1.1x with a tuned in-language variant kept next to each outlier (widths, `unchecked:`, layout; `perf.cp` beside `port.cp` as `cpl bench --corpus` already reads it, else `<stem>.perf.cp`), and an outlier that cannot get there in-language is a logged language gap. Today (no-trap build, 142 timed): p50 0.778x, p95 1.548x, max 3.90x (`p06_labyrinth`), 9 over 1.5x; tuned max not measured yet (0 tuned variants; the harness prints it once they exist). Since then, 77e70afe0 took `p06_labyrinth` from 69.3 to 40.4 ms (matched C++: 14.3 ms).
4. **No traps by default.** `cpl build` and `cpl submit` emit the no-trap build (decision 299, done); `--checks traps` is the opt-in. Trap cost on 46 `bench/` pairs: p50 1.011x, max 1.27x (`monodeque`).

Deferred: new judge-profile work (Codeforces/AtCoder flags, Windows g++); SIMD and optimisation beyond parity; zero-cost pairs not in the tail (`iterators`, `optionals`); `cpl explain` cost annotations; codegen file splits; Python harnesses into `cpl bench`.

Next 3 (in hours):
1. The tail: idiomatic max over 1.5x first (`p10_matching`, `cf2178h`, the rest of `p06_labyrinth`; `cf1146h` 3.47x -> about 1.04x by 037d7bf31 and c8e47d1b4), then a tuned `perf.cp` at <= 1.1x for each outlier over 1.1x, or a logged language gap. General fixes only; stdlib or port causes go to their owners.
2. Under 32 KB, about 1 h: `ucup5s05f` and `noi17p2`.
3. Re-run the full perf set and sizes after each fix; ratchet the numbers.

## Success criteria for v0.3

1. Zero-cost: every `bench/zero-cost` pair <= 1.05x against both `direct.cp` and `hand.cpp` (7/7).
2. Matched C++ on the perf set (v0.3 dry runs with a matched `_ref.cpp`, `bench/<area>/*.cp`, corpus `spec.cp` ports; both sides >= 10 ms at max-N): p50 < 1.1x, p95 < 1.2x, max < 1.5x, and max < 1.1x after constant-factor tuning (decision 269).
3. Every program in the corpus builds under 32 KB for every judge profile; none over 64 KB.
4. Every port reachable to <= 1.1x by tuning in the language, shown on the 5 worst.
5. All of it measured by one command per metric and ratcheted, so no landing makes a number worse.

## Milestones

- **M1 Measured and ratcheted** — zero-cost 7/7 build on v0.3; `devin/wip-codegen-perf` and `devin/wip-cpp-size-main` landed; `scripts/sizes.py --check` in the gate; one perf-set runner whose JSON feeds `metrics.toml`; a perf ratchet file that a regression > 10% fails. Exit: `cpl bench --zero-cost` exits 0 on build errors = 0; `scripts/check.sh --fast` runs sizes.
- **M2 Abstractions are free** — exit: `cpl bench --zero-cost bench/zero-cost --reps 11` 7/7 pass both bars; shape tests in `src/tests.rs` show zero `std::function` for `let f = mon.op; f(a, b)`, zero `cpl::at` for `for i in 0..n` over `[0; n]`, zero intermediate vectors for `filter.map.sum`.
- **M3 Parity with matched C++** — exit: perf set p50 < 1.1x, p95 < 1.2x and max < 1.5x (5 runs, median), and tuned max <= 1.1x (every outlier over 1.1x has a tuned in-language variant at <= 1.1x, or a logged language gap); `ranges/mo`, `cf2178h`, `p10_matching` each <= 1.5x; front end < 500 ms on the largest v0.3 program.

## Metrics

Today = measured on main `4b8f504fc`, 8-core VM, g++ 11.4 `-O2`, release `cpl`; branch harnesses copied in to run. Commands in [metrics.toml](metrics.toml).

| Metric | Command (short) | Today | Target | M |
|---|---|---|---|---|
| Zero-cost pairs passing | `cpl bench --zero-cost --reps 11` | 3/7 (2 don't build) | 7/7 | M2 |
| Worst zero-cost vs hand.cpp | same | iterators 7.98x | <= 1.05x | M2 |
| Worst zero-cost vs direct.cp | same | iterators 5.00x | <= 1.05x | M2 |
| optionals vs hand.cpp | same | 1.26x | <= 1.05x | M2 |
| Perf set p50 | `bench/v03_perf_bench.py` (branch) | 0.987x (n=48); 1.028x on `bench/` alone | <= 1.1x | M3 |
| Perf set p95 | same | 1.697x (n=137, 2026-11-22 full run) | < 1.2x | M3 |
| Perf set max | same | 2.431x (`ranges/mo`) | <= 1.5x idiomatic; <= 1.1x tuned | M3 |
| Perf set programs > 1.5x idiomatic (tuned bar: none > 1.1x, next rows) | same | 3: `ranges/mo` 2.43x, `cf2178h` 1.76x, `p10_matching` 1.53x | 0 | M3 |
| `runtime_tuned_max`: perf set tuned max (better of idiomatic and its `perf.cp`) | same, `tuned max` line | not measured: 0 tuned variants | <= 1.1x | M3 |
| Outliers over 1.1x tuned | same, `over 1.1x tuned` lines | 40 over 1.1x idiomatic, none tuned yet | 0, or each a logged language gap | M3 |
| Largest emitted C++ | `bench/sizes.py --check` | 21.7 KB (v0.3 dry runs, 35/35 build) | < 32 KB | M1 |
| Programs over 32 KB | same | 0 of 161 v0.3 programs that build; largest `cco21p4` 31.9 KB | 0 | M1 |
| Front end, largest v0.3 program | `time cpl check` | 3 ms | < 500 ms | M3 |
| Codegen shape tests | `cargo nextest run -p cpl-codegen` | 91 | grows | all |
| Codegen files over 2000 lines | `scripts/debt-ratchet.json` | 2 | 0 | M2 |

Not measured: sizes for judge profiles other than `codeforces`. Peak RSS, g++ time on the emitted C++ and the `bench/rust_latency_audit.py` toolchain budgets are in `metrics.toml` (measured after the packet). Caveats: 20/54 `bench/` and 187/307 corpus pairs did not build or run, so the set is partial; several dry-run "matched" references are slower algorithms (`p20_point` 0.055x), which drags p50 down. Fixing the set is M1.

## Judgment rubric

Scored 1-5 with a reason by a cold agent on fixed samples (`p10_matching`, `ranges/mo`, `bench/zero-cost/iterators`).

1. Would a strong C++ contestant submit this kernel as is: no `std::function`, no check they can see is redundant, no copy they would not make?
2. When a program is slow, does `cpl explain` / `cpl profile` say why, in source terms?
3. Is every optimisation one that majorizes and is shown, never a secret choice of algorithm?
4. Is each optimisation a general rule in one module with a shape test, never keyed to a library name?
5. Can a contestant read the emitted C++ back to the source?

Probes (cold agent, pass/fail + time + files touched):

- **Diagnose:** given `p10_matching` at 1.53x and the docs, name the cause (indirect call through `let f = mon.op`) in <= 15 min. Tests rubric 2.
- **Tune:** take the worst perf-set program to <= 1.1x by editing only the `.cp`. Tests criterion 4.
- **Extend:** add `reserve` on known-length producer loops following ARCHITECTURE.md; <= 3 files, a shape test, no special case. Tests rubric 4.

## Improve (highest impact first)

1. Land `devin/wip-codegen-perf`: loop fusion (iterators 7.98x -> 1.05x per the branch run) and power-of-two `/` `%` (segtree_prefix builds again; 1.01x vs direct per the branch run), fusion split into `src/fuse.rs`.
2. Land `devin/wip-cpp-size-main` (size report, 32 KB gate, `src/compact.rs`); drop superseded `wip-cpp-size`, `wip-cpp-size-report`.
3. Put sizes and a perf ratchet in the gate; one runner for all suites.
4. Bounds checks from range facts (`proven()` is syntactic): `z_prefix` 1.37x, `cf2178h` 1.76x.
5. Devirtualize local fn bindings: `p10_matching` 1.53x.
6. `ranges/mo` 2.43x: narrow element storage, `reserve`, dynamic-divisor `/` `%`.
7. Split `expr.rs` / `emit.rs` under 2000 lines; move `ranges.rs` / `niche.rs` facts into `cpl-ir` with compiler.

## Milestone 2

[Milestone 2](../../product/MILESTONE-2.md), the stable release. The most ambitious complete version of perf: what CPL emits is never the reason to write C++ instead. Idiomatic code runs at the speed of the C++ a strong contestant writes, a tuned program matches the best hand-written C++, every solution is small and fast to compile, and `cpl explain` shows each choice the compiler made.

Exit criteria:
- Perf set, every corpus port with a matched reference: idiomatic p50 <= 1.1x, p95 <= 1.2x, max <= 1.5x, and tuned max <= 1.1x (`runtime_tuned_max`); every gap left is a logged language gap with an owner.
- Zero-cost pairs 7/7 within 1.05x of hand-written C++: domains, optionals, iterators, operation values (`cpl bench --zero-cost`).
- Every corpus solution under 32 KB on every judge profile; g++ -O2 time p50 under 1 s.
- Peak RSS p50 <= 1.05x and max <= 1.5x matched C++.
- `cpl explain` names every implicit codegen choice (inline rows, fusion, reduction sinking, check elision); SIMD-friendly loops where g++ can vectorize the matched C++.

Next steps: tuned `perf.cp` for each M1 outlier; `cpl explain` for sinking and fusion; the iterators and optionals zero-cost gaps.

## Ryan's role

Signs off the targets and the perf set once (Q1). Hears only about trade-offs: readable C++ vs size (Q3), language semantics that cost time (Q2). Everything else is ordinary decisions in `decisions.md`.

## Decisions needed from Ryan

None open. Judge-build traps: Ryan decided no traps by default (decision 299). Decided by their recommendations (decisions.md 269-271): the perf set and its bars, floored `%` as semantics (optionals declares its ranges), minify before dropping the source header.
