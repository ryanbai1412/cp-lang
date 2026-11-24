# cheerio1s5

Score: 3/5
Dream: 4/5

Cold review against the friction's `## Dream`. The two DPs are the header's (lines 6-9), but both are filled as raw-`Int` cut tables with nested loops (lines 19-32), mixing cuts and coin positions (`clear[l, i] + clear[i + 1, r]`, line 23); the friction reports that a cut-typed memo cannot name the elements between two cuts. Translated, with the gap reported: 3.

## Checklist failures

- R6: `clear: [0..=n, 0..=n]Int` and `best: [0..=n, 0..=k]Int` (lines 19, 26) index by raw cuts; reported.

## Owed rewrites


## Language gaps

- Iterating the elements between two cuts, so `clear` can be a cut-typed memo: in the friction.
- `head.map(Int)` (a named `Bool -> Int`): in the friction.

## Dream notes

- `clear` as one memoized `max(...) ?? 0` over the H's between two cuts names the recurrence. Judged as the port with it; `best` stays the port's table.
