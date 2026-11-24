# Friction Report: ioi08p6

## Dream

```cp
var cover = SegTree<Y, Strip, Int>(join, act: shift_low, a: [Y] _ => Strip.cell)
let free = cover.all.runs_at(0)
```

## Gaps

- **"Longest run at the minimum" is a hand-written monoid.** `Strip`,
  `at_low`, `join` and the `Action` that shifts `low` are the classic
  min-and-runs node; std has min-count (`low`, how many) but no runs at the
  minimum. Severity: medium.
- **Struct-update has no short form.** The action rebuilds every field of
  `Strip` to change `low`; a `s with low: s.low + d` would say it. Severity: low.
- **The `B > 0` search builds its sweep lists per side.** `cheapest(k)` rebuilds
  `starts`/`stops` for each `k` of the binary search, because the boxes'
  corner ranges depend on `k`. Severity: low.
