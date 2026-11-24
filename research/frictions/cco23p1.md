# cco23p1 — Binaria (CCO 2023 P1)

Port: `corpus/problems/cco23p1/port.cp`. The old port walked each residue chain with a raw
`Int` cursor and `Window(w)` conversions, and counted with `std.math.choose`, which is the exact
binomial in `Int` (it overflows long before `free` reaches 1e6) where the answer wants it mod
1e6 + 3. The rewrite steps the windows of each class (`Window.drop(r).step(k)`), reads the
chain's offsets as a prefix sum, and counts with `Comb<Mod<1_000_003>>`.

## Frictions

- **A prefix sum is not a sequence of its prefixes.** `steps.scan(+)` is a `Prefix` with
  `prefix(i)`, `fold` and searches, but no iteration, `min`/`max` or `collect`, so the span of
  the offsets (including the 0 before the first step) is
  `(0..=steps.len).map(i => off.prefix(i))` and two folds.
- **`adjacent` on a table didn't give pairs to `map`.** `total.adjacent.map(p => p.1 - p.0)`
  is E0302 ("expected `(Int, Int)`, found `List<Int>`"), so the steps are a comprehension over
  windows with a `next`.

## Dream

```cp
for r in 0..k:
    let (lo, hi) = total.adjacent.map((a, b) => b - a).drop(r).step(k).prefixes.minmax()!
```

## Measurements

Max case (n = 1e6, k = 1000): port 0.02 s, accepted C++ 0.12 s, same output.
