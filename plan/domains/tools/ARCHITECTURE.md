# tools architecture

The owning lane keeps this current in the same commit as any structural change. It says: the modules and what each is for, the boundaries and invariants, the interfaces other lanes may use (and their consumers), and the known debt.

Read from the code at `4e2de3cd` (2026-11-22). Line counts from `wc -l`.

## Modules

### `crates/cpl-cli` (binary `cpl`, ~7,500 lines of `src/`)
| File | Lines | What it is for |
|---|---|---|
| `src/main.rs` | 35 | Entry: hands off to a pinned toolchain if one applies, else `cpl_cli::exec`. |
| `src/lib.rs` | 1,662 | `version_line`, `run`, `run_with_input`, `exec`; the top-level help text; the command `match` (`check`, `fix`, `run`, `judge`, `build`, `judges`, `submit`, `explain`, `lsp`, ...); `explain_file`; `run` and `build` pipelines; `fix` (applies `cpl_lint::apply_fixes`). Other crates' commands are dispatched through a `match` to their `cli::run` / `run` fns (`lib.rs:661-668`). |
| `src/check.rs` | 176 | `cpl check` (text and `--json`) and `cpl fix` (`apply_error_fixes`, then the lint fixes once no error is left); `errors_in`, which `checked_fix.rs` re-checks with |
| `src/unit_tests.rs` | 240 | `cpl test` on `.cp` files and directories: the inline `test "..."` blocks in the interpreter and, with `--cpp`, as sanitized C++ (a problem name goes to `cpl-contest` instead). |
| `src/toolchain.rs` | 866 | `cpl self`: installed toolchains, `cpl +<selector>`, pins, `settings.toml`, fetch, rollback, launcher handoff. |
| `src/report.rs` | 923 | `cpl report`: redacted bug reports and crash bundles; the panic hook. |
| `src/profile.rs` | 521 | `cpl profile`, `cpl cover`. |
| `src/interact.rs` | 301 | `run` of a file with an `@interactor`; `cpl judge`. |
| `src/laws.rs` | 92 | `@law` properties that `cpl test` checks. |
| `src/sha256.rs` | 107 | Hash for toolchain downloads. |
| `src/dap.rs`, `trace.rs`, `live.rs`, `replay.rs` | 1,178 / 205 / 196 / 250 | `cpl dap`, `run --trace`, live trace, `cpl replay`. Owned by **debugger**; live here until it has a crate. |
| `tests/` | 40 files | CLI goldens (`cli.rs`, `toolchain.rs`, `judges.rs`, `report.rs`, `profile.rs`) and many other lanes' end-to-end tests that need the binary (`differential.rs`, `diagnostics.rs`, `debugger.rs`, `corpus.rs`, `trap_parity.rs`, `vscode_sync.rs`, `lsp_parity.rs`, ...). |

### `crates/cpl-contest` (the contest loop, ~3,700 lines)
`new.rs` (`cpl new` scaffolding), `problem.rs` (find a problem's source, samples and `tests/` dir), `test.rs` (`cpl test`: samples and test dirs, `run_cases`), `watch.rs` (`cpl watch`), `bench.rs` (`cpl bench` harness), `corpus_bench.rs` and `zero_cost.rs` (corpus-wide and zero-cost benchmarks; numbers owned by **perf**). Each command is `pub fn run(args, stdout, stderr) -> u8`.

### `crates/cpl-doc` (`cpl doc`, ~1,850 lines)
`model.rs` (`extract` a `Module` of `Item`s and `Example`s from source), `markdown.rs`, `html.rs`, `highlight.rs` (renderers), `examples.rs` (doc examples as tests), `cli.rs` (`run`), `lib.rs` (`json` module: the doc JSON shape).

### `crates/cpl-json` (779 lines, zero dependencies)
`Value` (objects are `BTreeMap`, so output is deterministic), `parse`, `to_string`, `to_string_pretty`, `escape_string`, `write_script_safe_string` (safe inside `<script>`), `obj`.

### `crates/cpl-lsp` (`cpl lsp`, ~4,200 lines of `src/` plus 2,868 of unit tests)
`transport.rs` (stdio framing: `read_message`, `write_message`, `serve`), `server.rs` (`Server`: document store, dispatch, diagnostics, hover, definition, code actions, formatting, semantic tokens), `complete.rs` (completion, signature help, parameter hints), `outline.rs` (document symbols), `rename.rs` (references, rename), `build.rs` (generates the `use std.` names from `spec/std/`). Tests: `src/tests.rs`, `tests/{fixtures,fuzz,ports,latency}.rs`.

### `editors/`
`vscode/` (TypeScript client: `client.ts`, `completion.ts`, `inlayHints*.ts`, `lenses.ts`, `runner.ts`, `traceView.ts`, `languageData.ts` tables; TextMate grammar; snippets; tests in `test/`), `neovim/` (Lua, `test/smoke.lua`), `vim/`, `emacs/` (`smoke.el`), `helix/`, `zed/`, `jetbrains/` (Gradle, LSP4IJ), `tree-sitter-cp/`, `smoke.sh` (keyword parity with `crates/cpl-syntax/src/token.rs`, then each editor's smoke test when its binary exists).

## Dependencies
`cpl-cli` links every tool crate: check, codegen, contest, diag, doc, fmt, interp, judge, ir, json, lint, lsp, stress, syntax. `cpl-lsp` depends on check, diag, fmt, json, judge, lint, ir, syntax (not codegen). `cpl-contest` on check, codegen, diag, ir, json, judge, lint, stress, syntax. `cpl-doc` on diag, json, syntax. (From each `Cargo.toml`.)

LSP's view of the compiler (`grep -o 'cpl_[a-z]*::…' crates/cpl-lsp/src/*.rs`): `cpl_check::{Database, Analysis, Hover}` for types and hover, `cpl_syntax::{parse_file, ast, visit, KEYWORDS}` for structure, `cpl_lint::{proven_sizes_parsed, proven_ranges_parsed, proven_loops_parsed, Proven*}` for inlay hints, `cpl_diag::Span`.

## Interfaces other lanes use
| Interface | Where | Consumers |
|---|---|---|
| `cpl --help` and `cpl <cmd> --help` text | `crates/cpl-cli/src/lib.rs`, each crate's `run` | site (`web/scripts/gen-tools.mjs` → `content/generated/cli.json` → `web/lib/tools.ts`), docs, every user |
| Exit codes 0/1/2/3 and `--json` output | `spec/tools/cli.md`; each command | site, editors (VS Code `runner.ts`, `results.ts`), CI scripts, `cpl-wasm` parity (`crates/cpl-wasm/tests/parity.rs`) |
| `cpl_cli::{run, run_with_input, exec}` | `crates/cpl-cli/src/lib.rs` | `cpl-wasm` (playground), `crates/cpl-cli/tests/*` of all lanes |
| `cpl_lsp::Server` and transport | `crates/cpl-lsp/src/lib.rs` | `cpl-wasm` (`src/lib.rs`, `src/abi.rs`: playground editor), `cpl lsp`, every editor package |
| `cpl_json` | `crates/cpl-json/src/lib.rs` | cli, conform, contest, doc, interp, lsp, stress, wasm |
| `cpl_contest::problem` (samples, tests dir) | `crates/cpl-contest/src/problem.rs` | cli (`run`, `stress` defaults) |
| `cpl_doc::{extract, json}` | `crates/cpl-doc/src/lib.rs` | site (std reference), cli |
| VS Code language tables | `editors/vscode/src/languageData.ts` | VS Code completion; kept in sync by `crates/cpl-cli/tests/vscode_sync.rs` and `test/unit/languageData.test.ts` |

## Invariants
- `cpl-json` has no dependencies and writes objects in key order: the same value always prints the same bytes.
- Every command function has the shape `fn(args, stdout, stderr) -> u8`; nothing in a command calls `std::process::exit` (that is `main.rs`'s job), so `run` is testable and `cpl-wasm` can call it.
- Exit codes: 0 ok, 1 the program or a test failed, 2 compile error, 3 usage error (`spec/tools/cli.md`).
- Tools consume the compiler through its public crates (`cpl-check`, `cpl-syntax`, `cpl-diag`, `cpl-codegen`); they never re-implement a check. Diagnostic text is the diagnostics lane's, formatting the lint-fmt lane's.
- `cpl lsp` never shells out to `cpl`; it links the same crates (so the playground's in-browser LSP and the desktop one agree).
- Editor keyword lists match `crates/cpl-syntax/src/token.rs` (`editors/smoke.sh`).

## Known debt
1. **Spec and binary disagree on the command set**: 14 mismatches (`measure.py spec-sync`). Spec-only: `debug`, `fetch`, `interact`, `locate`, `migrate`, `selftest`, `trace`; binary-only: `check`, `dap`, `fix`, `judge`, `replay`, `report`, `self`. Nothing checks it. [Q tools-1]
2. **No single command table**: the help text in `lib.rs`, the dispatch `match`, and each crate's own flag parsing are separate; flags, `--json` and exit codes vary by command (18 of 23 commands offer JSON in `--help`, `measure.py json`). `cli.json` is scraped from help text by `web/scripts/gen-tools.mjs` instead of emitted.
3. **`crates/cpl-cli/src/lib.rs` is 2,552 lines** (cap 2,000, allowlisted in `scripts/debt-ratchet.json`), as are `crates/cpl-lsp/src/server.rs` (2,111) and `crates/cpl-lsp/src/tests.rs` (2,868).
4. **`cpl explain` is a stub of its spec**: `explain_file` (`lib.rs:2052`) prints only `cpl_codegen::explain_closures` and `explain_dropped_fields`; `spec/tools/explain.md`'s sizes, niches, peak memory, specializations and checks are missing, and hover computes its own (`cpl_check::Hover`). No representation query exists in the shared core yet.
5. **LSP over budget at 5,000 lines**: `tests/latency.rs` fails in release (check 67 ms vs 50; open-to-diagnostics 161 ms vs 100). The fast gate runs it in a debug build, which enforces the budget only at 500 lines (`latency.rs:96`), so nothing stopped the 5,000-line regression.
6. **Debugger code in `cpl-cli`**: `dap.rs`, `trace.rs`, `live.rs`, `replay.rs` (1,829 lines) belong to the debugger lane.
7. **Other lanes' tests live in `crates/cpl-cli/tests/`** because they need the binary; ownership of a failure there is unclear.
8. **Editor coverage is mostly unverified here**: on this VM `editors/smoke.sh` skips Neovim, Emacs, Helix, Zed, tree-sitter (binaries missing) and the JetBrains build fails resolving Gradle artifacts (network). VS Code unit tests: 36/37, a stale `PRELUDE_VALUES` table (`skip_while`, `reach_sets`), fixed on `devin/wip-editor-lsp`.
