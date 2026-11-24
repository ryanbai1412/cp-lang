# ceoi18p5

Score: 3/5
Dream: 4/5

Cold review against the friction's `## Dream`. The header (lines 6-10) reduces the problem to multiplicative partitions scored by `sum (factor - 1)`; the port enumerates them by hand with a divisor-index recursion (lines 17-22) because std has no `factorisations`, reported. A standard enumeration hand-rolled for a reported std gap is a 3.

## Checklist failures

- R3: `split(rest, from, score)` (lines 17-22) walks non-decreasing divisors by index into `divs`; reported.

## Owed rewrites

- Quote the `cpl run` of the dream's `factorisations(n, min_factor: 2)` line in the friction (R11).

## Language gaps

- Multiplicative-partition enumeration (`factorisations`): in the friction.

## Dream notes

- Names the object: `factorisations(n, min_factor: 2).map(f => f.sum() - f.len).to_set()`. Write `f.sum()` (the friction has `f.sum`, which is not std's spelling).
