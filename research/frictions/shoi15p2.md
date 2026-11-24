# Friction report: shoi15p2

There is no dream.cp; the ideal is below. The port is a lazy `SegTree` of hole runs under a paint-holes-or-tissue `Action<Run, ?Bool>`. A transplant counts the moved tissue, digs out the source and uses `right_while` to find where the holes from l1 reach that count.

## Dream

```cp
let short = brain.right_while(to_l.l, s => s.holes < moved)   // a Cut<Cell>
brain.apply(to_l.l..min(short.r, to_r.r), Some(false))
```

## Mixing a cut and a cell goes through ordinals
- **Where:** line 48: `to_l..=Cell.nth(min(short.ord, to_r.ord))`.
- **What felt wrong:** `right_while` returns a `Cut<Cell>` and the bound is a `Cell`. The port takes both ordinals, `min`s them as `Int`s and rebuilds a `Cell`. This review tried `min(short, to_r)`, which is rejected (E0302: `Cell.cuts` against `Cell`), as it should be.
- **Goal it hurts:** express.
- **Ideal code:** the dream's cut-based range, with `short.r` as the cut after the cell at that cut. That needs `Cut` and `Cell` conversions in both directions, plus `min` on cuts.
- **General fix:** a std "cut just after this cell" accessor (`Cell.r` exists) and a "cell just after this cut" accessor (`Cut.next_cell`), so that a search result can bound a range without ordinals. Owner: stdlib.
- **Severity:** low.

## Fixed with the scored review

`solid` was an `= if` body that fmt split into a column of arguments; it is now a block body, `if hole: Run(len, 0, len, len, len) else: Run(0, len, 0, 0, 0)`, which fmt keeps on one line. fmt should prefer a block body over splitting an inner call (Owner: fmt).
- **Owner:** idioms (done).

## Measurements

- **Tokens:** port 449, accepted C++ 1217 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.25 s / 17 MB, accepted C++ 0.18 s / 21 MB, with the same output.
- **Emitted C++:** 30564 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
