# oly20practice139 — Bob's Function

## std `MonotoneHull` overflows, so the port keeps its own upper hull
- **Where:** `port.cp`, `struct Upper` (hull lines with the first `t` each wins from).
- **Dream:** a `MonotoneHull(max)` per sqrt block (or, better, one kinetic segment tree; the Dream below).
- **What felt wrong:** `MonotoneHull.covered` cross-multiplies an intercept difference (up to 1.5e14 here) by a slope difference (up to 1e9), which traps R0202 under `cpl stress` at n = 2000. Repro: three lines `Line(1e6, 1e14)`, `Line(2e9, 0)`, `Line(3e9, -1e14)` under `max`. Reported to Core for the stdlib owner. A crossing by `ceil_div` (what `Upper.add` does) stays in range.
- **Goal it hurts:** hard-to-get-wrong, reuse.
- **General fix:** compare crossings in `I128` (or by the integer crossing point) inside `MonotoneHull`; then swap `Upper` for it.
- **Severity:** medium.

## No kinetic segment tree for max of v * (count + prefix add)
- **Where:** the sqrt blocks (`Blk`, `pending`, `rebuild`, `join`).
- **Dream:** `KineticSegTree<Y>(max, lines: ...)` with `shift(..=y, by: 1)` and `max()`, O(log^2 n) amortized.
- **Goal it hurts:** express, speed (O(n sqrt n) instead of polylog).
- **General fix:** a std kinetic segment tree (lines per leaf, range add to `t`, max query).
- **Severity:** low.

## `max` of a non-empty domain's generator is `?Int`; a run's `first` is optional
- **Where:** `max(... for b in Blk)!` and `run.first!.0`.
- **What felt wrong:** `Blk` and each `run` are non-empty by construction, but the types don't know it.
- **General fix:** a non-empty slice type from `runs` (so `first` is total).
- **Severity:** low.

## Old port no longer compiled: an `Int` read from a list is "lent"
- **Where:** the v0.2 port's `let tx = h_at[bi][last]` then `h_at[bi].pop()`: E0602. Reported to Core with yac1p2's repro.
- **Severity:** medium.

## Dream

The ungated ideal (it does not compile today, so it is kept here rather than as `dream.cp`):

```cp
// Source: DMOJ OLY 2020 Practice 139 (Bob's Function), https://dmoj.ca/problem/oly20practice139
// dream: sweep a over the sorted x, a kinetic max-segment-tree keeps max over y-values v of
//   v * (pairs joined with y >= v), and each joined pair is one prefix add
let n = read<1..=150_000>()
let pts = read<[n](x: 0..=1e9, y: 0..=1e9)>().sorted(by: .x)
let (domain Y, ys) = compress(pts.map(.y))
// each v has the line v * t + v * 0; adding 1 on a prefix of Y moves t there
var gain = KineticSegTree<Y>(max, lines: [Y] v => Line(ys[v], 0))
var best = 0
var left = n
for run in pts.runs(by: .x):
    best max= left * run.first.x + gain.max()
    for p in run: gain.shift(..=ys.index_of(p.y), by: 1)
    left -= run.len
print(best max gain.max())
```
