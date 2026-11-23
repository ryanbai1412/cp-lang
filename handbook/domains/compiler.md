# Compiler domain charter

| | |
|---|---|
| Owner | Compiler |
| Status | Active |

## Mission

Every program the spec defines means exactly what the spec says, in the
interpreter and in the generated C++. Every program the spec rejects is
rejected with one error naming the rule it breaks. There is one front end, one
IR and one interpreter engine, and no std name is special in the compiler.

Example: `let a = [10, 20, 30]; print(a.rev.take(2))` prints `30 20` from
`cpl run` and from the built C++. Before that holds, it is a known gap with a
conformance case, never a silent wrong answer.

## Scope

Owns:
- Crates: `cpl-syntax` (lexer, parser, lossless tree), `cpl-check` (names,
  types, domains, lends, the incremental `Database`), `cpl-ir` (lowering,
  monomorphization, the typed `Program`, shared facts such as
  `cpl_ir::effects`, `pushed`, `show`, `sites`), `cpl-interp` (the checked
  run: VM, traces, replay, profile), `cpl-prover` (integer reasoning).
- Commands' meaning: what `cpl check` and `cpl run` accept, reject and print.
- The shared-core interfaces other domains build on (below).

Doesn't own:
- C++ emission, layout, C++ size and speed: perf (`cpl-codegen`). A
  disagreement between the interpreter and the C++ is fixed by whichever side
  is wrong.
- Diagnostic wording and codes: diagnostics. The compiler decides when an
  error fires; diagnostics decides how it reads.
- What the language means and the conformance cases: spec. The compiler never
  edits a case; a case that looks wrong goes to spec.
- Library bodies (`std/`): stdlib. LSP and CLI commands: tools. Trace viewer
  and stress: debugger.

## Interfaces

Provides:

| Interface | API | Main consumers |
|---|---|---|
| Lossless syntax tree and spans | `cpl_syntax::{parse_file, ParsedFile, ast, visit, split_items, lex}` | fmt, lint, LSP, docs, stress |
| Typed tree and queries | `cpl_check::{Database, Analysis, TypedIndex, Hover, Definition, Ty, Def}`; `ItemCheck.resolved` (where each name resolves) | LSP, CLI, wasm, lint, codegen |
| IR | `cpl_ir::{lower, Program, Function, Stmt, Expr, Ty, Builtin}` | codegen, interp, CLI, stress, judge |
| Call effects | `cpl_ir::effects::{Table, Effects}`: per fn and closure, the globals written, I/O, may-trap, unknown calls | codegen, lint |
| Decided-once facts | `cpl_ir::{pushed, show, sites}`: pushed loops, printing, statement sites | both back ends |
| Run, trace, replay | `cpl_interp::{run, run_with, run_observed, run_profiled, Trap, Limits, trace, replay}` | CLI, stress, trace viewer, judge |
| Integer facts | `cpl_prover` (intervals, laws, lends) | lint |

Needs:
- spec: a conformance case for every rule, and a ruling when a case and the
  spec disagree.
- stdlib: std bodies that check under the same rules as a user library.
- perf: C++ that agrees with the interpreter on every program.
- diagnostics: wording and codes for every error the compiler raises.
- tech-debt: the ratchets in `scripts/debt-ratchet.py` and their review.

## Quality register

Each row is a quality the domain must be high on, its metric, and the
strongest mechanism that enforces it. `missing` means no real check exists
yet; those rows are the domain's work toward Milestone 2. Current numbers and
targets are in `plan/domains/compiler/metrics.toml`.

| Quality | Metric | Mechanism | Status |
|---|---|---|---|
| Programs mean what the spec says | conformance cases passing, of all | `cpl-conform conformance` over `spec/conformance/` (generated from spec rules) | enforced; `--check` blocks regressions |
| Interpreter and C++ agree | cases where the two outputs differ; cases skipped | `cpl-conform diff`; `crates/cpl-cli/tests/differential.rs` over `crates/cpl-cli/tests/programs/`; rule 4 of [compiler-changes](../guidelines/compiler-changes.md#the-rules) | enforced; skips only shrink |
| Agreement on unusual input | mismatches over seeded degenerate inputs | `crates/cpl-cli/tests/fuzz_three_way.rs` | enforced for its families |
| No silent wrong answer | conformance cases with wrong output | `cpl-conform conformance --causes` (`wrong output` group) | enforced as a count |
| A rejected program names its rule | rejected cases whose expected code matches | `cpl-conform conformance` (expected `error[...]` per case) | enforced |
| One error per root cause, at the right line | extra or misplaced diagnostics on broken code | `crates/cpl-cli/tests/recovery.rs` | enforced |
| Front end never panics | panics over arbitrary input | `crates/cpl-cli/tests/recovery.rs`; a parser/checker fuzzer | partial: fuzzer missing |
| Internal errors never reach a user | `R0001` / internal traps over conformance and corpus | `cpl-conform corpus` and `conformance` (internal traps fail the case) | enforced |
| Generic std is checked at its definition, so no error starts inside std | errors `cpl check std/<file>.cp` reports; diagnostics whose primary span is in `std/` on any user program | `crates/cpl-cli/tests/std_checks.rs` (count only drops, toward 0); a check that no conformance or corpus diagnostic points into `std/` | partial: the span check is missing |
| Std bounds mean what they say | generic calls that type-check by unwrapping a type that doesn't meet the bound | trait-bound checks at the call (`cpl-check`), conformance cases such as `core-07`; rule 5 of [compiler-changes](../guidelines/compiler-changes.md#the-rules) | partial: no suite generates bound violations |
| No std privilege | std names matched in check, IR and interp | `ir_builtin_names`, `prelude_privilege`, `backend_std_names`, `lint_std_names` ratchets in `scripts/debt-ratchet.py`; rule 1 of [compiler-changes](../guidelines/compiler-changes.md#the-rules) | enforced; only shrink |
| One interpreter engine | VM fallbacks to the tree-walker | `vm_fallbacks` and `tree_walker` ratchets; `CPL_VM_STATS=json` | enforced as a count; target 0, tree-walker deleted |
| Runs are deterministic | output differences across repeated runs | `crates/cpl-interp/tests/checkpoint.rs`, trace goldens (`crates/cpl-cli/tests/trace_goldens.rs`) | partial |
| Traps show source values | traps without values or span | `crates/cpl-cli/tests/runtime_errors.rs` | enforced |
| Editor and CLI agree | files where LSP and `cpl check` differ | `crates/cpl-cli/tests/lsp_parity.rs` | enforced |
| Incremental: an edit re-checks only what depends on it | items re-checked per edit | none | missing |
| Fast front end | `cpl check` time on the largest corpus program; cold start | `crates/cpl-cli/tests/cold_start.rs`; the latency metric in `metrics.toml` | partial: no gate under 500 ms |
| Proofs are sound | wrong verdicts against brute force | `crates/cpl-prover/tests/fuzz.rs`, `laws.rs` | enforced |
| Std module graph is acyclic | cycles | `crates/cpl-check/tests/std_acyclic.rs` | enforced |
| Every spec rule has a case (J2, J5) | rules in `spec/` with no conformance case | `scripts/spec-check.sh` (spec owns the cases; compiler asks for the gaps) | partial |
| Docs examples run as written (J2) | `/// example:` lines and spec code blocks that fail | `cpl-conform all` (spec blocks); `crates/cpl-cli/tests/builtin_catalog.rs` | enforced |
| Every error code the compiler raises is documented and reachable (J2) | codes raised but not in `spec/diagnostics.md`, or listed and never raised | `crates/cpl-cli/tests/catalogue.rs` | enforced |
| A quick fix keeps what the program does (J5, J10) | fixes that change output or leave an error | `crates/cpl-cli/tests/fix_behavior.rs` | enforced |
| Editor features answer on every file (J5) | hover, definition, references, completion failures | `crates/cpl-cli/tests/lsp_v03.rs` (tools owns the suite; compiler owns the queries) | enforced |
| An incomplete program still checks and runs its other items (J5, J6) | items lost to an error elsewhere | `crates/cpl-cli/tests/recovery.rs` | partial: no "run with holes" case suite |
| Declared input ranges are validated alike in both engines (J6) | inputs one engine accepts and the other rejects | conformance `read` cases; `crates/cpl-cli/tests/fuzz_three_way.rs` | partial |
| Interactive input and output are line-timely (J6) | answers delayed until stdin closes | `crates/cpl-cli/tests/live_stdin.rs` | enforced |
| Overflow and checked operations trap alike in both engines (J6, J7) | checked operations that trap in one engine only | conformance trap cases; `crates/cpl-cli/tests/differential.rs` | partial: no generated overflow suite |
| A checked and an unchecked build agree on valid programs (J9) | outputs that differ between `--checks` levels | none | missing |
| Deep recursion behaves as on the judge (J6, J9) | programs that overflow locally but not on the judge, or the reverse | `crates/cpl-cli/tests/stack/`; conformance `deep-recursion-trap` | partial |
| Traces and replay show what ran (J7) | trace steps that don't match execution | `crates/cpl-cli/tests/trace_goldens.rs`, `crates/cpl-cli/tests/replay.rs` | enforced |
| The interpreter is fast enough to test with (J6, J8) | slowest corpus run in `cpl run`; VM time against C++ | `crates/cpl-cli/tests/interp_ratio.rs` | partial: no corpus-wide gate |
| Interpreter memory fits contest inputs (J6) | peak memory on the largest corpus inputs | none | missing |
| The C++ builds on every judge profile (J9) | ports that fail to compile or exceed the source limit | `crates/cpl-cli/tests/judges.rs` | enforced |
| Old solutions keep building (J10) | corpus programs that stop checking after a change | `cpl-conform corpus`; `crates/cpl-cli/tests/judges.rs` ports | enforced |
| A user library has the powers std has (J10) | std-only mechanisms | `prelude_privilege`, `backend_std_names`, `ir_builtin_names`, `lint_std_names` in `scripts/debt-ratchet.py`; rule 1 of [compiler-changes](../guidelines/compiler-changes.md#the-rules) | partial |
| Suites catch the bugs they claim to | mutation score of the conformance and differential suites | `scripts/mutation-score.py` on the code a diff touches | partial: not yet in the gate |
| Generated programs find disagreements (as Csmith does for GCC and LLVM) | mismatches per 100k generated programs | a differential program generator, also run at each `--checks` level | missing |
| Code stays maintainable | owned files over the size cap; duplicated bodies | `rs_over_cap`, `dup_fn_bodies`, `near_dup_fns` ratchets | enforced |
| Every fix keeps its regression | fixes landed without a case or program | the checklist in [compiler-changes](../guidelines/compiler-changes.md#checklist) | process |

## Suites and tools

| Tool | Catches | Run |
|---|---|---|
| `cpl-conform conformance` | a case whose output or error differs from its expectation; `--causes` groups failures by cause | `./target/release/cpl-conform conformance --causes` |
| `cpl-conform diff` | interpreter and C++ disagreeing on a case | `./target/release/cpl-conform diff` |
| `cpl-conform all --check` | any regression against the recorded results | `./target/release/cpl-conform all --check` |
| differential programs | interpreter and C++ disagreeing on `crates/cpl-cli/tests/programs/` | `cargo nextest run -p cpl-cli --test differential` |
| owned-crate unit tests | regressions inside a crate | `cargo nextest run -p cpl-syntax -p cpl-check -p cpl-ir -p cpl-interp -p cpl-prover` |
| `CPL_VM_STATS=json` | statements and expressions the VM hands to the tree-walker | `CPL_VM_STATS=json cpl run a.cp < in` |
| debt ratchets | std names, fallbacks, file sizes, duplication growing | `python3 scripts/debt-ratchet.py --check` |
| `scripts/land.sh` | anything above failing on the rebased tip | `scripts/land.sh` |
