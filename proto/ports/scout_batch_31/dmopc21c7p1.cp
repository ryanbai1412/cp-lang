// DMOPC '21 Contest 7 P1: an n x m grid of positive numbers with blanks (0).
// Fill the blanks so every row and every column is strictly increasing, with
// all values in 1..=1e18, or print -1. Filling from the bottom-right, each
// blank takes one less than the smaller of the cells below and to its right,
// the largest value it could hold.
// original: corpus/submissions/4452959.cpp
let (n, m): (1..=1000, 1..=1000) = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
var a: [Row, Col]?Int = read(none: "0")
const TOP = 1e17                              // room below for n + m blanks

for p in Cell.rev:
    if a[p] == none:
        var v = TOP
        for d in [Dir4.S, Dir4.E]:
            if let q = p.step(d): v min= a[q]!
        a[p] = v - 1

fn rises(p: Cell, d: Dir4) -> Bool: p.step(d).map(q => a[p]! < a[q]!) ?? true
if all(a[p]! in 1..=1e18 and rises(p, Dir4.S) and rises(p, Dir4.E) for p in Cell):
    for r in Row: print(a.row(r))
else:
    print(-1)
