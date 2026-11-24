# Friction report: egoi22p4

Rethought. The v0.2 port kept a treap per city in flat parallel arrays with a `-1` sentinel and its own LCG for priorities, plus a hand-rolled ODT over a max-SegTree. The v0.3 port needs neither. Each city keeps a running event total; an `IntervalMap` of runs `(city, since)` (the city total when the run arrived) repaints a move with `on_old` settling the overwritten runs into a range-add `DualFenwick`; a query is `settled.get(v) + events[city] - since`. No sentinels; typed reads.

No language gap. `DualFenwick` needs `use std.fenwick.{DualFenwick}` (not in the prelude), while `Fenwick` is.

## Measurements

gen.py --max (n = m = q = 2e5, caterpillar): port 0.19 s / 36 MB, v0.2 port 0.40 s / 46 MB, accepted C++ 0.13 s but only with an unlimited stack (segfaults at 8 MB; 0.66 s unlimited).
