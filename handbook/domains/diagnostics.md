# Diagnostics domain

| | |
|---|---|
| Kind | Domain charter |
| Owner | diagnostics |
| Describes | `crates/cpl-diag/`, `spec/diagnostics.md`, `spec/checker-lints/`, `crates/cpl-ir/src/fix.rs`, `crates/cpl-lint/src/rules/proven/`, `crates/cpl-cli/tests/{diagnostics,catalogue,recovery,newcomer,runtime_errors,fix_behavior}.rs`, `cpl explain`, `cpl fix` |

## Mission

When a program is wrong, the first thing CP Lang says is the one root cause,
at the place the contestant has to edit, in the spec's words, with a fix
that is right, and it says the same thing in every tool. A broken program
should cost one read and, where the fix is obvious, one keystroke. A
runtime trap reads like a teacher looking over your shoulder: the line, the
values, the rule and the CP Lang way.

Example: a C++ habit, `if a && b:`, gets

```
error[E0107]: `&&` is `and` in CP Lang
  --> main.cp:3:6
  |
3 | if a && b:
  |      ^^
  = help: replace with `and`
```

and `cpl fix` rewrites it. Because the fix is applied only after the result
re-checks clean, no fix ever leaves a program broken.

## Scope

| Owns | Does not own (owner) |
|---|---|
| `crates/cpl-diag`: `Diagnostic`, `Span`, `Fix`, `Sink`, rendering, `REGISTRY` (the source of `cpl explain`) | What is an error: the language rules (spec, compiler) |
| `spec/diagnostics.md`: every E, W, L and R code, its name and example message | The checks themselves at error sites in `cpl-syntax`, `cpl-check`, `cpl-ir` (compiler); diagnostics edits their messages, spans, fixes and suppression |
| Message wording, spans, labels, help and fixes at every error site | Style and idiom lints and their wording (lint-fmt); codes still go in the catalogue |
| The one fix applier, `cpl_ir::fix`, used by `cpl fix`, the editor's quick fixes and fix-all, and the playground | How the editor shows a diagnostic (tools, editors) |
| Checker lints (`L06xx`, `spec/checker-lints/`) and bound lints over declared ranges (`L03xx`, `crates/cpl-lint/src/rules/proven/`) | Conformance cases that pin a code (spec) |
| Runtime trap messages (`Trap::to_diagnostic` in `cpl-interp`) | Trap detection in the built C++ (codegen); the trace and debugger views (debugger) |
| Error-quality suites: goldens, recovery scoreboards, newcomer habits, the catalogue test | Error pages on the web site (site) |

## Interfaces

**Provides**

| Interface | Used by | Held by |
|---|---|---|
| `Diagnostic`, `Span`, `Severity`, `Fix`, `Sink` | every front-end and tool crate | `crates/cpl-diag` unit tests |
| `codes::*`, one constant per catalogue code | every error site | `crates/cpl-cli/tests/catalogue.rs` |
| `REGISTRY`: title, explanation, a trigger that emits exactly the code, a fix that makes it check clean | `cpl explain`, the catalogue test, `scripts/conformance-coverage.py` | `catalogue.rs` |
| `render`, `render_all`: the text form `error[E0301]: ... --> f.cp:7:17` | `cpl check/run/build`, goldens | `crates/cpl-cli/tests/diagnostics.rs` |
| `cpl_ir::fix`: picks among an error's candidate fixes by re-checking, applies them in rounds | `cpl fix`, `cpl-lsp`, `cpl-wasm` | `crates/cpl-ir/src/fix.rs` tests, `crates/cpl-lsp/src/tests/parity.rs`, `crates/cpl-wasm/tests/parity.rs` |

Invariants every consumer may rely on: a code never changes meaning and a
retired code is never reused; every emitted code is in the catalogue; spans
are byte ranges into the user's file; `cpl-diag` has no dependencies and no
language knowledge; `Sink` drops a diagnostic inside a span the parser
already reported, so one root cause gives one error.

**Needs**

| From | What |
|---|---|
| compiler | Error sites that call `codes::` constants and attach fixes; lowering that reports through `Sink` |
| stdlib | `std` that checks clean, so an error inside `std` is a library bug, not the user's |
| tools | The editor showing exactly the CLI's diagnostics and fixes |
| spec | Rule ids and one vocabulary (`spec/GLOSSARY.md`) for messages to cite |
| lint-fmt | Lint fixes applied through `cpl_lint::apply_lint_fixes` |

## Quality register

Each row is a quality a contestant, learner, editor user or library author
feels, the number that measures it, and the strongest mechanism that
enforces it. Mechanisms rank: a design that makes the failure impossible,
then a generated, differential or mutation suite, then a hand-written
suite, then a lint, ratchet or gate, then a process. `missing` marks a row
with no real check yet. Targets and current values are in
`plan/domains/diagnostics/metrics.toml`.

### Where the error points

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| The first error is on the line to edit | first error on the broken line, over every clean port broken 12 ways | generated suite, gated | `crates/cpl-cli/tests/recovery.rs` `recovery_scoreboard` |
| One error per root cause | cascaded errors per broken file | design (`Sink` poisoning) plus generated suite; lowering in `cpl-ir` still bypasses `Sink` | `recovery.rs` |
| No broken program checks clean | silent broken copies | generated suite, gated at 0 | `recovery.rs` |
| The primary span is never inside `std/` | user-facing errors whose primary span is in `std/` | `missing` | — |
| An error inside `std` names the std line and the user's call chain | chain present | hand-written | `crates/cpl-cli/tests/cli.rs` `an_error_inside_std_names_the_library_calls_that_led_there` |
| Spans are right on CRLF and non-ASCII sources | span and fix differences between LF and CRLF copies | hand-written | `crates/cpl-cli/tests/lsp_parity.rs` |
| Labels point at the other half (the declaration of a written `let`, the first of two borrows) | errors needing a second place that have a label | `missing` | — |

### What the message says

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Real mistakes get errors that name the place, the rule and a working fix | share of the real-mistake suite meeting all three | `missing`: goldens pin the text but nothing grades it | `crates/cpl-cli/tests/diagnostics.rs` (goldens only) |
| C++, Python and Rust habits get the CP Lang way | habits on the practice solutions whose first error is on the line and whose fix makes the tests pass | generated suite, gated | `crates/cpl-cli/tests/newcomer.rs` |
| Common mistakes (`&&`, `!x`, `let` for `var`, typos) get a fix | errors with a fix; fixes that restore the program | generated suite, gated | `recovery.rs` `fix_it_scoreboard` |
| Messages use the spec's vocabulary, one spelling per idea | banned spellings in messages | `missing`: `scripts/spec-vocab.py` checks the spec only | — |
| Every suggestion names something that exists and runs | did-you-means, helps and habit renames naming a missing or private item | generated suite for the editor's member catalogue; `missing` for every other advice path | `crates/cpl-cli/tests/builtin_catalog.rs` |
| Types read as the user wrote them (aliases, domains, views like `Rev<List<Int>>`) | goldens showing internal names | hand-written | `diagnostics.rs` |
| Unfinished code still gets useful errors | first error right on copies with a value cut off | generated suite | `recovery.rs` (cut-value class) |
| A message is understood in one read | rewrites needed by a cold reader on sampled errors | process: rubric in `handbook/guidelines/errors.md` | — |

### Fixes

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| An offered fix leaves a program that checks | fixes that leave errors | design: `cpl_ir::fix` applies a candidate only if the result re-checks; suites over every trigger and scoreboard | `crates/cpl-ir/src/fix.rs`, `catalogue.rs`, `recovery.rs` |
| A fix keeps what the program does | programs whose output changes after a fix | differential suite over every runnable program in the repository | `crates/cpl-cli/tests/fix_behavior.rs` |
| Fixes converge: applying them twice changes nothing | non-idempotent fixes | `missing` | — |
| CLI, editor and playground apply the same fixes | fix-all results differing from `cpl fix` | differential suite | `crates/cpl-wasm/tests/parity.rs` `fix_all_matches_cpl_fix` |
| Recoverable errors carry a fix | share with a fix | ratchet | `recovery.rs`, `plan/domains/diagnostics/metrics.toml` |

### Catalogue

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Every emitted code is catalogued with a trigger that emits exactly it and a clean fix | uncatalogued or untriggered codes | gate | `catalogue.rs` |
| Every code has a `cpl explain` page | codes without an explanation | gate | `catalogue.rs` |
| Every lint has a record whose `before.cp` fires and `after.cp`, `near_miss.cp` don't | records failing | generated suite | `crates/cpl-lint/src/record_tests.rs`, `checker_tests.rs` |
| Codes never change meaning; retired codes are never reused | a code whose name changed or a retired code emitted | process: append-only table in `spec/diagnostics.md`; `missing` as a check | — |
| A diagnostic links to a web page for its code | codes with no page | `missing` (with site) | — |

### Lints

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| No false positives on idiomatic code | lint warnings on the corpus, `std` and the practice solutions | ratchet, only goes down | `crates/cpl-lint/src/corpus_tests.rs` `corpus_warnings_only_go_down`, `std_tests.rs` |
| A bound lint's claim is true (the named input really overflows, divides by 0, empties the range) | claims whose witness does not trap when run | design: the witness comes from the range prover; `missing` as a suite that runs it | `crates/cpl-lint/src/rules/proven/` |
| Lints catch the bugs they name | mutation score of lint suites | `missing` | — |

### Runtime traps

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| A trap names the values, the line, the call stack and the test case | R-codes missing a part | hand-written goldens | `crates/cpl-cli/tests/runtime_errors.rs` |
| The interpreter and the built C++ stop on the same trap | traps that differ | differential suite | `crates/cpl-cli/tests/trap_parity.rs` |
| Every trap says the CP Lang fix | R-codes with no fix in help | `missing` | — |

### Every tool, every platform

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| The editor shows exactly what the CLI prints | files whose editor diagnostics differ | differential suite over every v0.3 file | `crates/cpl-lsp/src/tests/parity.rs`, `crates/cpl-cli/tests/lsp_parity.rs` |
| The playground shows the same | differences | differential suite | `crates/cpl-wasm/tests/parity.rs` |
| Neovim, VS Code and JetBrains show diagnostics and quick fixes on Linux, macOS and Windows | editor jobs failing | CI jobs | `.github/workflows/ci.yml` jobs `neovim`, `vscode`, `jetbrains` |
| `cpl check`, `cpl run` and `cpl build` report the same errors | commands disagreeing | `missing` | — |

### Robustness and speed

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| No input crashes the front end | panics on generated and mutated sources | `missing`: no front-end fuzzer asserts no panic | — |
| Diagnostics are deterministic | output differing between runs | `missing` as a check (goldens catch some) | — |
| The front end answers in under 500 ms per corpus file; diagnostics add under 10% | worst file, overhead | gate for the front end; `missing` for the overhead | `crates/cpl-cli/tests/cold_start.rs`, `budget.rs`, CI job `lsp-latency` |
| Suites catch regressions | mutation score of the error-quality suites | `missing` | — |
| Every escaped error-quality bug becomes a suite case | escaped bugs without a case | process: each bug report lands with its golden | `crates/cpl-cli/tests/diagnostics/` |

## Suites and tools

| Tool | What it catches | Run it |
|---|---|---|
| Goldens | Any change to the text of an error on a known file | `cargo test -p cpl-cli --test diagnostics`; `BLESS=1` rewrites |
| Catalogue | Codes missing from `spec/diagnostics.md`, triggers that emit another code, fixes that don't clean | `cargo test -p cpl-cli --test catalogue` |
| Recovery scoreboards | Errors off the broken line, cascades, silent copies, missing fixes | `cargo test -p cpl-cli --test recovery -- --nocapture` |
| Newcomer habits | C++ and Python habits whose error or fix fails | `cargo test -p cpl-cli --test newcomer` |
| Fix behaviour | Fixes that change program output | `cargo test -p cpl-cli --test fix_behavior` |
| Trap messages and parity | Traps missing values; interpreter vs C++ differences | `cargo test -p cpl-cli --test runtime_errors --test trap_parity` |
| Editor parity | Editor or playground showing other diagnostics or fixes | `cargo test -p cpl-lsp parity`, `cargo test -p cpl-wasm --test parity` |
| Lint records and precision | Lints firing on `after.cp` or the corpus | `cargo test -p cpl-lint` |
| `cpl explain E0301` | Reads one code's page | `cpl explain <code>` |

## Changes

- 2026-11-23: first version, folded from `plan/domains/diagnostics/README.md` and `ARCHITECTURE.md`.
