# ccc22s3

Score: 4/5
Dream: 3/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the greedy fill.
- R2: greedy: each note takes as many new distinct runs as `k` still allows: the editorial.
- R8: `i.idx` (L20, 21, 23) where the corpus spells `i.ord`.
- R5: `Pitch.first!` as filler before the fill (reported).

## Owed rewrites

- `i.idx` to `i.ord`.

## Language gaps

- Greedy fill with a filler value (in the report).

## Dream notes

`Note.try_build((i, tune) => ...)` decrements the outer `left` from inside the builder (`left -= take`), a hidden side effect (R7) that caps the dream at 3; the builder should carry `left` as state or the dream keep the loop.
