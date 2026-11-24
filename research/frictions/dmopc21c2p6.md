# Friction report: dmopc21c2p6

No semantic gap. Query 2 is `parallel_bsearch` over the values, largest first, in place of a hand-written recursive split; `p.inverse()` gives each value's place; the queries are typed (`Val`, `Ask`) and answers are `?Int` per query in place of a print order.

## Friction: positions from 0

`Pos = 0..n` keeps the shift by k passes as plain arithmetic (`Pos(i - k)`, `Pos.cuts.at(c)`); with `1..=n` every shifted index would need a label offset. Printing a position is then `place + 1`.

## Measurements

gen.py --max (N = Q = 3e5): port 0.42 s, accepted C++ 0.59 s.
