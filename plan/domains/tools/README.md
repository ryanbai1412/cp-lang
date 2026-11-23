# tools

Segment: C. Design, then build. One page; detail in [PLAN.md](PLAN.md), the code in [ARCHITECTURE.md](ARCHITECTURE.md), the v0.3 experience in [design/overview.md](design/overview.md), numbers in [metrics.toml](metrics.toml), history in [worklog.md](worklog.md).
Owns: `crates/cpl-cli`, `crates/cpl-contest`, `crates/cpl-doc`, `crates/cpl-json`, `crates/cpl-lsp`, `editors/`, `spec/tools/{cli,bundle,explain}.md`.
Status (2026-11-22): every command exists and has `--help`; the spec and the binary disagree on 14 command names; `cpl explain` shows only closure lowering; LSP fast at contest sizes but over budget at 5,000 lines; editor work from last round is unlanded (`devin/wip-editor-lsp`, 5 commits).

## Mission
Every `cpl` command and every editor answers the contestant fast, does what its name says, and tells the truth about the program: one vocabulary, one JSON contract, all built on the compiler's query API.

## Covers
- **CLI**: `check`, `fix`, `run`, `judge`, `build`, `judges`, `submit`, `explain`, `doc`, `new`, `test`, `bench` (harness), `profile`, `cover`, `watch`, `report`, `self` (toolchains, pins, `settings.toml`), argument parsing, help, exit codes, `--json`.
- **Contracts**: `cpl --help` (the site's `cli.json` is generated from it), JSON output, `spec/tools/{cli,bundle,explain}.md`.
- **Editors**: `cpl lsp` and the packages in `editors/` (VS Code, Neovim/Vim, JetBrains, Zed, Helix, Emacs, tree-sitter grammar).

Not covered (owner): `stress`, `run --trace`, `dap`, `replay`, trace viewer (debugger; the code still sits in `cpl-cli`, see ARCHITECTURE debt); `fmt`, `lint` (lint-fmt); C++ size and speed, `bench` numbers (perf); diagnostic text and codes (diagnostics); judge profiles, corpus (idioms); installers, release, gate (infra); playground and docs pages (site).

## Success criteria (v0.3)
1. `spec/tools/cli.md` and `cpl --help` name the same commands, checked by a test in the fast gate (0 mismatches).
2. Every command has a golden test of help, one success, one usage error and its exit code (0/1/2/3 as the spec says).
3. Every command that produces results takes `--json`, with one shape documented in `spec/tools/cli.md`.
4. `cpl explain` and LSP hover show the spec's representation sections (types, niches, arrays and peak memory, specializations) from one computation.
5. Every corpus port that checks builds to one C++17 file under 32 KB.
6. LSP p90 under budget (check 50 ms, diagnostics/hover/completion 100 ms) at 500 and 5,000 lines in release, enforced by the gate (today the gate's debug run enforces 500 lines only).
7. Tier-1 editors install from the repo and pass a smoke test against the real `cpl lsp` in the gate.

## Milestones
- **M1 Honest contract** (first): `devin/wip-editor-lsp` landed; spec/CLI sync test green with 0 mismatches; golden tests for all 23 commands; `cli.json` regenerated in the site build with 0 commands missing help; `cpl-cli/src/lib.rs` under the 2,000-line cap.
- **M2 Explain is the truth**: `cpl explain <file> --json` emits the four sections above for every corpus port that builds; LSP hover reads the same data (a test compares them on 20 ports); `--json` on every result-producing command.
- **M3 Editors shipped**: latency test green at 5,000 lines and run in the gate; tier-1 smoke tests in the gate; VS Code `.vsix` built per release; JetBrains plugin built in CI.

## Metrics
All measured 2026-11-22 on this VM (2 cores), release build at `4e2de3cd`.

| Metric | Command | Today | Target | M |
|---|---|---|---|---|
| spec/CLI command mismatches | `measure.py spec-sync` | 1 (spec-only: locate, to build) | 0 | M1 |
| commands with `--help` | `measure.py help` | 25/25 | 25/25 | M1 |
| end-to-end tests per command | `measure.py golden` | 25/25 with help, usage error, success and exit tested (`judges` has no failing exit) | each >= 4 | M1 |
| commands offering `--json` | `measure.py json` | 21/25 (not: dap, lsp, self, watch) | all that print results (all but dap, lsp, self, watch) | M2 |
| `cpl check` wall time, corpus p95 | `measure.py corpus` | 84.5 ms (p50 29.8, max 141.5; 877 ports, 498 check clean) | < 500 ms (spec); hold < 100 ms | - |
| largest corpus C++ | `measure.py corpus` | 48,686 B (cf2255f); 5 of 497 over 32 KB | 0 over 32 KB | M2 |
| LSP latency, 500 lines (median) | `cargo test --release -p cpl-lsp --test latency -- --nocapture` | check 6 ms; diagnostics 8 ms; hover 3 ms; completion 1 ms | < 50 / 100 ms | M3 |
| LSP latency, 5,000 lines (median) | same | check 53 ms, open-to-diagnostics 37 ms (p90 53; test fails on check, next hot spots filed with compiler) | < 50 / 100 ms | M3 |
| LSP unit tests | `cargo test --release -p cpl-lsp --lib` | 77/77 | all pass | - |
| VS Code unit + grammar tests | `cd editors/vscode && npm test` | 37/37 | all pass | M1 |
| editor smoke tests passing (this VM) | `CPL=target/release/cpl editors/smoke.sh` | see metrics.toml | tier 1 all, in gate | M3 |
| tools files over the 2,000-line cap | `python3 scripts/debt-ratchet.py --check` | 0 (cli lib.rs 1,422; lsp server.rs 1,966; lsp tests.rs 2,000) | 0 | M1 |
| unlanded commits on `devin/wip-editor-lsp` | `git cherry origin/main origin/devin/wip-editor-lsp` | 0 | 0 | M1 |

`measure.py` is `plan/domains/tools/measure.py`. Not measured yet: `explain` sections per port (M2's test).

## Judgment rubric
A cold agent scores each 1-5 with a reason, on fixed samples; the trend is the signal.
1. **Guessable**: could you have typed this command and flag without reading the help? Probe: given only `cpl --help`, do 8 fixed tasks (run sample 2 only, build for Codeforces and copy, see why line 12 is slow, find what no test reaches, ...); count wrong guesses and help lookups.
2. **Usage errors at 3am**: does each usage error name the right invocation? Probe: 12 fixed wrong invocations; score each message.
3. **Explain answers the question**: can you say why a value takes its bytes, or why a call is O(n), from `cpl explain` or hover alone? Probe: 3 questions on each of 5 ports, answers checked against the emitted C++.
4. **Editor in 5 minutes**: from `editors/README.md` alone, get hover, completion, rename and a quick fix working. Probe: cold agent on a fresh VM with VS Code and Neovim; time and steps.
5. **One vocabulary**: do help, errors, hover and explain use the spec's words (`spec/GLOSSARY.md`)? Probe: list every term in help and hover texts that the glossary does not define.

## Improve (highest impact first)
1. Land `devin/wip-editor-lsp` (v0.3 `use std.` completion, corpus latency bench, VS Code/Vim smoke, fixes the stale prelude table).
2. Reconcile `spec/tools/cli.md` with the binary and gate it ([Q tools-1](../../QUESTIONS.md)).
3. One argument parser and one output layer for every command (`--json`, exit codes, `--help` from the same table); split `cpl-cli/src/lib.rs` by command.
4. `cpl explain` to the spec, sharing its computation with LSP hover (needs a representation query from compiler/perf: request filed).
5. LSP at 5,000 lines: incremental diagnostics on open; enforce the release-build latency test in the gate.
6. Bundle under 32 KB everywhere: size gate at 32 KB with `--size` attribution (the bytes are perf's).
7. Tier-1 editors in the gate; JetBrains in CI (network blocks it on VMs).
8. Move debugger code (`dap.rs`, `trace.rs`, `live.rs`, `replay.rs`) into the debugger's `cpl-trace` when it splits out.

## Decisions for Ryan
Each is in [plan/QUESTIONS.md](../../QUESTIONS.md) with options and a reversible default.
1. **tools-1 command set**: the spec's 23 commands vs the binary's 23. Recommend: spec adopts `check`, `fix`, `judge`, `self`, `report`, `replay`, `dap`; `trace` stays `run --trace`, `debug` stays `dap` + editor, `interact` stays `run` on a file with an `@interactor`, `selftest` goes (it's the gate); build `fetch`, `locate`, `migrate` for v0.3.
2. **tools-2 tier-1 editors**: recommend VS Code and Neovim tier 1 (gated); JetBrains, Zed, Helix, Emacs tier 2 (smoke when installed, fixed on report).
3. **tools-3 explain scope for v0.3**: recommend the four sections contestants ask about (types and niches, arrays and peak memory, specializations, remaining checks) in both explain and hover; contexts, generic instances and unsafe-tier sites after v0.3.

## Ryan's role
One design review: the command set and names, the toolchain model (`cpl self`, `cpl +edge`, pins), tier-1 editors, explain scope (the three decisions above, plus [design/overview.md](design/overview.md)). After that the acceptance tests are the contract and the lane runs as a metric lane; Ryan sees milestone demos (a recording of the editor flow, a sample `explain`).
