# ccc23s3

Score: 3/5
Dream: 3/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header lists every case.
- R2: the cases as the editorial, but the poster is built from `letter(i: Int, j: Int)` on raw coordinates (L28-41) and transposed by swapping them (L43).
- R3: std's `transposed()` already builds the swapped poster (`cpl run` in the report, now marked not a gap); the port does not use it.
- R6: `Int` coordinates instead of `Row`/`Col`.

## Owed rewrites

- Build the extreme-side poster over `[Col, Row]` and call `.transposed()`, as the dream does.

## Language gaps

None.

## Dream notes

`fn frame<A, B>(pal: Int, cross: Int) -> ?[A, B]Char` has no body, so the hard half of the dream is elided; 3 until it is written.
