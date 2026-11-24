# ccc14s4hard

Score: 3/5
Dream: 3/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the sweep and the sqrt blocks with their cost.
- R2: the sweep reads like the editorial; the threshold structure is a hand-written square-root decomposition, reported (std has nothing for "range add, count weight at or over T").
- R2/R4: one block's state is four parallel arrays (`pending`, `rows`, `upto`, `shade`, L31-36) kept in step by `heights`/`measure`/`touch`; a `struct Block` would own the invariant.
- R4: the two `events` comprehensions (L72-76) repeat the same record but for `x` and the tint's sign.
- R6: strip ends travel as `lo`/`hi: Int` ordinals (`ys.index_of(...).ord`) instead of domain values.
- The friction report names `rebuild` and `dark`; the port calls them `heights` and `measure`.
- R10: 6.46 s vs 6.15 s (1.05x), measured.

## Owed rewrites

- Fold the per-block arrays into a `struct Block` with `add`/`measure` methods.
- Build `events` once per pane side from `[(p.x0, p.tint), (p.x1, -p.tint)]`, keeping strip values, not ordinals.

## Language gaps

- No range-add / weight-at-least structure in std (in the report).
- No gap domain over a compressed axis (in the report).

## Dream notes

`sum(... after tint.add(...) ...)` mutates inside the sum (R7), which caps the dream at 3. It should be a `for (a, b) in events.adjacent:` loop that adds, then accumulates the area. The port is judged against that, so this score is provisional.
