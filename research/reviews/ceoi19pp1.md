# ceoi19pp1

Score: 3/5
Dream: 4/5

Cold review against `dream.cp`. The program is the dream: tally the gaps of `a`'s pairs (line 14), sum the tallies at `b`'s gaps (line 15). But the coordinates are read as `[n]Int` (lines 12-13) where the statement gives strictly increasing values in `0..=2^30`.

## Checklist failures

- R5: `let a: [n]Int` / `let b: [m]Int` (lines 12-13); the friction's reason is that the package's inline cases hold negative, unsorted coordinates. That is a test-data defect, not a language gap, so the excuse does not hold.
- The friction states the same reason twice ("Coordinates stay `Int`" and "Coordinates read as Int").

## Owed rewrites

- Replace the invalid inline cases with statement-valid ones, then read `[n]0..=2^30` and `[m]0..=2^30` and add them to the `// Bounds:` line.
- Merge the two duplicate friction sections.

## Language gaps

- None.

## Dream notes

- Two named std calls and the destructured `(x, y) => y - x`; it shares the unbounded reads, which the rewrite above fixes in both.
