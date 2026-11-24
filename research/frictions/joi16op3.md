# Friction report: joi16op3

## Dream

```cp
let heights: [Building]1..=1000 = read(distinct)
let ways = heights.sorted().with_prev().fold(start, (ways, (level, h)) => insert(ways, h - level))
print(ways[1, .., 2].sum())
```

## Distinct heights

The statement promises distinct heights, so the port writes
`assert heights.sorted().pairs().all(p => p.0 < p.1)`. A `distinct` read bound (or a
`.is_strictly_sorted()`) would say it in the type.

## Rise from the previous height

`level` is a `var` holding the previous sorted height, so each step knows how far the open
sides rise. A `with_prev` / adjacent walk over the sorted heights would make it a fold.

## A state cube keyed by plain ranges

`ways` is `[0..=n, 0..=limit, 0..=2]Z`. The three axes are groups, cost and pinned ends,
which a typed DP state would name. Today the ranges stay as `Int` ranges because the
transitions do arithmetic on all three.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
(N = 100, L = 1000) matches in 0.07 s. Corpus tests pass.
