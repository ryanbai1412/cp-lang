# Diagnostics domain

| | |
|---|---|
| Kind | Domain charter |
| Owner | diagnostics |
| Describes | `crates/cpl-diag/`, `spec/diagnostics.md`, `spec/checker-lints/`, `crates/cpl-ir/src/fix.rs`, `crates/cpl-lint/src/rules/proven/`, `crates/cpl-cli/tests/{diagnostics,catalogue,recovery,newcomer,runtime_errors,fix_behavior}.rs`, `cpl explain`, `cpl fix` |

The diagnostics domain owns what CP Lang says when a program is wrong:
compile errors, warnings from the checker and from declared bounds, runtime
traps, and the fixes that `cpl fix` and the editor apply. If you have read an
error from `cpl check`, pressed a quick fix in the editor, or read a trap
message from `cpl run`, you have used this domain's work.

## What good looks like

A contestant used to C++ writes `if a && b:` and gets

```
error[E0107]: `&&` is `and` in CP Lang
  --> main.cp:3:6
  |
3 | if a && b:
  |      ^^
  = help: replace with `and`
```

One read tells them the line, the rule and the edit, and `cpl fix` makes it.
A contestant who writes a comparator where a sort key belongs,
`rows.sort(by: (a, b) => (-b, a))`, is told that `sort(by:)` takes a key of
one element, with the caret on `(a, b)` and a key like
`x => (x.name, x.score)` to start from. A program that overflows on hidden
input stops with the line, both operands and the advice to use `I128`. The
same file shows the same errors and fixes in the terminal, the editor and the
browser playground.

## How it fits together

- **One vocabulary.** `crates/cpl-diag` defines a diagnostic (code, span,
  message, labels, help, fixes) and has no language knowledge. Every code is
  listed in `spec/diagnostics.md` and in a registry that `cpl explain` reads.
- **Errors are raised where the rule lives.** The parser, checker and lowerer
  report through one `Sink`, which drops a second report inside a span already
  reported, so one root cause gives one error. Diagnostics owns the wording,
  the span and the fix at each site; the compiler domain owns the check.
- **One fix applier.** `cpl_ir::fix` tries each candidate fix, keeps it only if
  the program then checks clean, and serves `cpl fix`, the editor's quick fixes
  and fix-all, and the playground.
- **Lints that prove their claim.** Checker lints (`L06xx`) and bound lints
  over declared ranges (`L03xx`) warn only when they can show the problem, for
  example an input inside the declared ranges that overflows.
- **Traps in source terms.** The interpreter turns a runtime failure into a
  diagnostic with the values, the call stack and the test case.

## How we know it is good

The rules every error follows are in `handbook/guidelines/errors-checklist.md`.
Generated suites break every clean program in the repository a dozen ways and
check that the first error is on the broken line, that nothing cascades and
that the fix restores the program. Differential suites check that a fix never
changes what a program prints and that the editor and playground match the
CLI. A suite of real mistakes, each one a user actually hit, checks that the
error names the place, the rule and a working fix. Gates keep every code
catalogued and no error pointing into `std/`.

The weakest areas are labels for the second place an error involves, one
vocabulary across messages, checks that every suggestion names a real item,
a front-end fuzzer, and cascades from the lowerer, which does not yet report
through `Sink`. The quality register below lists every quality with the
check behind it, and marks the ones with none as `missing`.

## Where to start

- **Improving an error:** find its code with `cpl check --json`, change the
  message or fix at the site that emits it, and update the golden with
  `BLESS=1 cargo test -p cpl-cli --test diagnostics`.
- **A user found an error unhelpful:** add the file to
  `crates/cpl-cli/tests/mistakes/` first, then fix the error until it passes.
- **Adding a code:** follow "Codes" in `handbook/guidelines/errors-checklist.md`.

---

# Reference

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
enforces it, citing the rule in `handbook/guidelines/errors-checklist.md`. Mechanisms rank: a design that makes the failure impossible,
then a generated, differential or mutation suite, then a hand-written
suite, then a lint, ratchet or gate, then a process. `missing` marks a row
with no real check yet. Targets and current values are in
`plan/domains/diagnostics/metrics.toml`.

### Where the error points

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| The first error is on the line to edit | first error on the broken line, over every clean port broken 12 ways | `errors/R2`: generated suite, gated | `crates/cpl-cli/tests/recovery.rs` `recovery_scoreboard` |
| One error per root cause | cascaded errors per broken file | `errors/R1`: design (`Sink` poisoning) plus generated suite; lowering in `cpl-ir` still bypasses `Sink` | `recovery.rs` |
| No broken program checks clean | silent broken copies | `errors/R1`: generated suite, gated at 0 | `recovery.rs` |
| The primary span is never inside `std/` | user-facing errors whose primary span is in `std/` | `errors/R3`: gate over the goldens; `missing` over generated programs | `crates/cpl-cli/tests/diagnostics.rs` `no_golden_error_points_into_std` |
| An error inside `std` names the std line and the user's call chain | chain present | `errors/R3`: hand-written | `crates/cpl-cli/tests/cli.rs` `an_error_inside_std_names_the_library_calls_that_led_there` |
| Spans are right on CRLF and non-ASCII sources | span and fix differences between LF and CRLF copies | `errors/R12`: hand-written | `crates/cpl-cli/tests/lsp_parity.rs` |
| Labels point at the other half (the declaration of a written `let`, the first of two borrows) | errors needing a second place that have a label | `errors/R4`: `missing` | — |

### What the message says

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Real mistakes get errors that name the place, the rule and a working fix | share of the real-mistake suite meeting all three | `errors/R15`: hand-written suite, graded on all three, with no known-failure list | `crates/cpl-cli/tests/mistakes/`, `diagnostics.rs` `real_mistakes_name_the_place_the_rule_and_a_fix` |
| C++, Python and Rust habits get the CP Lang way | habits on the practice solutions whose first error is on the line and whose fix makes the tests pass | `errors/R8`: generated suite, gated | `crates/cpl-cli/tests/newcomer.rs` |
| Common mistakes (`&&`, `!x`, `let` for `var`, typos) get a fix | errors with a fix; fixes that restore the program | `errors/R8`: generated suite, gated | `recovery.rs` `fix_it_scoreboard` |
| Messages use the spec's vocabulary, one spelling per idea | banned spellings in messages | `errors/R5`: `missing`: `scripts/spec-vocab.py` checks the spec only | — |
| Every suggestion names something that exists and runs | did-you-means, helps and habit renames naming a missing or private item | `errors/R7`: generated suite for the editor's member catalogue; `missing` for every other advice path | `crates/cpl-cli/tests/builtin_catalog.rs` |
| Types read as the user wrote them (aliases, domains, views like `Rev<List<Int>>`) | goldens showing internal names | `errors/R6`: hand-written | `diagnostics.rs` |
| Unfinished code still gets useful errors | first error right on copies with a value cut off | `errors/R2`: generated suite | `recovery.rs` (cut-value class) |
| A message is understood in one read | rewrites needed by a cold reader on sampled errors | `errors/R5`: process: rubric in `handbook/guidelines/errors.md` | — |

### Fixes

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| An offered fix leaves a program that checks | fixes that leave errors | `errors/R9`: design: `cpl_ir::fix` applies a candidate only if the result re-checks; suites over every trigger and scoreboard | `crates/cpl-ir/src/fix.rs`, `catalogue.rs`, `recovery.rs` |
| A fix keeps what the program does | programs whose output changes after a fix | `errors/R9`: differential suite over every runnable program in the repository | `crates/cpl-cli/tests/fix_behavior.rs` |
| Fixes converge: applying them twice changes nothing | non-idempotent fixes | `errors/R9`: `missing` | — |
| CLI, editor and playground apply the same fixes | fix-all results differing from `cpl fix` | `errors/R12`: differential suite | `crates/cpl-wasm/tests/parity.rs` `fix_all_matches_cpl_fix` |
| Recoverable errors carry a fix | share with a fix | `errors/R8`: ratchet | `recovery.rs`, `plan/domains/diagnostics/metrics.toml` |

### Catalogue

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Every emitted code is catalogued with a trigger that emits exactly it and a clean fix | uncatalogued or untriggered codes | `errors/R13`: gate | `catalogue.rs` |
| Every code has a `cpl explain` page | codes without an explanation | `errors/R13`: gate | `catalogue.rs` |
| Every lint has a record whose `before.cp` fires and `after.cp`, `near_miss.cp` don't | records failing | `errors/R13`: generated suite | `crates/cpl-lint/src/record_tests.rs`, `checker_tests.rs` |
| Codes never change meaning; retired codes are never reused | a code whose name changed or a retired code emitted | `errors/R14`: process: append-only table in `spec/diagnostics.md`; `missing` as a check | — |
| A diagnostic links to a web page for its code | codes with no page | `errors/R13`: `missing` (with site) | — |

### Lints

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| No false positives on idiomatic code | lint warnings on the corpus, `std` and the practice solutions | `errors/R10`: ratchet, only goes down | `crates/cpl-lint/src/corpus_tests.rs` `corpus_warnings_only_go_down`, `std_tests.rs` |
| A bound lint's claim is true (the named input really overflows, divides by 0, empties the range) | claims whose witness does not trap when run | `errors/R16`: design: the witness comes from the range prover; `missing` as a suite that runs it | `crates/cpl-lint/src/rules/proven/` |
| Lints catch the bugs they name | mutation score of lint suites | `errors/R10`: `missing` | — |

### Runtime traps

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| A trap names the values, the line, the call stack and the test case | R-codes missing a part | `errors/R11`: hand-written goldens | `crates/cpl-cli/tests/runtime_errors.rs` |
| The interpreter and the built C++ stop on the same trap | traps that differ | `errors/R11`: differential suite | `crates/cpl-cli/tests/trap_parity.rs` |
| Every trap says the CP Lang fix | R-codes with no fix in help | `errors/R11`: `missing` | — |

### Every tool, every platform

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| The editor shows exactly what the CLI prints | files whose editor diagnostics differ | `errors/R12`: differential suite over every v0.3 file | `crates/cpl-lsp/src/tests/parity.rs`, `crates/cpl-cli/tests/lsp_parity.rs` |
| The playground shows the same | differences | `errors/R12`: differential suite | `crates/cpl-wasm/tests/parity.rs` |
| Neovim, VS Code and JetBrains show diagnostics and quick fixes on Linux, macOS and Windows | editor jobs failing | `errors/R12`: CI jobs | `.github/workflows/ci.yml` jobs `neovim`, `vscode`, `jetbrains` |
| `cpl check`, `cpl run` and `cpl build` report the same errors | commands disagreeing | `errors/R12`: `missing` | — |

### Robustness and speed

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| No input crashes the front end | panics on generated and mutated sources | `missing`: no front-end fuzzer asserts no panic | — |
| Diagnostics are deterministic | output differing between runs | `missing` as a check (goldens catch some) | — |
| The front end answers in under 500 ms per corpus file; diagnostics add under 10% | worst file, overhead | gate for the front end; `missing` for the overhead | `crates/cpl-cli/tests/cold_start.rs`, `budget.rs`, CI job `lsp-latency` |
| Suites catch regressions | mutation score of the error-quality suites | `missing` | — |
| Every escaped error-quality bug becomes a suite case | escaped bugs without a case | `errors/R15`: process: each bug report lands with its golden | `crates/cpl-cli/tests/diagnostics/` |

## Suites and tools

| Tool | What it catches | Run it |
|---|---|---|
| Goldens and real mistakes | Any change to the text of an error on a known file; a real mistake whose error misses the place, the help or a clean fix; a golden pointing into `std/` | `cargo test -p cpl-cli --test diagnostics`; `BLESS=1` rewrites |
| Catalogue | Codes missing from `spec/diagnostics.md`, triggers that emit another code, fixes that don't clean | `cargo test -p cpl-cli --test catalogue` |
| Recovery scoreboards | Errors off the broken line, cascades, silent copies, missing fixes | `cargo test -p cpl-cli --test recovery -- --nocapture` |
| Newcomer habits | C++ and Python habits whose error or fix fails | `cargo test -p cpl-cli --test newcomer` |
| Fix behaviour | Fixes that change program output | `cargo test -p cpl-cli --test fix_behavior` |
| Trap messages and parity | Traps missing values; interpreter vs C++ differences | `cargo test -p cpl-cli --test runtime_errors --test trap_parity` |
| Editor parity | Editor or playground showing other diagnostics or fixes | `cargo test -p cpl-lsp parity`, `cargo test -p cpl-wasm --test parity` |
| Lint records and precision | Lints firing on `after.cp` or the corpus | `cargo test -p cpl-lint` |
| `cpl explain E0301` | Reads one code's page | `cpl explain <code>` |

## Changes

- 2026-11-23: prose overview first; mechanisms cite `errors/` rules.
- 2026-11-23: first version, folded from `plan/domains/diagnostics/README.md` and `ARCHITECTURE.md`.
