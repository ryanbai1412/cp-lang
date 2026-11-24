# Tools

| | |
|---|---|
| Kind | Domain charter |
| Owner | tools |
| Status | Active |
| Describes | `crates/cpl-cli`, `crates/cpl-contest`, `crates/cpl-lsp`, `crates/cpl-doc`, `crates/cpl-json`, `editors/`, `spec/tools/cli.md`, `spec/tools/bundle.md`, `spec/tools/explain.md` |

The tools domain owns everything a contestant touches to drive CP Lang:
the `cpl` command, the language server behind every editor, the editor
packages themselves, and the contest workflow that turns a problem page into
a folder of tests. If you have used CP Lang from a terminal or an editor,
you have used this domain's work.

## What good looks like

A contestant who has never used CP Lang should be able to install it, open
a problem in their own editor, and get to an accepted submission without
reading anything except `cpl --help`.

Concretely: on a Codeforces problem page they click Competitive Companion
and get `a.cp` with the samples beside it. They type the solution with
completion and hover that never lie, run `cpl test a` or press the editor's
test button, and paste the single C++17 file from `cpl b -c` into the judge.
Every step answers within a fraction of a second, and works the same on
Linux, macOS and Windows, in Neovim, VS Code and JetBrains.

## How it fits together

There is one program, `cpl`, and every surface is a thin client of it.

- **Commands** (`check`, `run`, `test`, `build`, `fix`, `explain` and
  the rest) are plain functions that take arguments and write output. They
  never exit the process, so tests and the browser can call them directly.
- **The language server** (`cpl lsp`) gives every editor the same
  diagnostics, hover, completion, navigation and fixes. The browser
  playground runs the same server compiled to WebAssembly, so the website
  and your editor can't disagree.
- **Editor packages** add only what an editor needs: starting the server,
  syntax highlighting and a test panel. The intelligence lives in the server.
- **None of it reimplements the compiler.** Commands and the server ask the
  compiler's shared, incremental analysis for answers, so what an editor
  shows is exactly what `cpl check` reports.

## How we know it is good

Each command has snapshot tests of its help, output, errors and exit codes,
checked against the CLI spec. The language server has fixture tests that
place a cursor in a program and assert the answer, plus fuzzing and latency
budgets. Parity tests check that the editor, the browser and the command
line give the same answers. Each editor runs a smoke test in CI on all
three operating systems.

The weakest areas today are completion (we check that it answers, not that
it offers the right thing first), hover coverage of every token, and making
hover, signature help, completion and docs come from one description of a
function. Those are first in line. The full list of qualities, with the
check behind each, is the quality register at the end of this document.

## Where to start

- **Adding or changing a command:** update `spec/tools/cli.md`, then the
  command in `crates/cpl-cli`, then its snapshot in the golden suite.
- **Adding an editor feature:** implement it once in `crates/cpl-lsp` with
  a fixture test, and add a parity case so the browser gets it too.
- **Fixing an editor package:** `editors/<editor>/`, and run
  `editors/smoke.sh`.

---

# Reference

The rest of this document is for looking things up: the exact scope, the
interfaces other domains rely on, every quality we track and the suites
that check them.

## Scope

**Owns:**

- The `cpl` binary: argument parsing, `--help`, exit codes (0 ok, 1 errors
  in the source, 2 usage, 3 the program or a test failed), `--json`, and the
  commands `check`, `fix`, `run`, `build`, `submit`, `explain`, `doc`,
  `new`, `fetch`, `test`, `watch`, `bench` (the harness), `profile`,
  `cover`, `report`, `self`, `setup`, `lsp`, `migrate`, `locate`.
- The contest loop in `crates/cpl-contest`: problem and case discovery,
  Competitive Companion import, contest folders, `cpl test`.
- The language server `crates/cpl-lsp` (`cpl lsp`), which the desktop
  editors and the browser playground both run.
- Every editor package under `editors/`: VS Code, JetBrains, Neovim, Vim,
  Zed, Helix, Emacs, the tree-sitter grammar and `editors/smoke.sh`.
- `cpl doc` (`crates/cpl-doc`) and the zero-dependency JSON crate
  (`crates/cpl-json`).

**Doesn't own:**

- What a diagnostic says, or an error's fix (diagnostics); lint rules and
  the formatter (lint-fmt). Tools shows them unchanged.
- Types, facts and the analysis itself (compiler). Tools never
  re-implements a check.
- `cpl stress`, traces, `cpl dap` and the debugger, including
  `crates/cpl-cli/src/{dap,trace,live,replay}.rs` (debugger).
- C++ size and runtime numbers (perf), judge profiles' contents (idioms),
  the playground page and the site (site).

## Interfaces

**Provides:**

| Interface | Where | Used by |
|---|---|---|
| `cpl --help`, `cpl <command> --help`, `cpl --help --json` | `crates/cpl-cli/src/lib.rs` and each command's `run` | site (`cli.json`), docs, every user |
| Exit codes and `--json` output | `spec/tools/cli.md` | editors, site, CI scripts |
| `cpl_cli::{run, run_with_input, exec}`: every command is `fn(args, stdout, stderr) -> u8` and never exits the process | `crates/cpl-cli/src/lib.rs` | `crates/cpl-wasm`, end-to-end tests of every domain |
| `cpl_lsp::Server`, its transport, and `cpl_lsp::fix::document` (the one fix-all path) | `crates/cpl-lsp/src/` | `cpl lsp`, `crates/cpl-wasm`, every editor package |
| `cpl_contest::problem` (`find_tests_dir`, `find_checker`, samples) | `crates/cpl-contest/src/problem.rs` | `cpl run`, `cpl stress`, `cpl test`, the editors' test panels |
| `cpl_doc::{extract, json}` | `crates/cpl-doc/src/lib.rs` | site's std reference |
| `cpl_json` (objects print in key order) | `crates/cpl-json/src/lib.rs` | most crates |

**Needs:**

| From | What |
|---|---|
| compiler | One analysis session per command (`cpl_check::Session`), `Analysis`, the representation query behind `cpl explain` and hover |
| diagnostics | Diagnostic text and the error fixes `cpl fix` and quick fixes apply (`cpl_ir::fix`) |
| lint-fmt | `cpl_lint::lint_analysis`, `cpl_lint::fix_in`, the formatter |
| stdlib | Doc comments with signature, named options, cost and examples for every `pub` item |
| debugger | `cpl run --trace`, `cpl stress` and `cpl dap`, which the editors drive |
| infra | CI jobs on Linux, macOS and Windows; release channels and installers |

## Quality register

Each row is a quality a user of tools could feel (a contestant, a learner,
an editor user, a library author, a contributor), the metric that says how
high it is, and the strongest mechanism that enforces it. Mechanisms,
strongest first: an architecture that makes the failure impossible; a
generated, fuzz, property, differential or metamorphic suite; a hand-written
suite; a lint, ratchet or gate; a process where no mechanism can decide.
`missing` means no real check exists yet. Rows are grouped by the user
journeys in `handbook/product/journeys.md`. Targets and today's numbers are
in `plan/domains/tools/metrics.toml`. A rule a row enforces is stated once,
in `handbook/guidelines/commands.md`, and the row cites it.

Where mature projects already measure a quality, the row follows them:
rust-analyzer's fixture tests with cursor markers and its whole-repository
analysis runs, Cargo's per-command snapshot suite, rustc's UI tests for
what a command prints, and fuzzing of every request as LLVM and GCC fuzz
their front ends.

### Trying it and installing it (J1, J3)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| The playground means what `cpl` means | entry points with a parity test (check, lint, fix, run, build, trace, explain, stress) | differential on every conformance case, `spec.cp` port and example; rule `commands/R9` | `crates/cpl-wasm/tests/parity.rs` |
| One step installs the editor support | editors whose `cpl setup <editor>` is tested | hand-written suite | `crates/cpl-cli/tests/golden.rs` (setup); a fresh-machine run per OS: `missing` |
| Each editor works from that step | editors passing the shared smoke (highlighting, diagnostics, completion, formatting, definition) | the same smoke on each editor | CI `neovim`, `vscode`, `jetbrains` and `editors` jobs; Vim in `editors/smoke.sh` |
| A failed install or setup says why and what to do | failure modes with a tested message (no network, no compiler, unwritable folder, unknown editor) | hand-written suite | `crates/cpl-cli/tests/golden.rs` (`setup_failures_say_why_and_what_to_do`, `setup_writes_the_editor_files`: no network, no curl, unwritable folder, unknown editor, each with its `help:`); no C++ compiler: `missing` |
| Toolchains, pins and channels behave as documented | `cpl self` paths tested, including offline | hand-written suite | `crates/cpl-cli/tests/toolchain.rs` |
| Linux, macOS and Windows behave alike (paths, CRLF input, clipboard) | CLI suite passing per OS | the same suite on each OS | CI `cli` job (three OSes); CRLF sample and input files: `missing` as a dedicated case |
| Everything but `fetch` and `self update` works offline | commands that make no network call | the contest flow with `curl`, `wget` and `code` stubbed and proxies closed | `crates/cpl-cli/tests/offline.rs` (Unix: 14 commands, `new` through `submit`, 0 network calls; `fetch` reaching the stub is the control) |

### Starting problems and contests (J4)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Companion import lands title, URL, limits, judge and samples | fields kept from a Companion problem | hand-written suite | `crates/cpl-contest/src/tests.rs`, `crates/cpl-cli/tests/golden.rs` (`new_creates_the_solution_and_its_tests_folder`) |
| A whole contest imports as one folder | problems and samples kept from a contest page | hand-written suite | `crates/cpl-cli/tests/golden.rs` (`fetch_writes_a_pages_samples_and_keeps_other_tests`) |
| Editors import like the CLI | editors with an import action calling `cpl new --listen` | hand-written suite | `editors/vscode/test/unit/`, `editors/jetbrains/src/test/kotlin/dev/cplang/ide/CplTest.kt` |
| The user's own layout and template work | layouts `cpl test`, `run` and the editors find the cases in (`a.tests/`, sibling `tests/`) | differential against `crates/cpl-contest` (`find_tests_dir`); rule `commands/R15` | `crates/cpl-contest/src/tests.rs`, `editors/jetbrains/src/test/kotlin/dev/cplang/ide/CplTest.kt`; VS Code test panel and lenses: `missing` |

### Writing and revising in the editor (J2, J5)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| The editor reports what `cpl check` and `cpl lint` report | files where they differ | differential over every conformance case, `spec.cp` port and example; rule `commands/R10` | `crates/cpl-cli/tests/lsp_parity.rs` |
| Diagnostics after a sequence of edits equal a fresh open | edit sequences where they differ | metamorphic: replay real edits, compare with opening the final text | `crates/cpl-lsp/tests/edit_replay.rs` |
| The server never fails internally on half-typed code | internal errors | fuzz: real ports cut, spliced and garbled, then every request; rule `commands/R12` | `crates/cpl-lsp/tests/fuzz.rs` |
| Answers arrive within the latency budget | p95 per request at 500 and 5,000 lines, per OS | generated large files, release build; rule `commands/R13` | `crates/cpl-lsp/tests/latency.rs`; CI `lsp latency` job |
| Completion offers what the contestant types next | recall: share of identifiers, replayed token by token over the corpus, offered after `.`, after `(` and in scope, including after a call such as `runs(xs).` | generated replay; rule `commands/R14` | `crates/cpl-lsp/tests/completion_replay.rs` |
| Completion ranks the right item first | mean rank of the identifier actually typed | generated replay; rule `commands/R14` | `crates/cpl-lsp/tests/completion_replay.rs` (mean rank per kind, ceilings that only fall) |
| Hover explains every token | std items and syntax forms whose hover shows the signature with named options (`by:`), the cost, and what syntax like `[0..n]Int` means | every `pub` std item and `BUILTIN_MEMBERS` row used in a program, floors that only rise; every corpus token for syntax forms; rules `commands/R11`, `commands/R14` | `crates/cpl-lsp/tests/std_items.rs` (std items, built-in members), `crates/cpl-lsp/tests/hover_tokens.rs` (every corpus name and type-syntax token) |
| Hover, signature help, completion and docs agree | surfaces rendering a signature from one model | every std item and built-in member checked on all three surfaces; architecture: one signature model | `crates/cpl-lsp/tests/std_items.rs` (parameter names agree; one model still `missing`) |
| Signature help shows the active named option | calls with named options where the right one is highlighted | generated over std signatures | `crates/cpl-lsp/tests/std_items.rs` (each named option of every std item and built-in member, written first) |
| Go to definition reaches every name, in the browser too | std and user names resolving to a definition and a docs page | generated over the corpus, desktop and playground; rule `commands/R14` | `crates/cpl-lsp/tests/ports.rs` (first uses of a name in the hill-climb ports that resolve: 65.0%, a floor that only rises); docs pages and the playground: `missing` (`crates/cpl-wasm/tests/parity.rs` has one case) |
| References and rename keep the program's meaning | renames after which the file checks clean and runs the same | metamorphic over the corpus | `crates/cpl-lsp/src/tests.rs` (hand-written); `crates/cpl-cli/tests/rename_replay.rs`: every 4th hill-climb port (all in release) has each user name renamed, then checks clean and prints the same (402 renames; 21 known misses, shrink-only) |
| Every lint and error fix is a code action, and fix-all equals `cpl fix` | fixes offered as code actions; fix cases matching `cpl fix` | differential plus fuzz; rule `commands/R10` | `crates/cpl-wasm/tests/parity.rs` (`fix_all_matches_cpl_fix`), `crates/cpl-lsp/tests/fuzz.rs`; one action per fix, generated: `missing` |
| Formatting in the editor equals `cpl fmt` | files where they differ | differential over every conformance case; rule `commands/R10` | `crates/cpl-lsp/tests/format_parity.rs` |
| Highlighting agrees with the parser | tokens the grammars classify differently from `cpl-syntax` | generated diff | `crates/cpl-cli/tests/vscode_sync.rs`, `editors/smoke.sh` (keywords), `editors/vscode/test/grammar/`; tree-sitter and TextMate against the parser on the corpus: `missing` |
| Inlay hints are right and can be turned off | hints matching the checker's types and ranges | hand-written suite | `editors/vscode/test/unit/inlayHints.test.ts` |
| `cpl explain` and hover agree on storage | ports where they state the same bytes | differential; rule `commands/R10` | `crates/cpl-lsp/tests/explain_hover.rs` |

### Running and testing (J6)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| `cpl run` with no input runs every sample and shows the diff | sample layouts covered | hand-written suite | `crates/cpl-cli/tests/cli.rs` |
| Local verdicts match the judge's | token pairs every caller accepts and rejects alike | one comparator (`cpl_stress::judge`) for `cpl test`, `stress`, `submit` and `watch`; rule `commands/R10` | an agreement test across `cpl test`, stress, the corpus bench and `cpl judge`: `missing` (the corpus bench still has its own tolerance formula) |
| Multi-answer problems are judged by their checker | problems with `@check` or `checker.py` judged by it | hand-written suite | `crates/cpl-contest/src/tests.rs` |
| Interactive problems run with correct flushing | interactive problems run end to end | hand-written suite | `crates/cpl-cli/tests/cli.rs`, `crates/cpl-contest/src/tests.rs`; every interactive corpus problem: `missing` |
| Adding a custom test is one step | `cpl test --add` paths and editor Add Test | hand-written suite | `crates/cpl-contest/src/tests.rs`, `editors/vscode/test/unit/` |
| Editor test panels show what `cpl test` shows | panels whose cases, order and verdicts match | differential; rule `commands/R15` | `editors/vscode/test/unit/results.test.ts`, `editors/jetbrains/src/test/kotlin/dev/cplang/ide/CplTest.kt` |
| `cpl watch` reruns on save and never shows stale results | saves after which the shown result is current | hand-written suite | `crates/cpl-cli/tests/golden.rs` (watch) |

### Submitting (J9)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| `cpl b -c` puts the judge's file on the clipboard on every OS | OSes with a tested copy | the same suite on each OS | CI `cli` job; a real clipboard round trip per OS: `missing` |
| Submission problems are caught before pasting (size, failed samples, checks) | `submit` refusals tested | hand-written suite | `crates/cpl-cli/tests/golden.rs` |

### Every command (all journeys)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Every command is discoverable from `cpl --help` | commands with `--help`, help lines within 80 columns | generated: one case per command listed by `cpl --help`; rule `commands/R1` | `crates/cpl-cli/tests/golden.rs` (`every_command_has_help_and_rejects_an_unknown_option`, `every_help_fits_80_columns`) |
| The spec and the binary name the same commands | mismatches | differential, with a shrink-only allowlist; rule `commands/R4` | `crates/cpl-cli/tests/spec_sync.rs` |
| A usage error names the right invocation | commands with a usage-error test | generated over the command list; rule `commands/R1` | `crates/cpl-cli/tests/golden.rs` |
| Every command's output is pinned | commands with a snapshot of text and `--json` output | hand-written snapshots | `crates/cpl-cli/tests/golden.rs`; snapshots for every command and outcome: `missing` |
| One `--json` contract | commands with `--json`; one flag spelling; outputs valid against a published schema | hand-written suite; rule `commands/R3` | `crates/cpl-cli/tests/golden.rs` (`json_and_format_json_are_one_flag_on_every_command_with_json`); schema validation: `missing` |
| Exit codes mean the same on every command | commands asserting success and failure codes | hand-written suite; rule `commands/R5` | `crates/cpl-cli/tests/golden.rs` |
| Every command starts in under 100 ms | median and p90 per command, release build | generated over the command list and the largest corpus ports; rule `commands/R7` | `crates/cpl-cli/tests/cold_start.rs`; CI `lsp latency` job on macOS and Windows |
| The same input prints the same bytes | `cpl-json` output order | architecture: objects are a `BTreeMap`; rule `commands/R6` | `crates/cpl-json` unit tests |
| Paths shown are ones the user can open | printed paths going through one helper | architecture: `cpl_diag::Shown`; rule `commands/R8` | `crates/cpl-cli/tests/golden.rs` |
| Help, errors, hover and explain use the spec's words | spellings `spec/GLOSSARY.md`'s Never column bans, in prose | generated scan: every `--help`, every `cpl explain` entry, `cpl explain` and hover over 200 conformance files | `scripts/tool-vocab.py` (in `check.sh`); error messages over the corpus: `missing` |
| Names are guessable | wrong guesses on fixed tasks given only `cpl --help` | process: a cold run on fixed tasks | `missing` as a suite |

### Long-term use and contributing (J10)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Old solutions keep building, and `cpl migrate` moves them | removed spellings rewritten under a folder | hand-written suite | `crates/cpl-cli/tests/golden.rs` (`migrate_rewrites_removed_spellings_under_a_folder`) |
| Personal libraries get completion, hover and definition like std | user-library names covered by the completion, hover and definition metrics | generated, as for std | `missing` |
| Editor settings are few and documented | settings each editor exposes that are documented and tested | hand-written suite | `crates/cpl-cli/tests/vscode_sync.rs` (VS Code: 12 settings, each read, documented in its README and defaulted alike, at most 15); other editors: `missing` |
| One analysis per command | databases each command opens (`own_databases`) | ratchet now; architecture once every tool takes one `cpl_check::Session`; rule `commands/R11` | `scripts/debt-ratchet.py` |
| No tools file is too large to change safely | files over the 2,000-line cap | ratchet | `scripts/debt-ratchet.py` |

## Suites and tools

| Run | What it catches |
|---|---|
| `cargo nextest run -p cpl-cli --test golden` | a command without help, a usage error, JSON or exit-code coverage |
| `cargo test --release -p cpl-cli --test cold_start -- --nocapture` | a command slower than 100 ms to start |
| `cargo nextest run -p cpl-cli --test spec_sync` | a command the spec and the binary disagree on |
| `cargo nextest run -p cpl-contest` | case discovery, checkers, contest folders, Companion import |
| `cargo nextest run -p cpl-lsp` | LSP fixtures, fuzz, ports |
| `cargo test --release -p cpl-lsp --test latency -- --nocapture` | an LSP request over budget |
| `cargo nextest run -p cpl-cli --test lsp_parity` and `cargo nextest run -p cpl-wasm --test parity` | the editor or playground disagreeing with `cpl` |
| `cd editors/vscode && npm test` | VS Code unit and grammar tests |
| `cd editors/jetbrains && ./gradlew test` | JetBrains unit tests |
| `CPL=target/release/cpl editors/smoke.sh` | each installed editor's smoke test, and keyword parity |
| `python3 plan/domains/tools/measure.py golden` (also `help`, `json`, `spec-sync`, `corpus`) | the numbers in `plan/domains/tools/metrics.toml` |

## Judgment rubric
A cold agent scores each 1-5 with a reason, on fixed samples; the trend is the signal.
1. **Guessable**: could you have typed this command and flag without reading the help? Probe: given only `cpl --help`, do 8 fixed tasks (run sample 2 only, build for Codeforces and copy, see why line 12 is slow, find what no test reaches, ...); count wrong guesses and help lookups.
2. **Usage errors at 3am**: does each usage error name the right invocation? Probe: 12 fixed wrong invocations; score each message.
3. **Explain answers the question**: can you say why a value takes its bytes, or why a call is O(n), from `cpl explain` or hover alone? Probe: 3 questions on each of 5 ports, answers checked against the emitted C++.
4. **Editor in 5 minutes**: from `editors/README.md` alone, get hover, completion, rename and a quick fix working. Probe: cold agent on a fresh VM with VS Code and Neovim; time and steps.
5. **One vocabulary**: do help, errors, hover and explain use the spec's words (`spec/GLOSSARY.md`)? Probe: list every term in help and hover texts that the glossary does not define.

## Changes

- 2026-11-23: first version, folding in the stable parts of the tools
  lane's old README and architecture notes.
- 2026-11-23: the judgment rubric moved here from the lane README.
