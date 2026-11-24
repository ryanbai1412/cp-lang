# Friction Report: ccc14s4

## Running sums over a 2-D array by hand
- **Where:** the two `for (j, next) in Y.all.adjacent: tint[i, next] += tint[i, j]` passes.
- **What felt wrong:** "turn a 2-D difference grid into values" is two in-place scans written
  out; `.adjacent` of a bare domain is a codegen gap (E0501), so it goes through `.all`.
- **Goal it hurts:** brevity; one spelling per idea.
- **Ideal code:** `let tint = marks.prefix_sums()` on a `[X, Y]Int`, with `Y.adjacent` direct.
- **General fix:** an in-place 2-D `scan`/`prefix_sums` for `[R, C]T`, and codegen for
  `.adjacent` of a domain.
- **Severity:** low

## Dream

```cp
let n: 1..=1000 = read()
let need: 1..=1e9 = read()
let panes: [n](x0: 0..=1e9, y0: 0..=1e9, x1: 0..=1e9, y1: 0..=1e9, tint: 1..=1e9) = read()
let (domain X, xs) = compress(panes.flat_map(p => [p.x0, p.x1]))
let (domain Y, ys) = compress(panes.flat_map(p => [p.y0, p.y1]))
var marks: [X, Y]Int = 0
for p in panes: marks.add_rect(xs.index_of(p.x0)..xs.index_of(p.x1), ys.index_of(p.y0)..ys.index_of(p.y1), p.tint)
let tint = marks.prefix_sums()
print(sum((xs[i2] - xs[i]) * (ys[j2] - ys[j]) for (i, i2) in X.adjacent for (j, j2) in Y.adjacent if tint[i, j] >= need))
```

## Measurements

- Tokens: port 382, original 1306
- Max input (`gen.py --max`): port 0.04 s, 34176 KB, original 0.02 s, 3840 KB
- Emitted C++: 23534 bytes
