# diagnostics architecture

The code as it is on main (4b8f504f, 2026-11-22), from reading it. The lane
keeps this current in the same commit as any structural change.

## Modules

`crates/cpl-diag` (1771 lines, zero dependencies) is the vocabulary every
tool reports in. It decides nothing about the language; the error sites do.

| File | Lines | What it is for |
|---|---|---|
| `crates/cpl-diag/src/lib.rs` | 490 | `Span` (byte range, item-relative until `shift`), `Severity`, `Code`, `codes::*` (90 named constants), `Label`, `Fix` (title + edits), `Diagnostic` (builder: `error`, `warning`, `with_label/note/help/fix`, `shift`), `Sink` (dedup + poisoning) |
| `crates/cpl-diag/src/registry.rs` | 957 | `REGISTRY: &[CodeInfo]` (91 entries): title, summary, explanation, verified trigger and fix, mode (`check`/`run`/`build`/`reserved`/`internal`), stdin. Source of `cpl explain` and the site's error pages |
| `crates/cpl-diag/src/render.rs` | 170 | `render`, `render_all`: the `error[E0301]: ... --> f.cp:7:17` text form with carets, labels, notes, help |
| `crates/cpl-diag/src/source.rs` | 154 | `LineIndex`, `LineCol`: offsets to lines/columns, UTF-16 for the LSP |

`spec/diagnostics.md` (146 lines) is the normative catalogue: every code,
its family, name, example message and status (*(0.3)* = specified, not yet
emitted; *(retired)*; *(reserved)*).

The error sites are not in this crate. Counting `codes::` /
`Diagnostic::error` / `Diagnostic::warning` uses per file
(`grep -rc` over `crates/`): `cpl-ir/src/lower/call.rs` 119,
`cpl-check/src/check/expr.rs` 74, `cpl-ir/src/lower/expr.rs` 56,
`cpl-check/src/check/stmt.rs` 56, `cpl-ir/src/lower/stmt.rs` 43,
`cpl-syntax/src/parser/stmt.rs` 41, `cpl-check/src/db.rs` 38,
`cpl-check/src/check/lends.rs` 33, `cpl-check/src/check/call.rs` 31,
`cpl-syntax/src/parser/expr.rs` 30, `cpl-syntax/src/lexer.rs` 19. Those files
are owned by compiler; diagnostics changes messages, spans, fix-its and
suppression there by request in `plan/domains/compiler/worklog.md`.

Runtime traps (`R` codes) are built by `Trap::to_diagnostic` in
`crates/cpl-interp`, shared by `cpl-cli` and `cpl-wasm` (decisions.md).

## Interfaces other domains use

| Interface | Consumers | Held by |
|---|---|---|
| `Diagnostic`, `Span`, `Severity`, `Fix`, `Sink` | every front-end and tool crate: cpl-syntax, cpl-check, cpl-ir, cpl-interp, cpl-codegen, cpl-lint, cpl-fmt, cpl-lsp, cpl-cli, cpl-stress, cpl-contest, cpl-doc, cpl-conform, cpl-wasm | `crates/cpl-diag/src/lib.rs` unit tests (15 tests in the crate) |
| `codes::*` | the error sites above | `crates/cpl-cli/tests/catalogue.rs` |
| `REGISTRY`, `lookup` | `cpl explain` (`crates/cpl-cli/src/lib.rs` `explain_code`), site error pages, `site/check_docs.py` (index drift), `scripts/conformance-coverage.py` | `catalogue.rs` (every trigger emits exactly its code, every fix runs clean) |
| `render`, `render_all` | `cpl check/run/build`, goldens | `crates/cpl-cli/tests/diagnostics.rs` (53 `.err` goldens + 2 `cpl fix` goldens) |
| `LineIndex::utf16` | cpl-lsp | cpl-lsp tests |
| `Diagnostic.fixes` | `cpl fix` (`apply_error_fixes`, `crates/cpl-cli/src/lib.rs`), LSP quick fixes (`crates/cpl-lsp/src/server.rs`) | `tests/fixes/*.fixed`, recovery `fix_it_scoreboard` |

The feedback loop: `crates/cpl-cli/tests/recovery.rs` breaks every clean
`corpus/problems/*/spec.cp` in the `SPEC_PORTS` list (37) with six accidental
edits and six common mistakes on up to three lines each, and scores first
error on the right line, cascades, fix-its and restores; it also checks the
fenced cp blocks of `spec/{core,lib,tools}`.

## Invariants

- A code never changes meaning; retired codes are never reused
  (`spec/diagnostics.md`).
- Every code an error site emits is in the catalogue; every emitted code has
  a registry trigger that produces exactly it and a fix that is clean
  (`catalogue.rs`). Error sites use `codes::` constants; the one raw lint
  literal left is listed below.
- One per root cause, mechanically: `Sink::push` drops anything inside a
  span the parser poisoned and a second diagnostic at the same span;
  `Sink::finish` keeps only the first use of each unimplemented feature.
- Spans are item-relative inside the parser and shifted to file offsets
  once (`Diagnostic::shift`), so an edit elsewhere keeps an item's
  diagnostics cached.
- `cpl-diag` has no dependencies and no language knowledge.

## Known debt

1. Both recovery scoreboards are `#[ignore]`d since 954d6842 (edition 0.2
   deleted): `crates/cpl-cli/tests/recovery.rs:562`, `:589`. Nothing gates
   recovery today.
2. Raw code literals outside `codes::`: the E-code sites now use `codes::`
   (E0703, E0704, E0705 gained constants and registry triggers). Left:
   `crates/cpl-check/src/check/stmt.rs:1623` emits lint `L0609` raw (see 4),
   and `crates/cpl-lsp/src/server.rs:1832` makes `Code("overflow")`, not a
   catalogue code (request in the tools worklog).
3. Every catalogue row has an example message (was 33 empty; `spec/diagnostics.md`).
4. Lint codes live in two registries (`crates/cpl-lint/src/lib.rs` makes
   `Code("L…")` itself; `codes::` also has L0601-L0608). One registry should
   own every code.
5. The benchmark base is a hand list (`SPEC_PORTS`) instead of "every
   `spec.cp` that checks clean", so it does not grow with the corpus.
6. The scoreboard counts "first right" by line, so `let` for a `var`
   (error at the write, fix at the declaration) scores 0.0% first-right
   though the error is arguably right.
7. `Sink` is used by the parser and checker (`Sink::` in `cpl-syntax`,
   `cpl-check`); `cpl-ir` lowering pushes to its own vectors, so its
   cascade suppression is ad hoc (`Ty::Error` checks per site).
