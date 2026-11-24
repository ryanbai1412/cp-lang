# Friction report: ioi94p1

## Dream

```cp
let n: 1..=100 = read()
let rows: [r: 1..=n][0..r]0..=99 = read()
print(rows.fold([], (best, row) => [x + max(best.get(c - 1), best.get(c)) ?? 0 for (c, x) in row.items]).max()!)
```

## Missing neighbours

The edge cells have one parent, so the port reads `best.get(c - 1) ?? 0`. A max over
optionals (`max(?a, ?b)`, `None` when both are missing) would say "the parents that exist"
instead of standing in 0, which only works because values are non-negative.

## A fold over rows read one at a time

The rows are read one by one inside the loop, so `best` is a `var` rebuilt per row. A
ragged-row read (`[r: 1..=n][0..r]T`) followed by a fold would make it one expression.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++, and the max case
(n = 100) matches. Corpus tests pass.
