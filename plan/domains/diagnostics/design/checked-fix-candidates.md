# Checked fix candidates (2026-11-22, proposed)

`spec/tools` says `cpl fix` applies each error's fix; it does not say what
happens when the right edit can't be known from the text alone. This note
does, until it is built.

## Problem
A missing closer can belong in several places, and only types tell them
apart. On the broken-ports benchmark (`RECOVERY_SHOW=unclean cargo test -p
cpl-cli --test recovery -- --nocapture`), 41 of 176 E0101 fixes leave the
file with errors after 9ba87803a:

| Broken | Fix offered today | Right |
|---|---|---|
| `SuffixArray(s.distinct() + 1` | `SuffixArray(s.distinct() + 1)` | `SuffixArray(s).distinct() + 1` |
| `(2..=n.find(p => ...)!` | `(2..=n.find(p => ...)!)` | `(2..=n).find(p => ...)!` |
| `sieve.divisors(a.any(d => kept[d]):` | `...kept[d])):` (by recovery) | `sieve.divisors(a).any(...)` |

The lexer picks one spot (`end_of_unclosed` in
`crates/cpl-syntax/src/lexer.rs`), usually the latest, and it is right only
when the latest spot is the only one that parses.

## Design
- **Errors offer ranked alternatives.** A diagnostic whose edit is
  ambiguous lists every plausible fix in `Diagnostic::fixes`, best first.
  Nothing new in `cpl-diag`: `fixes` is already a list, and the LSP already
  shows each one as a quick fix (`crates/cpl-lsp/src/server.rs`, code
  actions).
- **`cpl fix` keeps the first that checks.** For a diagnostic with more
  than one fix, `apply_error_fixes` (`crates/cpl-cli/src/lib.rs`) applies
  each candidate alone and re-checks. It keeps the first candidate after
  which no error starts on or before the diagnostic's line. If none
  qualifies, it keeps `fixes[0]`, which is today's behaviour. A diagnostic
  with one fix is unchanged.
- **Capped.** At most `MAX_CANDIDATES = 8` re-checks per diagnostic and 32
  per run. A 100-line contest file checks in 10-30 ms
  (`/usr/bin/time target/debug/cpl check corpus/problems/ccc18s4/spec.cp`:
  0.02 s debug, 0.01 s release), so the worst case stays under a second.
  The LSP never re-checks; it lists candidates in rank order.
- **Candidates for E0101.** Close before each `.name` or binary operator
  at depth 0 after the opener's first argument, nearest first, then today's
  spot. Cap 8.

## API wanted from tools (owns `cpl-cli`)
```rust
/// The first of `diag.fixes` (at most MAX_CANDIDATES) after which `path`
/// has no error starting on or before `diag`'s line; `fixes[0]` if none.
pub fn checked_fix<'d>(db: &mut cpl_check::Database, path: &str, text: &str,
                       diag: &'d Diagnostic) -> Option<&'d Fix>;
```
`apply_error_fixes` calls it per diagnostic instead of
`cpl_lint::apply_fixes`'s `fixes.first()`, applying the chosen fixes in one
pass as today (the clash check stays).

## Measures
- Broken ports, E0101 first errors whose fix leaves a clean file: 135 of
  176 today, target >= 165.
- `cpl fix` wall time on the 37 benchmark ports, p95: under 100 ms.
- Common mistakes and the golden fixes (`tests/fixes/`) unchanged.

## Alternatives
- **Check inside the lexer.** It has no types; rejected.
- **A smarter text heuristic.** Each new rule fixes 2-8 cases and risks
  others (9ba87803a); the ambiguous cases are ambiguous in the text.
- **Ask the user.** The LSP already does, by listing every candidate; the
  CLI has no one to ask.

## Reversal
Drop `checked_fix` and `apply_error_fixes` takes `fixes[0]` again. The
extra candidates stay harmless LSP quick fixes.
