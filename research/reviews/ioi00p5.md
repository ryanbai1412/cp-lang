# ioi00p5

Score: 3/5
Dream: 3/5 (the friction's `## Dream`: the port as written)

## Cold review (HEAD)

Port 3/5, dream 3/5. Two `@memo` functions: `serve` for one run at its median and `best` over suffixes with `?Int` for "no plan" (lines 18-30). The header derives both.
- The candidate cost `serve(i, j) + best(j + 1, k - 1)` is written twice (lines 29 and 38): the reconstruction replays the recurrence. In the friction (a memo that keeps its argmin).
- The median index `(i + j) / 2` is also written twice (lines 20 and 39), and line 38's `while ...: j += 1` is a first-match search that `(i..n).first(j => ...)!` says today (idiomatic-code/R3, R4).

## Owed rewrites

- `fn median(i, j)` once; the reconstruction as a `first`.
