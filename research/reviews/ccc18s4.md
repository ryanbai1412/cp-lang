# ccc18s4

Score: 2/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the split at `isqrt(w)` and the O(n^(3/4)) state count.
- R3: the quotient grouping is hand-rolled (L15: `2..=w / r` plus `(w / q - w / (q + 1)) * trees(q) for q in 1..r`) although std has `floor_quotients(n, from:)`, which the dream uses; the report does not say why the port skips it.
- R10: 0.20 s vs 0.13 s (1.5x) at `--max`, reported as a medium performance friction.

## Owed rewrites

- Use `sum((b.hi - b.lo) * trees(b.q) for b in floor_quotients(w, from: 2))`, as the dream does, and re-measure.

## Language gaps

- No dense memo over the quotients of n (in the report, measured).

## Dream notes

`floor_quotients` with a `Quotients(n)` memo domain is the right ask.
