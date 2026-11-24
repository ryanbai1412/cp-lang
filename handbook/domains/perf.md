# Perf

| | |
|---|---|
| Kind | Domain charter |
| Owner | perf |
| Status | Active |
| Describes | `crates/cpl-codegen`, `bench/`, `cpl bench`, the codegen sections of `cpl explain` |

The perf domain owns the C++ that CP Lang submits: how fast it runs, how
big it is, how quickly it compiles, and whether a contestant can see and
control what it costs. If you have submitted a CP Lang solution, the file
the judge compiled came from this domain's code generator.

## What good looks like

A contestant writes the natural program, say a binary search over each row
of a list of lists, or a segment tree with `max` as its operation, and
submits it without a second thought about speed. It runs as fast as the C++
they would have written, because each row is read in place rather than
copied, and `max` is called directly rather than through a function
pointer. If one problem is tight, `cpl explain` tells them what the
generated code copies or calls indirectly, and they tighten it in CP Lang
(a narrower integer range, an `unchecked:` block) instead of rewriting it
in C++. Whatever they change for speed, the program prints exactly what it
printed before.

## How it fits together

The compiler hands codegen a typed, lowered program and a table of what
each function can do (write, print, trap). Codegen decides, in one place
each, how every value is stored (its width, whether it is a reference or a
copy), how every call is made (direct or indirect), and which checks a
proven range makes unnecessary. It then writes one C++17 file and fits it
under the judge's size limit.

Around it sit the measuring tools. `cpl bench` and the scripts in `bench/`
time CP Lang against matched C++ over a set of real problems, check the
zero-cost pairs, and measure size and compile time. Ratchet tests count the
copies and indirect calls left in the corpus's generated C++, and the
differential tests check that every optimization leaves both engines in
agreement.

## How we know it is good

The differential and conformance suites run every program through the
interpreter and the sanitized C++ and compare output, effect counts and
traps. Two shrink-only counts cover the whole corpus: copies of values that
are only read, and `std::function` calls. The perf set times every
corpus port against its contestant's C++ with matched algorithms, and the
zero-cost pairs time each abstraction against its hand-written form.

The weakest areas are that runtime is measured but not gated at landing,
the generated-program fuzzer covers only a fixed grammar of statements
around perf's rewrites, and `cpl explain` names copies and
closures but not yet fusion, sinking, check elision or widths. The rules
behind all of this are in `handbook/guidelines/performance.md` and its
checklist; the full list of qualities, with the check behind each, is the
quality register below.

## Where to start

- **Speeding up generated code:** read `handbook/guidelines/performance.md`
  and `handbook/design/codegen.md`, then the module for the choice in `crates/cpl-codegen/src/` (`uses/` for
  copies and moves, `devirt.rs` for direct calls, `ranges.rs` for checks).
  Add a shape test beside it and a parity case in
  `crates/cpl-cli/tests/parity/`.
- **Measuring:** `python3 bench/v03_perf_bench.py --suite dryrun,corpus,bench`
  for the perf set, `cpl bench --zero-cost bench/zero-cost` for abstractions.
- **A slow port:** compare it with its `orig.cpp`, then add a tuned
  `perf.cp` beside it or log the gap in `plan/domains/perf/PLAN.md`.

---

# Reference

The rest of this document is for looking things up: the exact scope, the
interfaces other domains rely on, every quality we track, and the suites
that check them.

## Scope

Owns:
- `crates/cpl-codegen`: C++17 emission, layout and widths, niches,
  devirtualization, bounds-check elimination, fusion, sinking, lending and
  moves (the Uses table), and fitting a judge's source limit.
- `cpl bench` behaviour (`crates/cpl-contest/src/{bench,zero_cost,corpus_bench}.rs`,
  files kept in tools' crate) and every harness under `bench/`.
- The perf set: matched C++ references (`orig.cpp`, `_ref.cpp`), tuned
  variants (`perf.cp` beside `port.cp`) and their accounting
  (`bench/oversize.txt`).
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

Each row: the quality, its metric, the strongest mechanism that enforces it
(with the rule it enforces, from `handbook/guidelines/performance-checklist.md`),
and its status. `missing` means no real check exists yet; building those rows
is perf's Milestone 2 work, highest user impact first. Targets and current
numbers live in `plan/domains/perf/metrics.toml`. Journeys are from
`handbook/product/journeys.md`.

| Quality | Metric | Mechanism | Status |
|---|---|---|---|
| Optimizations never change behaviour (J6, J9) | programs whose C++ output, effect count or trap differs from the interpreter | `cpl-conform diff`; `crates/cpl-cli/tests/differential.rs` (ASan/UBSan) with `crates/cpl-cli/tests/parity/`; `crates/cpl-cli/tests/fuzz_three_way.rs` ([performance/R9](../guidelines/performance-checklist.md)) | enforced for written programs |
| Same, on programs nobody wrote | mismatches per generated program (as Csmith does for GCC) | `crates/cpl-cli/tests/effect_fuzz.rs`: seeded effectful programs, interpreter against ASan/UBSan C++ on trap and non-trap inputs ([performance/R9](../guidelines/performance-checklist.md)) | partial: one statement grammar |
| Each optimization is gated on proven purity | rewrites that reorder, drop or repeat a call not proven pure by `cpl_ir::effects` | the Uses/Effects tables (one decision point); `crates/cpl-codegen/src/sink_tests.rs`, `copy_tests.rs` ([performance/R10](../guidelines/performance-checklist.md)) | partial: per-pass gates remain |
| Each expression runs once (J5, J6) | calls evaluated more often than written | `crates/cpl-cli/tests/parity/view_receiver_runs_once.cp`; conformance `core-07` view cases ([performance/R9](../guidelines/performance-checklist.md)) | partial: no generated suite |
| Idiomatic code is as fast as matched C++ (J8, J9) | p50, p95, max runtime ratio on the perf set | `bench/v03_perf_bench.py` ([performance/R5](../guidelines/performance-checklist.md)) | measured, not gated |
| Tuned code matches the best C++ | `runtime_tuned_max`, `memory_tuned_max`; outliers over 1.1x on either left | `bench/v03_perf_bench.py` (`tuned max` line); `perf.cp` per outlier ([performance/R6](../guidelines/performance-checklist.md)) | measured, not gated |
| Comparisons are fair | perf-set pairs with an unmatched algorithm; pairs skipped | matched references, every pair in p50/max (no exemption list; `bench_lists` counts any) ([performance/R1](../guidelines/performance-checklist.md), [R4](../guidelines/performance-checklist.md)) | process: reviewed per reference |
| Abstractions are free | zero-cost pairs within 1.05x of hand-written C++ and of the direct `.cp` | `cpl bench --zero-cost bench/zero-cost` (7 pairs) ([performance/R7](../guidelines/performance-checklist.md)) | measured, not gated |
| No hidden copies (J8) | deep copies of containers in generated C++ over corpus and conformance | `crates/cpl-cli/tests/read_only_copies.rs` (`deep_copies`, shrink-only); `crates/cpl-codegen/src/copy_tests.rs` ([performance/R12](../guidelines/performance-checklist.md)) | enforced |
| No hidden indirect calls | `std::function` in generated C++ | `read_only_copies.rs` (`std_functions`, shrink-only); `crates/cpl-codegen/src/devirt_tests.rs` ([performance/R13](../guidelines/performance-checklist.md)) | enforced |
| No hidden complexity blowup | programs whose time grows faster than the matched C++ across input sizes | scaled inputs in `bench/v03_perf_bench.py` (`gen.py` sizes) ([performance/R2](../guidelines/performance-checklist.md)) | partial: not compared per size |
| Redundant checks are dropped | `cpl::at` on indices a domain or range proves | `crates/cpl-codegen/src/tests.rs`, `range_tests.rs` shape tests | partial: no corpus count |
| Values take the fewest bytes their types allow | fields and elements wider than their declared range | `crates/cpl-codegen/src/tests.rs` narrowing tests | partial: no corpus count |
| Memory matches C++ (J8) | peak RSS ratio p95 (bar 1.5x) and max (bar 2x), programs over 1.5x | `bench/v03_perf_bench.py` `peak RSS` line ([performance/R5](../guidelines/performance-checklist.md)) | measured, not gated |
| Submissions fit (J9) | programs over 32 KB, over 64 KB, per judge profile | `bench/sizes.py --check`; `crates/cpl-cli/tests/budget.rs`, `judges.rs` ([performance/R8](../guidelines/performance-checklist.md)) | enforced for 64 KB; 32 KB measured |
| Submissions compile everywhere (J9) | programs that build but fail g++ -std=c++17, per profile | `bench/compiles.py`; `crates/cpl-cli/tests/judges.rs` | partial: not every profile in CI |
| Fast to compile (J6, J9) | g++ -O2 time p50 and max | `bench/v03_perf_bench.py` compile column; `budget.rs` | partial |
| No undefined behaviour in the C++ | sanitizer reports | `crates/cpl-cli/tests/differential.rs` and `crates/cpl-cli/tests/fuzz_three_way.rs` under ASan/UBSan ([performance/R9](../guidelines/performance-checklist.md)) | partial: the corpus isn't run sanitized |
| Deterministic output | byte differences across two builds of the same program | `crates/cpl-cli/tests/deterministic_cpp.rs`: every corpus and conformance program built twice ([performance/R18](../guidelines/performance-checklist.md)) | enforced |
| Costs are visible (J8) | implicit codegen choices `cpl explain` doesn't name (copies, closures, fusion, sinking, check elision, widths) | `cpl explain` text and `--json`; `crates/cpl-cli` explain tests ([performance/R17](../guidelines/performance-checklist.md)) | partial: copies and closures only |
| Explain never claims a guess as proven (J8) | explain lines whose claim the C++ contradicts | `crates/cpl-cli/tests/explain_accuracy.rs`: every corpus and conformance program's `cpl explain --json` against its C++ (direct calls, `std::function`s, copies) ([performance/R17](../guidelines/performance-checklist.md)) | partial: closures and copies |
| Profiling points at source (J8) | `cpl profile` lines that map to no source line | `crates/cpl-cli/tests/profile.rs` | enforced |
| Trap build stays cheap | `--checks traps` time against the no-trap build | `bench/` trap-cost run (`metrics.toml`) | measured, not gated |
| A tuning knob does what it says | `unchecked:`, widths and layouts with no effect on the C++ | `crates/cpl-codegen/src/tests.rs` shape tests | partial |
| Perf regressions are caught at landing | landings that make a perf number worse unnoticed | ratchets above; a timed perf gate ([performance/R14](../guidelines/performance-checklist.md)) | missing for runtime |
| The perf suites catch what they claim | mutation score of the codegen shape tests and ratchets | none (as LLVM measures with mutation and fuzz campaigns) | missing |
| Code stays maintainable | codegen files over the line cap; duplicated reasoning | `scripts/debt-ratchet.py` (`rs_over_cap`, `backend_std_names`) ([performance/R16](../guidelines/performance-checklist.md)) | enforced |
| Every fix keeps its regression | perf fixes landed without a test or ratchet step | review at landing ([performance/R11](../guidelines/performance-checklist.md)) | process |

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

## Changes

Changes to this charter land like code, with `Domain: perf`. A new quality
gets a register row (with `missing` until its check exists); a new rule goes
in `handbook/guidelines/performance-checklist.md` and the row cites it.
