# coci21c6p2

Score: 3/5
Dream: none

The port follows the header's argument: collapse row bands into column sums (L19–22, `for c in 0..s: column[c] += price[bottom, c]`), then sweep the two candidate windows (L25–30). The named objective (L16, `fn cost(m: Int) -> Int = abs(m - a) + abs(m - b)`) keeps the minimization readable. Raw row and column indices weaken the typing, and the missing dream caps the score at 3.

## Checklist failures
- idiomatic-code/R6: distinct row and column spaces remain raw integer ranges: L19 `for top in 0..r:`, L21 `for bottom in top..r:`, and L22 `for c in 0..s: column[c] += price[bottom, c]`. L14 `let price: [r, s]1..=1e9 = read()` does not distinguish those spaces.

## Owed rewrites
- Introduce row and column domains for the grid, row-band iteration and column indexing (L14, L19–22). Preserve the sweep's end-position handling at L26, `while right < s and sum < min(a, b):`.

## Language gaps
None identified.

## Dream notes
None: there is no `dream.cp` and no `## Dream` in the friction report, so the port is capped at 3.
