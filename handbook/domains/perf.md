# Perf domain charter

| | |
|---|---|
| Owner | Perf |
| Status | Active |
| Describes | `crates/cpl-codegen`, `bench/`, `cpl bench`, `cpl explain` (codegen sections) |

## Mission

The C++ that `cpl build` submits is as fast and as small as the C++ a strong
contestant writes by hand for the same algorithm and layout, so nobody drops
to C++ for speed. Domains, optionals, iterators and operation values cost
nothing. Any cost that remains is visible in `cpl explain` and can be tuned
in the language. No optimization changes what a program prints, how often a
call with effects runs, or where it traps.

Example: `let best = xs.rev.find(x => x % 7 == 0)` compiles to one downward
loop that stops at the first hit. It builds no list, uses no
`std::function`, and runs as fast as the hand-written loop. If `xs` is a
`List<List<Int>>` row, it is read through a `const&` and never copied.

## Scope

Owns:
- `crates/cpl-codegen`: C++17 emission, layout and widths, niches,
  devirtualization, bounds-check elimination, fusion, sinking, lending and
  moves (the Uses table), and fitting a judge's source limit.
- `cpl bench` behaviour (`crates/cpl-contest/src/{bench,zero_cost,corpus_bench}.rs`,
  files kept in tools' crate) and every harness under `bench/`.
- The perf set: matched C++ references (`orig.cpp`, `_ref.cpp`), tuned
  variants (`perf.cp` beside `port.cp`) and their accounting
  (`bench/unmatched.txt`, `bench/oversize.txt`).
- The codegen sections of `cpl explain` (copies, closures, dropped fields).
- Ratchets on the generated C++: `deep_copies` and `std_functions` in
  `crates/cpl-cli/tests/read_only_copies.rs`.

Doesn't own:
- What a program means, lowering and `cpl_ir::effects`: compiler. Perf
  fixes IR lowering when the cause is a cost (a value lowered twice), and
  tells compiler.
- Interpreter speed and the front end's speed: compiler (perf tracks the
  front-end number).
- Judge profiles' contents and `cpl-judge`: tools.
- Std bodies and their algorithms: stdlib. Which algorithm a port uses:
  idioms.
- Gate wiring and CI: infra. Perf supplies the commands.

## Interfaces

Provides:

| Interface | API | Main consumers |
|---|---|---|
| Emission | `cpl_codegen::{emit, emit_for, emit_within_for, emit_profiled, sizes, Target, Part}` | CLI (`cpl build`, `cpl submit`), conform, stress, wasm playground, profile |
| Explanations | `explain_closures`, `explain_dropped_fields`, the copies list | `cpl explain` text and `--json` |
| Bench results | `research/perf/corpus-p95-results.json` (schema checked in `corpus_bench::schema_tests`) | site |
| Perf numbers | `plan/domains/perf/metrics.toml` | coordinator, tech-debt burn-down |

Needs:
- compiler: a typed `Program`; `cpl_ir::effects::{Table, Effects}` for
  every purity gate; lowering that evaluates each expression once.
- tools: judge profiles (compiler, flags, source limits).
- stdlib: std bodies with the same cost as hand-written code, and documented
  costs.
- spec: a conformance case for each rule an optimization relies on (effect
  counts, evaluation order, traps).
- infra: the perf commands in CI, a quiet machine for timed runs.

## Quality register

Each row: the quality, its metric, the strongest mechanism that enforces it,
and its status. `missing` means no real check exists yet; building those rows
is perf's Milestone 2 work, highest user impact first. Targets and current
numbers live in `plan/domains/perf/metrics.toml`. Journeys are from
`handbook/product/journeys.md`.

| Quality | Metric | Mechanism | Status |
|---|---|---|---|
| Optimizations never change behaviour (J6, J9) | programs whose C++ output, effect count or trap differs from the interpreter | `cpl-conform diff`; `crates/cpl-cli/tests/differential.rs` (ASan/UBSan) with `tests/parity/`; `crates/cpl-cli/tests/fuzz_three_way.rs` | enforced for written programs |
| Same, on programs nobody wrote | mismatches per generated program (as Csmith does for GCC) | a generator of effectful programs run through both engines | missing |
| Each optimization is gated on proven purity | rewrites that reorder, drop or repeat a call not proven pure by `cpl_ir::effects` | the Uses/Effects tables (one decision point); `crates/cpl-codegen/src/sink_tests.rs`, `copy_tests.rs` | partial: per-pass gates remain |
| Each expression runs once (J5, J6) | calls evaluated more often than written | `tests/parity/view_receiver_runs_once.cp`; conformance `core-07` view cases | partial: no generated suite |
| Idiomatic code is as fast as matched C++ (J8, J9) | p50, p95, max runtime ratio on the perf set | `bench/v03_perf_bench.py` | measured, not gated |
| Tuned code matches the best C++ | `runtime_tuned_max`; outliers over 1.1x left | `bench/v03_perf_bench.py` (`tuned max` line); `perf.cp` per outlier | measured, not gated |
| Comparisons are fair | perf-set pairs with an unmatched algorithm; pairs skipped | matched references; `bench/unmatched.txt` lists each skip with its reason | process: reviewed per reference |
| Abstractions are free | zero-cost pairs within 1.05x of hand-written C++ and of the direct `.cp` | `cpl bench --zero-cost bench/zero-cost` (7 pairs) | measured, not gated |
| No hidden copies (J8) | deep copies of containers in generated C++ over corpus and conformance | `crates/cpl-cli/tests/read_only_copies.rs` (`deep_copies`, shrink-only); `crates/cpl-codegen/src/copy_tests.rs` | enforced |
| No hidden indirect calls | `std::function` in generated C++ | `read_only_copies.rs` (`std_functions`, shrink-only); `crates/cpl-codegen/src/devirt_tests.rs` | enforced |
| No hidden complexity blowup | programs whose time grows faster than the matched C++ across input sizes | scaled inputs in `bench/v03_perf_bench.py` (`gen.py` sizes) | partial: not compared per size |
| Redundant checks are dropped | `cpl::at` on indices a domain or range proves | `crates/cpl-codegen/src/tests.rs`, `range_tests.rs` shape tests | partial: no corpus count |
| Values take the fewest bytes their types allow | fields and elements wider than their declared range | `crates/cpl-codegen/src/tests.rs` narrowing tests | partial: no corpus count |
| Memory matches C++ (J8) | peak RSS ratio p50 and max | `bench/v03_perf_bench.py` RSS columns | measured, not gated |
| Submissions fit (J9) | programs over 32 KB, over 64 KB, per judge profile | `bench/sizes.py --check`; `crates/cpl-cli/tests/budget.rs`, `judges.rs` | enforced for 64 KB; 32 KB measured |
| Submissions compile everywhere (J9) | programs that build but fail g++ -std=c++17, per profile | `bench/compiles.py`; `crates/cpl-cli/tests/judges.rs` | partial: not every profile in CI |
| Fast to compile (J6, J9) | g++ -O2 time p50 and max | `bench/v03_perf_bench.py` compile column; `budget.rs` | partial |
| No undefined behaviour in the C++ | sanitizer reports | `differential.rs` under ASan/UBSan | enforced for its programs; corpus missing |
| Deterministic output | byte differences across two builds of the same program | none | missing |
| Costs are visible (J8) | implicit codegen choices `cpl explain` doesn't name (copies, closures, fusion, sinking, check elision, widths) | `cpl explain` text and `--json`; `crates/cpl-cli` explain tests | partial: copies and closures only |
| Explain never claims a guess as proven (J8) | explain lines whose claim the C++ contradicts | none | missing |
| Profiling points at source (J8) | `cpl profile` lines that map to no source line | `crates/cpl-cli/tests/profile.rs` | enforced |
| Trap build stays cheap | `--checks traps` time against the no-trap build | `bench/` trap-cost run (`metrics.toml`) | measured, not gated |
| A tuning knob does what it says | `unchecked:`, widths and layouts with no effect on the C++ | `crates/cpl-codegen/src/tests.rs` shape tests | partial |
| Perf regressions are caught at landing | landings that make a perf number worse unnoticed | ratchets above; a timed perf gate | missing for runtime |
| The perf suites catch what they claim | mutation score of the codegen shape tests and ratchets | none (as LLVM measures with mutation and fuzz campaigns) | missing |
| Code stays maintainable | codegen files over the line cap; duplicated reasoning | `scripts/debt-ratchet.py` (`rs_over_cap`, `backend_std_names`) | enforced |
| Every fix keeps its regression | perf fixes landed without a test or ratchet step | review at landing | process |

## Suites and tools

| Tool | Catches | Run |
|---|---|---|
| codegen unit and shape tests | C++ shapes the differential tests can't see | `cargo nextest run -p cpl-codegen` |
| differential and parity | interpreter and C++ disagreeing, including traps and effect counts | `cargo nextest run -p cpl-cli --test differential` |
| `cpl-conform diff` | the same over every conformance case | `./target/release/cpl-conform diff` |
| copy and `std::function` ratchets | a new copy or indirect call anywhere in corpus or conformance | `cargo nextest run -p cpl-cli --test read_only_copies` |
| perf set | runtime, RSS and compile-time ratios against matched C++; tuned max | `python3 bench/v03_perf_bench.py --suite dryrun,corpus,bench --runs 5 --min-ms 10 --jobs 1` |
| zero-cost pairs | an abstraction costing more than its hand-written form | `./target/release/cpl bench --zero-cost bench/zero-cost --reps 11` |
| sizes | a program over 32 KB or 64 KB | `python3 bench/sizes.py --cpl target/release/cpl --check -j 4` |
| compiles | C++ that g++ rejects | `python3 bench/compiles.py` |
| toolchain latency | slow check, run, build, LSP or stress | `python3 bench/rust_latency_audit.py --reps 1` |
| `cpl explain` | why a value was copied, how a closure is called | `cpl explain a.cp [--json]` |
