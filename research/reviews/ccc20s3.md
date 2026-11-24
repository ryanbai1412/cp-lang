# ccc20s3

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1/R5: no `// Bounds:` line; `needle` and `hay` are unbounded `Str` reads (L12-13), where the lengths up to 2e5 belong in the type.
- R2: a sliding window with a mismatch count and a rolling hash in a set: the editorial.
- R3: the window and its count are hand-rolled (reported).
- R10: 0.00 s vs 0.06 s.

## Owed rewrites

- Bound both reads and add the `// Bounds:` line.

## Language gaps

- No sliding count-equality window (in the report).

## Dream notes

Names the window's counts against the needle's; reads like the editorial.
