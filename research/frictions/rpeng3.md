# Friction Report: rpeng3

## Dream

```cp
type Profit = -5e6..=2e9
var best = SegTree<Loc, Profit>(max)          // best profit standing at each location
fn arrive(x: Loc) -> Profit =
    max(best.fold(..=x, shift: p => -(x - p) * down), best.fold(x.., shift: p => -(p - x) * up))
for day in fairs.sorted().runs(by: f => f.t):
    let stretch = day.sweep_both_ways(enter: arrive, gain: f => f.gain, step: (up, down))
```

## Gaps

- **Distance-shifted folds take two trees.** Folding `best[p] - |x - p| * cost`
  needs the shift folded into the key, so the port keeps `best + p * down`
  and `best - p * up` in two trees. Severity: low.
- **The empty-fold identity is a proved low end.** `Key` is a range alias
  whose low end lies below every stored key, which takes the bound argument
  in the comment; `?Key` with `None` would need no argument. Severity: low.
- **A run is indexed by position.** `runs(by:)` gives views keyed by the
  original positions, so the port reads `day.nth(q.ord)` into a local `Q`
  domain. Severity: low.
