# Friction report: ioi18p2

## Dream

```cp
var bad = SegTree<Who, MinCount<Int>>(MinCount.monoid, fill: MinCount.of(0), act: add)
fn arrival(cell: (Int, Int)) -> Cut<Who> = who.get(cell).map(p => p.l) ?? Who.cuts.last!
let touched = Set([n for p in [a, b] for n in grid.around(seat[p], self: true)])
```

## Range add on a min-count tree needs a hand-made action

- **Where:** `let shift = Action<MinCount<Int>, Int>.new(apply: ...)`.
- **What happened:** the `MaxCount` doc says `SegTree(MaxCount.monoid, a, act: add)`, but
  `s.apply(0..1, 5)` is E0302 (`apply` wants a `MaxCount` for `f`), and with `F = Int` the
  check fails inside `Action.of` (it adds `v + f` with `v: MinCount`). Routed to the stdlib owner.
- **General fix:** `Action.of(add)` shifts `.val` for the `*Count` monoids.
- **Severity:** medium (the textbook "count the minimum under range add" tree).

## Grid neighbours are hand-written offsets

- **Where:** `arrival(y - 1, x)` ... and the `(dy, dx)` list in the swap loop.
- **What happened:** the four neighbours of a cell, in-grid or as "never" outside it, go through
  `Int` coordinates and `Row.at` / `Col.at` by hand; the dedup of touched cells is
  `sort` + `dedup` on a list.
- **General fix:** a grid view over `[Row, Col]` with `around(cell, self:)` and an optional
  `get` for off-grid lookups.
- **Severity:** low.

## The C++ is over the size target

`cpl build` warns W0201: 34070 bytes against the 32768-byte target of judge `generic`, so the
emitted C++ drops its comments.

## Measurements

Contestant-written tokens: port 584, original 637 (`corpus/audit.py` count). Emitted C++: 32139 bytes (`cpl build`, judge `generic`).
Max input (h = w = 1000, q = 50000): build + compile + run under 2 s locally; the accepted C++ is grader functions with no main.
