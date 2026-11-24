# dmopc20c5p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Each row r of an n by m grid crosses columns `m r / n` up to `ceil(m (r + 1) / n)`; the port counts and prints those ranges directly.

## Checklist failures

- `(m * Int(r) / n)..(m * (Int(r) + 1)).ceil_div(n)` converts the row to `Int` twice inside the formula.
- `print(Int(r) + 1, c + 1)` shifts to 1-based output by hand because `Row = 0..n` is 0-based; a 1-based domain would print as is.
