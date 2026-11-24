# Friction Report: cco18p4

Already at bar: rows and columns are domains with their bounds, the search is std `ternary_min`,
and scores are typed `0..=2e9`.

## A memo may not ask
- **Where:** `seen: HashMap<(Row, Col), Int>` and `score`.
- **What felt wrong:** caching the answers of `ask` is the textbook `@memo`, but a memo may not do
  I/O (spec 6.4), so the cache is spelled by hand.
- **Ideal:** `@memo(io)` for idempotent queries, or a std `asked(...)` cache for interactive
  problems.

## Dream
The committed `dream.cp` (`argmin_convex` and `wire.ask`); the port keeps the golden-section
search, which reuses probes and fits K = 75.

- Tokens: n/a (interactive)
- Package cases: the 13 reply transcripts that the case converter had turned into inline cases are
  removed; all 17 real cases pass.
