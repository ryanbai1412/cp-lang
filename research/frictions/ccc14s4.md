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

## Adding a rectangle on a 2-D difference grid
- **Where:** the four corner marks of each rectangle on `marks` before the two running-sum passes.
- **What felt wrong:** the corner pattern (`+v`, `-v`, `-v`, `+v` at the four corners) is spelled by
  hand. `cpl run` of `marks.add_rect(X(0)..X(2), Y(0)..Y(2), 5)` fails with
  `error[E0203]: a [X, Y]Int has no method add_rect`.
- **Goal it hurts:** obvious way is correct (a sign or an off-by-one corner is the usual bug).
- **Ideal code:** `marks.add_rect(xs.index_of(r.x0)..xs.index_of(r.x1), ys.index_of(r.y0)..ys.index_of(r.y1), r.tint)`
  then `marks.prefix_sums()`.
- **General fix:** a 2-D difference array in std (`add_rect` plus the prefix-sum read-back), the
  2-D form of the 1-D `Diff`.
- **Severity:** low
- **Owner:** stdlib.

## The formatter splits a two-`for` generator's tuple pattern
- **Where:** the final `print(sum(...))`, port L35-41: `for (` / `j,` / `j2,` / `) in Y.all.adjacent`.
- **What felt wrong:** `cpl fmt` breaks the second tuple pattern over four lines instead of breaking
  before the second `for`, so the one-line editorial sum reads as a puzzle.
- **Goal it hurts:** readable layout.
- **Ideal code:** `sum(... for (i, i2) in X.all.adjacent` / `for (j, j2) in Y.all.adjacent if ...)`,
  one clause per line.
- **General fix:** in the formatter, prefer breaking between generator clauses over breaking inside a
  clause's pattern.
- **Severity:** low
- **Owner:** lint-fmt.

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
