# Friction Report: ccc21s4

## A SegTree built from a function keeps that function
- **Where:** `SegTree<Stop, ?Int>(min, [Stop] i => arrival(i))` and `fastest[i] = arrival(i)`.
- **What felt wrong:** `SegTree.of(min, arrival)` with `refresh(i)` says "the tree mirrors
  `arrival`" in one place, but the tree stores `arrival` as a `std::function` and calls it
  dynamically on every refresh; the port writes each leaf back by hand to stay static.
- **Goal it hurts:** costs are visible; the obvious way is the fast way.
- **Ideal code:** `var fastest = SegTree.of(min, arrival)` / `fastest.refresh(x)` with the
  function kept as a template parameter.
- **General fix:** monomorphise `SegTree.of`'s source into the tree type (Core codegen), so
  `refresh` inlines it.
- **Severity:** medium

## Dream

```cp
let n: 3..=2e5 = read()
let w: 0..=2e5 = read()
let d: 1..=2e5 = read()
domain Station = 1..=n
domain Stop = 1..=n
let walkways: [w](from: Station, to: Station) = read()
var line: [Stop]Station = read()
let swaps: [d](x: Stop, y: Stop) = read()
let on_foot = bfs(group((to, from) for (from, to) in walkways, over: Station), from: [Station.last!])
fn arrival(i: Stop) -> ?Int: i.ord + on_foot[line[i]]?
var fastest = SegTree.of(min, arrival)
for (x, y) in swaps:
    line.swap(x, y)
    fastest.refresh(x, y)
    print(fastest.all!)
```

## Measurements

- Tokens: port 325, original 648
- Max input (`gen.py --max`): port 0.05 s, 23892 KB, original 0.34 s, 23040 KB
- Emitted C++: 21978 bytes
