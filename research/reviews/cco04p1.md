# cco04p1

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the per-word letter count test.
- R2: count each word's letters against the tiles: the editorial.
- R3: `used` is a hand count (L23-24) where std `tally` counts; `score`/`have` are filled by a loop.
- R5: tiles and words are read as `Char`/`Str` and converted (reported, run-time error quoted).

## Owed rewrites

- Count with `tally`, as the dream's `Counter(w) <= have`.

## Language gaps

- A character domain cannot be read (in the report).

## Dream notes

Reads like the editorial.
