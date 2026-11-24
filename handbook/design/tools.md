# Tools design

| | |
|---|---|
| Kind | Design |
| Owner | tools |
| Status | Active |
| Describes | `crates/cpl-cli`, `crates/cpl-contest`, `crates/cpl-doc`, `crates/cpl-json`, `crates/cpl-lsp`, `editors/` |

Every `cpl` command is a function from arguments to output and an exit
code, built on the compiler's public crates. Nothing in a command exits the
process or re-implements a check, which is what lets the playground call
the same functions in the browser and the tests call them in-process.
The editor side of this (the language server and the playground engine) has
its own design doc, [editor.md](editor.md).

Example: `cpl test a` enters `cpl_cli::exec`, which dispatches to
`cpl_contest::test::run(args, stdout, stderr)`. That finds `a.cp` and its
samples through `cpl_contest::problem`, analyses the file with `cpl_check`,
runs each case and returns 3 if one fails. `main.rs` turns the 3 into the
process's exit status; `cpl-wasm` returns it to the page instead.

## What good looks like

- **One shape per command.** `fn(args, stdout, stderr) -> u8`, testable in
  process and callable from WebAssembly.
- **Facts from the compiler.** Types, names, ranges and costs come from
  `cpl_check`; diagnostic text is the diagnostics lane's, layout the
  formatter's.
- **Deterministic output.** The same input prints the same bytes, JSON
  included.

## How it fits together

`cpl-cli` owns the binary, the top-level help, dispatch, and the commands
that don't belong to a lane-specific crate. `cpl-contest` is the contest
loop (`new`, `test`, `watch`, `bench`). `cpl-doc` turns std and user
sources into reference pages. `cpl-json` is the dependency-free JSON every
`--json` flag writes. `cpl-lsp` is the language server, described in
[editor.md](editor.md). The exact module map, dependencies and invariants
are below.

## Where to start

Read `crates/cpl-cli/src/lib.rs` `exec` for the dispatch, then one command
end to end (`crates/cpl-contest/src/test.rs`). The rules every command
follows are in
[handbook/guidelines/commands.md](../guidelines/commands.md).

# Reference

## Crates and modules

### `crates/cpl-cli` (binary `cpl`)
| File | What it is for |
|---|---|
| `src/main.rs` || Entry: hands off to a pinned toolchain if one applies, else `cpl_cli::exec`. |
| `src/lib.rs` || `version_line`, `run`, `run_with_input`, `exec`; the top-level help text; the command `match` (`check`, `fix`, `run`, `judge`, `build`, `judges`, `submit`, `explain`, `lsp`, ...); `explain_file`; the `run` pipeline; `fix` (applies `cpl_lint::apply_fixes`). Other crates' commands are dispatched through a `match` to their `cli::run` / `run` fns (`lib.rs:661-668`). |
| `src/build.rs` || `cpl build` (`BuildOptions`, `build_file`, text and `--json`), `--checked` (the C++ built here with the sanitizers on) and `take_judge`; `submit.rs` builds through it |
| `src/check.rs` || `cpl check` (text and `--json`) and `cpl fix` (`apply_error_fixes`, then the lint fixes once no error is left); `errors_in`, which `checked_fix.rs` re-checks with |
| `src/unit_tests.rs` || `cpl test` on `.cp` files and directories: the inline `test "..."` blocks in the interpreter and, with `--cpp`, as sanitized C++ (a problem name goes to `cpl-contest` instead). |
| `src/toolchain.rs` || `cpl self`: installed toolchains, `cpl +<selector>`, pins, `settings.toml`, fetch, rollback, launcher handoff. |
| `src/report.rs` || `cpl report`: redacted bug reports and crash bundles; the panic hook. |
| `src/profile.rs` || `cpl profile`, `cpl cover`. |
| `src/interact.rs` || `run` of a file with an `@interactor`; `cpl judge`. |
| `src/laws.rs` || `@law` properties that `cpl test` checks. |
| `src/sha256.rs` || Hash for toolchain downloads. |
| `src/dap.rs`, `trace.rs`, `live.rs`, `replay.rs` || `cpl dap`, `run --trace`, live trace, `cpl replay`. Owned by **debugger**; live here until it has a crate. |
| `tests/` | 40 files | CLI goldens (`cli.rs`, `toolchain.rs`, `judges.rs`, `report.rs`, `profile.rs`) and many other lanes' end-to-end tests that need the binary (`differential.rs`, `diagnostics.rs`, `debugger.rs`, `corpus.rs`, `trap_parity.rs`, `vscode_sync.rs`, `lsp_parity.rs`, ...). |

### `crates/cpl-contest` (the contest loop)
`new.rs` (`cpl new` scaffolding), `problem.rs` (find a problem's source, samples and `tests/` dir), `test.rs` (`cpl test`: samples and test dirs, `run_cases`), `watch.rs` (`cpl watch`), `bench.rs` (`cpl bench` harness), `corpus_bench.rs` and `zero_cost.rs` (corpus-wide and zero-cost benchmarks; numbers owned by **perf**). Each command is `pub fn run(args, stdout, stderr) -> u8`.

### `crates/cpl-doc` (`cpl doc`)
`model.rs` (`extract` a `Module` of `Item`s and `Example`s from source), `markdown.rs`, `html.rs`, `highlight.rs` (renderers), `examples.rs` (doc examples as tests), `cli.rs` (`run`), `lib.rs` (`json` module: the doc JSON shape).

### `crates/cpl-json` (zero dependencies)
`Value` (objects are `BTreeMap`, so output is deterministic), `parse`, `to_string`, `to_string_pretty`, `escape_string`, `write_script_safe_string` (safe inside `<script>`), `obj`.

### `crates/cpl-lsp` (`cpl lsp`)
`transport.rs` (stdio framing: `read_message`, `write_message`, `serve`), `server.rs` (`Server`: document store, dispatch, diagnostics, hover, definition, code actions, formatting, semantic tokens), `complete.rs` (completion, signature help, parameter hints), `outline.rs` (document symbols), `rename.rs` (references, rename), `build.rs` (generates the `use std.` names from `std/`). Tests: `src/tests.rs`, `tests/{fixtures,fuzz,ports,latency}.rs`.

### `editors/`
`vscode/` (TypeScript client: `client.ts`, `completion.ts`, `inlayHints*.ts`, `lenses.ts`, `runner.ts`, `traceView.ts`, `languageData.ts` tables; TextMate grammar; snippets; tests in `test/`), `neovim/` (Lua, `test/smoke.lua`), `vim/`, `emacs/` (`smoke.el`), `helix/`, `zed/`, `jetbrains/` (Gradle, LSP4IJ), `tree-sitter-cp/`, `smoke.sh` (keyword parity with `crates/cpl-syntax/src/token.rs`, then each editor's smoke test when its binary exists).

## Dependencies

Each crate's `Cargo.toml` is the list. The one rule is that the language
server does not link the C++ backend, so the playground's engine stays
small.

## Invariants
- `cpl-json` has no dependencies and writes objects in key order: the same value always prints the same bytes.
- Every command function has the shape `fn(args, stdout, stderr) -> u8`; nothing in a command calls `std::process::exit` (that is `main.rs`'s job), so `run` is testable and `cpl-wasm` can call it.
- Exit codes in the binary: 0 ok, 1 errors in the source (`EXIT_ERRORS`), 2 usage (`EXIT_USAGE`), 3 the program or a test failed (`EXIT_FAILED`, `crates/cpl-cli/src/lib.rs`); `spec/tools/cli.md` says 1 failure, 2 compile error, 3 usage.
- Tools consume the compiler through its public crates; they never re-implement a check. Diagnostic text is the diagnostics lane's, formatting the lint-fmt lane's.
- `cpl lsp` never shells out to `cpl`; it links the same crates (so the playground's in-browser LSP and the desktop one agree).
- Editor keyword lists match `crates/cpl-syntax/src/token.rs` (`editors/smoke.sh`).


## Decisions this design rests on

- **Decision 31: the generated C++ runs on the judge's stack, with no
  depth limit.** A recursion-depth limit rejects correct programs that the
  judge accepts, such as a DFS down a path of 10^6 vertices. So `cpl run`
  stops a program only when it really runs out of stack, and `cpl build`
  runs `main` on a thread sized by the judge profile. The rejected
  alternatives were a fixed call-depth cap, and leaving the stack to the
  judge's default, which overflows on the same programs.
- **Decision 34: a judge is a profile, chosen flag first.** Profiles live
  in `cpl-judge`, a crate with no dependencies, so the CLI, the playground
  and stress share them. Codegen sees only a `Target` and never a judge
  name. The order is: `--judge`, then the file's `// judge:` line, then
  `cp.toml`, then `generic`. The most local statement wins, as with
  `// limits:`. The rejected alternative was judge names known inside
  codegen, where adding a course's judge would need a code change instead
  of a `[judge.<name>]` table.
- **Decision 188: one size budget for every judge.** A build warns above
  32 KB, or above half of a smaller judge limit, and fails above 64 KB or
  the judge's own limit when that is smaller, even when the judge accepts
  more (AtCoder's 512 KiB). The budget is the contestant's, not the
  judge's: a file that only fits a generous judge is library bloat, and the
  same solution may go to a stricter judge next. Profiles also carry
  `time_ms` and `memory_mb`, the limits a problem gets when its `cp.toml`
  has no `[limits]`. The rejected alternative, a warning at 75% of each
  judge's limit, fired at 384 KB on AtCoder and never caught anything.

## Changes

- 2026-11-23: first version, from the tools lane's old architecture notes.
