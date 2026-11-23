# Diagnostics design: how an error is built and reported

| | |
|---|---|
| Kind | Design |
| Owner | diagnostics |
| Describes | `crates/cpl-diag/`, `spec/diagnostics.md`, `crates/cpl-ir/src/fix.rs`, `Trap::to_diagnostic` in `crates/cpl-interp/` |

How an error gets from the place the compiler notices it to the user's
screen, and the rules every change keeps. What diagnostics must be good at
is its charter, `handbook/domains/diagnostics.md`; the rules for a good
error are `handbook/guidelines/errors-checklist.md`.

## An error's path to the user

1. **Noticed.** The parser, checker and lowering find the problem and build
   a `Diagnostic`: a code from `codes::`, a primary span, labels, notes,
   help and candidate fixes. Those error sites belong to compiler;
   diagnostics edits their wording, spans, fixes and suppression.
2. **Collected.** A `Sink` drops anything inside a span the parser marked
   broken, and a second diagnostic at the same span, so one mistake reports
   once. A name declared by a broken line gets the error type, which stays
   silent where it is used.
3. **Fixed.** `cpl_ir::fix` re-checks each candidate fix and keeps the one
   that clears the error, then applies fixes in rounds until none is left.
   `cpl fix`, the editor's quick fixes and fix-all, and the playground all
   call it, so they make the same edits.
4. **Shown.** `render` prints the `error[E0301]: ... --> f.cp:7:17` text
   form; the language server turns the same `Diagnostic` into LSP JSON, with
   UTF-16 columns from `LineIndex`.
5. **Explained.** `cpl explain E0301` prints the code's page from
   `REGISTRY`, which also holds a trigger that must produce exactly that
   code and a fix that must check clean.

Runtime traps take a shorter path: the interpreter raises a `Trap`, and
`Trap::to_diagnostic` turns it into a `Diagnostic` with the code, the
values, help in CP Lang terms and the call stack. The CLI and the
playground share it.

## Parts

- `crates/cpl-diag/src/lib.rs`: `Span`, `Severity`, `Code`, `codes::*`,
  `Label`, `Fix`, `Diagnostic` and `Sink`.
- `crates/cpl-diag/src/registry.rs`: `REGISTRY`, one entry per code, the
  source of `cpl explain`.
- `crates/cpl-diag/src/render.rs`: the text form.
- `crates/cpl-diag/src/source.rs`: byte offsets to lines and columns.
- `spec/diagnostics.md`: every E, W and R code with its name and example
  message; `spec/lints.md` (generated) does the same for lints.

## Rules every change keeps

- A code never changes meaning, and a retired code is never reused.
- Every code an error site emits is in the catalogue and has a `codes::`
  constant and a `REGISTRY` entry (`crates/cpl-cli/tests/catalogue.rs`).
- Spans are relative to their item inside the parser and shifted to file
  offsets once, by `Diagnostic::shift`, so an edit elsewhere keeps an
  item's diagnostics cached.
- `cpl-diag` has no dependencies and knows nothing about the language; the
  error sites decide what is an error.
- Lowering in `cpl-ir` still reports through its own vectors, not a `Sink`,
  so its cascade suppression is per site. Moving it onto `Sink` is open
  work.
