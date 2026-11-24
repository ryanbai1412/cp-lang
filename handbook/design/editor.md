# Editor and language server design

| | |
|---|---|
| Kind | Design |
| Owner | tools |
| Status | Active |
| Describes | `crates/cpl-lsp`, `crates/cpl-cli`, `crates/cpl-wasm` (the engine calls), `editors/` |

The terminal, the desktop editors and the browser playground all answer
from one analysis of the file. `cpl check` and the editor's red squiggle
are the same diagnostic, made by the same code. `cpl fix` and the editor's
fix-all make the same edits. A playground run prints what `cpl run` prints.

Example: a contestant types `rows.sort(by: (a, b) => (-b, a))` in VS Code.
The editor sends the text to `cpl lsp`, which hands it to the compiler's
analysis (`cpl_check`) and answers diagnostics, hover and completion from
the result. The playground runs the same server compiled to WebAssembly
(`crates/cpl-wasm`). `cpl check` on the saved file calls the same analysis
and prints the same diagnostic.

## What good looks like

- **One answer.** The CLI, the editors and the playground never disagree
  about what a file means. Each job has one entry point that the others call
  and add only presentation to.
- **Fast enough to feel instant.** Every request at 5,000 lines answers
  within 50 ms at p95 in a release build; every command starts in under
  100 ms.
- **Never break on half-typed code.** A file in the middle of an edit gets
  diagnostics, hover and completion, never an internal error.
- **Everything the compiler knows, shown where you type.** Types, named
  options, costs, storage and proven ranges come from the compiler's facts,
  never re-derived in a tool.
- **Completion offers only what the checker accepts.** Every item comes from
  the checker's resolution at the cursor, so accepting one never adds an
  error (decision 513). A hand table of members is a second copy of the
  library's surface: it drifts, and a suggestion that doesn't compile costs
  a round trip in the moment completion should save one.

## How it fits together

```
source text
  -> cpl_check (Database today, Session next): parse, names, types, facts
  -> Analysis: diagnostics with fixes, types, ranges, representation
       -> cpl check / cpl lint / cpl explain / cpl fix   (crates/cpl-cli)
       -> cpl lsp: diagnostics, hover, completion,       (crates/cpl-lsp)
          signature help, definition, rename, actions
       -> the playground                                  (crates/cpl-wasm)
       -> native editor UIs: test panel, lenses, trace    (editors/)
```

- **Analysis.** `cpl_check` owns the query database: it parses, resolves
  names, checks types and computes facts (`Analysis`). Tools only read it.
  A tool opens one `cpl_check::Session` per command (or `cpl_check::open`
  for one file once), handed down as `&mut Session`; only `cpl-check` can
  build a `Database`, and the `own_databases` gate in
  `scripts/debt-ratchet.py` fails on any site outside it.
- **Language server** (`crates/cpl-lsp`). `server.rs` keeps the open
  documents in one database and dispatches requests; `complete.rs` does
  completion and signature help, `outline.rs` symbols, `rename.rs`
  references and rename, `fix.rs` the one fix path. The server never shells
  out to `cpl`; it links the same crates, so the desktop and the browser
  agree.
- **Commands** (`crates/cpl-cli`). Every command is
  `fn(args, stdout, stderr) -> u8` and never exits the process, so the
  playground and the tests call it directly.
- **Playground** (`crates/cpl-wasm`). Wraps the language server and the
  command functions for the browser. It adds no logic of its own; the page
  around it is the site domain's.
- **Editor packages** (`editors/`). Each speaks LSP to `cpl lsp`. VS Code
  and JetBrains add native UIs (a test panel, lenses, the trace view,
  Competitive Companion import) by running `cpl` commands with `--json`,
  never by re-implementing them.

## Next: one signature model

Today hover, signature help, completion details and the docs each format a
function's signature themselves, so they drift: hover on `sort` can show
`sort()` without its `by:` and `desc:` options. The target is one signature
model built from the checker's resolved item (parameters, named options with
defaults, return type, cost, doc summary) that every surface renders. Its
checks are the hover snapshots, completion replay and definition coverage
rows of `handbook/domains/tools.md`.

## How we know it works

Parity tests run the CLI, the language server and the playground on every
conformance case, corpus solution and example and require the same
diagnostics, fixes, runs and traces. The server is fuzzed with cut and
garbled real files, its latency is measured at 5,000 lines on each OS, and
completion is replayed over the corpus. Hover, signature help and go to
definition are the weakest: fixtures cover a handful of cases, and the
signature model below does not exist yet.

## Where to start

- A request's behaviour: `crates/cpl-lsp/src/server.rs`, then
  `complete.rs` for completion and signature help.
- What the editor shows for a case: the fixtures in
  `crates/cpl-lsp/tests/fixtures/`; `BLESS=1` rewrites their expected
  output.
- A new shared job: add a row to the table below and a parity test in
  `crates/cpl-wasm/tests/parity.rs`.

# Reference

## One entry point per job

| Job | Entry point | CLI | Editor | Playground | Parity test |
|---|---|---|---|---|---|
| Diagnostics | `cpl_check` analysis, `cpl_lint::lint_analysis` | `cpl check`, `cpl lint` | published diagnostics | `check` | `crates/cpl-cli/tests/lsp_parity.rs`, `crates/cpl-wasm/tests/parity.rs` |
| Fixes | `cpl_lsp::fix::document` (error fixes, then lint fixes) | `cpl fix` | `source.fixAll` | fix-all | `crates/cpl-wasm/tests/parity.rs` |
| Quick fix for one diagnostic | `cpl_ir::fix::checked_fix` on the server's own database | | code action | code action | `crates/cpl-lsp/tests/fixtures/fixes.cp` |
| Run and trace | `cpl_cli::run`, `cpl run --trace` | `cpl run` | lenses, trace view | Run, Trace | `crates/cpl-wasm/tests/parity.rs` |
| Storage and costs | the compiler's representation query | `cpl explain` | hover | explain | `crates/cpl-lsp/tests/explain_hover.rs` |
| Cases of a problem | `cpl_contest::problem::find_tests_dir` | `cpl test` | test panels | | `editors/vscode/test/unit/results.test.ts`, `editors/jetbrains/src/test/kotlin/dev/cplang/ide/CplTest.kt` |

A new job gets a row here and a parity test before a second caller uses it
(`handbook/guidelines/commands-checklist.md#r10-one-entry-point-per-job`).

## Invariants

The rules every command and editor feature follows are in
`handbook/guidelines/commands.md`. The design adds these:

- The language server applies a quick fix on its own database and then
  restores the document's text, because the editor has not applied the edit
  yet.
- Edits handed to an editor never overlap; `cpl_diag::apply_fixes` rejects
  overlapping edits and orders insertions before replacements at the same
  position.
- Case order is the same everywhere: numbered cases by number, then the
  rest by name.
- Editor keyword and std name tables match the compiler
  (`crates/cpl-cli/tests/vscode_sync.rs`, `editors/smoke.sh`).

## Measures

The quality register in `handbook/domains/tools.md` lists every check;
targets and current numbers are in `plan/domains/tools/metrics.toml`.


## Decisions this design rests on

- **Decision 29: hover shows the declaration's own `///` doc, and the
  editors are thin.** The doc above a definition is where its cost is
  written, so hover reads it from wherever the name is declared, std
  included, instead of keeping a second table of descriptions. Neovim,
  Emacs and Helix get setups in their own idiom that only start `cpl lsp`.
  Every feature lives in the server, so it reaches every editor and the
  playground at once. The rejected alternative was per-editor plugins with
  their own logic, which drift apart and each need their own tests.

## Changes

- 2026-11-23: first version, from the tools lane's old architecture notes and
  `plan/rearchitecture.md` (one analysis pipeline).
