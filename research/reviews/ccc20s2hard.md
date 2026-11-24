# ccc20s2hard

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the value graph and the CSR.
- R3: CSR by counting sort is hand-rolled (L22-35) because `Graph.of` buffers every arc: 568 MB measured against the 512 MB limit (reported).
- R6: `R` includes the zero row the judge passes (L15), so cells are walked with `R.all.drop(1)`.
- R10: 0.69 s vs 1.16 s.

## Owed rewrites

None.

## Language gaps

- `Graph.of` cannot build in two passes from a re-iterable source (in the report, measured).

## Dream notes

Reads `domain R = 1..=rows`, but the input carries the zero row and column the port keeps (L15), so the dream misreads it; it needs `R = 0..=rows` with `R.after_first`.
