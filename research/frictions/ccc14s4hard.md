# Friction Report: ccc14s4hard

## No range-add, count-at-least structure in std
- **Where:** `Block`, `strips`, `rebuild`, `dark`, `add` (about 30 lines).
- **What felt wrong:** "add to a range of strips, measure the height of strips at or over T" is the
  whole problem, and std has no structure for it, so a square-root decomposition with sorted blocks
  and a pending add per block is written by hand.
- **Ideal:** a `std.sqrt` blocked array (`Blocks<Strip, Int>` with `add(range, t)` and
  `weight_at_least(t)`), or a segment tree with "chmin/threshold" beats generalized to weighted
  counts.

## Strip ends come from the cut list by hand
- **Where:** `domain Strip = 0..Cut.len - 1` and `height(s)` via `Cut.nth(s.ord + 1)`.
- **Ideal:** `Cut.gaps` as a domain of neighbouring pairs of a compressed axis, with `.width`.

## Dream
```cp
use std.sqrt.{Blocks}
let n: 1..=1e5 = read()
let need: 1..=1e9 = read()
let panes: [n](x0: 0..=1e9, y0: 0..=1e9, x1: 0..=1e9, y1: 0..=1e9, tint: 1..=1e9) = read()
let (domain Cut, ys) = compress([p.y0 for p in panes] + [p.y1 for p in panes])
var tint = Blocks(Cut.gaps, weight: g => ys.width(g), of: 0)
var events = [(x: p.x0, gaps: ys.gaps(p.y0, p.y1), t: p.tint) for p in panes] +
             [(x: p.x1, gaps: ys.gaps(p.y0, p.y1), t: -p.tint) for p in panes]
events.sort(by: e => e.x)
print(sum((b.x - a.x) * tint.weight_at_least(need) after tint.add(a.gaps, a.t) for (a, b) in events.adjacent))
```

- Tokens: port 731, original 1306
- Max input (`gen.py --max`): port 6.46 s, 26792 KB, original 6.15 s, 34316 KB (both over the
  local 2 s default; the judge limit is not in the package, so tests.toml has no `--max` case)
