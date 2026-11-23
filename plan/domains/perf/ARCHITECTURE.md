# perf architecture

The code as it is on main (read 2026-11-22). Updated in the same commit as any structural change; reviewed by architecture & tech-debt.

## Modules

`crates/cpl-codegen` (16.3k lines; depends only on `cpl-ir` and `cpl-diag`) turns a lowered `cpl_ir::Program` into one C++17 file.

| File | Lines | What it is for |
|---|---|---|
| `src/lib.rs` | 352 | Public API: `emit`, `emit_for`, `emit_profiled`, `emit_within(_for)`, `sizes`, `Target`, `Part`, `Size`, `Fitted`. Fitting a judge's source limit: drop comments, then the source header, then halve indentation. |
| `src/emit.rs` | 2214 | The `Emitter`: whole-file printing, includes, types, globals, fn order, `main`, judge header and thread stack. |
| `src/expr.rs` | 2883 | Expressions, indexing (`cpl::at` vs unchecked), builtin calls, map/set ops, IR-shape recognizers (`map_remove_do`). |
| `src/stmt.rs` | 1811 | Statements and loops. |
| `src/fuse.rs` | 381 | Producer-consumer loop fusion: a loop over a `do { let v = []; for .. { v.push(e) }; give v }` runs its body at each `push` (`FusedPush`); taken only when the loops' effects don't interleave and one side cannot trap or stop early. |
| `src/bounds.rs` | 415 | Index checks taken out of a range loop: one test before `for i in lo..hi` covers `xs[i + c]` reached every round, when the body can only compute and store (no print, exit, resize or other trap); `Emitter::inside` holds what is covered. |
| `src/inline.rs` | 115 | A `for` over a call to a fn that is one list literal walks the literal with the arguments put in (stack array, no `std::vector`), when every argument is a variable or constant. |
| `src/divrem.rs` | 79 | Euclidean `/` `%` on `Int`: plain C++ when signs allow, shift/mask for a power-of-two divisor, else `cpl::ediv`/`emod`/`divp`/`modp`. |
| `src/runtime.rs` | 1767 | 100 runtime `Piece`s (I/O, `emod`, `at`, `reserve`, judge conversions), each emitted only when used; `unused_lines` trims the rest. |
| `src/show.rs` | 586 | Printing and reading values. |
| `src/names.rs` | 316 | C++ names and C++ types (`cty`, `elem_ty`), niche-aware. |
| `src/scope.rs` | 1850 | Where locals are declared (hoisting), moved-out values, row reuse, int and element narrowing (`narrow_ints`, `narrow_elems`). |
| `src/devirt.rs` | 1456 | Fn-typed fields/params that only ever hold one capture-free closure become direct calls (`ClosureMode`); feeds `cpl explain`. |
| `src/ranges.rs` | 645 | Sign facts so Euclidean `/` `%` print as plain C++ where they agree. |
| `src/niche.rs` | 60 | Which optionals fit in one scalar (`?Int` = one `Int`, `INT64_MIN` = none). |
| `src/rowcache.rs` | 221 | Hoists `a[i][j]` row address out of inner loops. |
| `src/unordered.rs` | 167 | Which `Map`/`Set` key types can be salted hash tables. |
| `src/walk.rs` | 3 | Re-export of `cpl_ir::walk`. |
| `src/tests.rs` | 1927 | 91 shape tests: the C++ text the differential tests can't see. |

`cpl bench` (perf owns the behaviour; the files sit in tools' crate `crates/cpl-contest`):

| File | What it is for |
|---|---|
| `crates/cpl-contest/src/bench.rs` | `cpl bench` dispatch, single-file worst-case bench. |
| `crates/cpl-contest/src/zero_cost.rs` | `cpl bench --zero-cost`: `bench/zero-cost/<pair>/{abstract,direct}.cp` + `hand.cpp`, 1.05x bar, kernel asm comparison. |
| `crates/cpl-contest/src/corpus_bench.rs` | `cpl bench --corpus`: `corpus/problems/*` vs `orig.cpp`, writes `research/perf/corpus-p95-results.json`. |

Python harnesses under `bench/`: `corpus_p95_bench.py` (0.2 corpus sweep; last report `research/perf/corpus-p95-report.md`), `rust_max_n_audit.py`, `rust_latency_audit.py` (toolchain latency against the P0 budgets: check, run, build, LSP, stress), per-area suites (`bench/trees/`, `bench/ranges/`, ...).

## Interfaces other domains use

| Interface | Consumers | Held by |
|---|---|---|
| `emit(program, name, source)` | `crates/cpl-conform/src/diff.rs` (interp vs C++), `crates/cpl-contest/src/zero_cost.rs`, `crates/cpl-cli/src/lib.rs` | `crates/cpl-cli/tests/differential.rs`, `crates/cpl-conform` |
| `emit_for(.., &Target)` | `crates/cpl-stress/src/program.rs` (stress builds) | `crates/cpl-stress` tests |
| `emit_within_for` + `sizes` + `Part` | `cpl build --judge [--size]` (`crates/cpl-cli/src/lib.rs`), `crates/cpl-wasm/src/lib.rs` (playground) | `crates/cpl-cli/tests/judges.rs`, `crates/cpl-cli/tests/budget.rs` |
| `emit_profiled` | `cpl profile` / `cpl cover` (`crates/cpl-cli/src/profile.rs`) | `crates/cpl-cli/tests/profile.rs` |
| `explain_closures`, `explain_dropped_fields`, `ClosureMode` | `cpl explain` (`crates/cpl-cli/src/lib.rs`) | `crates/cpl-cli` tests |
| `Target` (stamp, thread stack, windows) | built from `cpl-judge` profiles; `TARGET_BYTES` 32 KB / `HARD_BYTES` 64 KB in `crates/cpl-judge/src/lib.rs` | `crates/cpl-judge/src/tests.rs` |
| `research/perf/corpus-p95-results.json` schema | site (`web/lib/bench.ts`) | `corpus_bench::schema_tests` (both the writer and the committed file, checked against the TS interfaces) |

What codegen needs from the shared core (`cpl-ir`): the typed `Program`, `NarrowLayout`/`IntWidth`/`ElemLoc`, `pushed::{Feeds, FnSummary}`, `sites::user_stmts`, `walk`.

## Invariants

- One deterministic C++17 file per program; same input, same bytes.
- Interpreter and C++ agree on every program, including traps (`crates/cpl-cli/tests/differential.rs`, ASan/UBSan on).
- Every optimisation majorizes: it never changes output or a trap, and `cpl explain` can show it (closures today).
- An index check is dropped only when provable: today `proven()` in `src/expr.rs` accepts a domain-typed index or an `unchecked:`/raw cast, nothing else.
- Runtime pieces are emitted only when used, so a small program is a small file.
- Fitting to a source limit drops reader-only bytes first (comments, header source, indentation), never semantics.

## Known debt

- `src/expr.rs` (2883) and `src/emit.rs` (2214) are over the 2000-line cap, allowlisted in `scripts/debt-ratchet.json` (ratcheted down when fusion and `/` `%` moved out).
- Bounds-check proof is syntactic (`proven()`): ignores `ranges.rs` facts and `prover`; `for i in 1..n` over `[0; n]` still emits `cpl::at` (root of `strings/z_prefix`, `cf2178h`).
- `src/devirt.rs` tracks only `Home::Field` and `Home::Param`: `let f = mon.op; f(a, b)` falls back to `std::function` (root of `dryrun/p10_matching`).
- `src/ranges.rs` and `src/niche.rs` duplicate fact/layout reasoning that `plan/ARCHITECTURE.md` B3 step 6 moves into `cpl-ir` (shared with the interpreter and `prover`).
- IR-shape recognizers (`map_remove_do` in `src/expr.rs`) pattern-match lowered blocks; a lowering change silently loses the fast path. They need contract tests in `cpl-ir` or an explicit IR node.
- Benchmarks are scattered across two Rust runners and four Python scripts in `bench/` with different suites and bars; none runs in `scripts/check.sh`, so perf and size can regress silently.
- `research/perf/corpus-p95-report.md` is a 0.2 run at `6145b0b`; nothing current replaces it.
