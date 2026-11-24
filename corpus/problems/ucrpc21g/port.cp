// Source: UCRPC F21 G (Snack Attack), https://dmoj.ca/problem/ucrpc21g
// limits: 1000 ms, 256 MB
// Problem: an n x n grid (n <= 100), a start cell, p <= 1000 popcorn and b <= 1000 boulder drops
//   (cell, time 1..=100). Each second stay or move to a side neighbour; being on a cell when
//   popcorn drops there gains one each, a boulder leaves floor(h/2) of the h held. Print the
//   most popcorn held at the end.
// Solution: both events are monotone in the popcorn held (h + k and floor(h / 2)), so more held
//   at the same cell and second never hurts. Keep the most held per cell, None while unreachable:
//   each second take the best of the cell and its side neighbours, then apply that second's
//   drops. O(100 n^2 + p + b).
// original: corpus/submissions/3944717.cpp
// Bounds: n: 1..=100; coins, bombs: 0..=1000; start.0, popcorn.0, boulders.0: 0..=n - 1; start.1, popcorn.1, boulders.1: 0..=n - 1;
//   popcorn.2, boulders.2: 1..=100
let (n, coins, bombs): (1..=100, 0..=1000, 0..=1000) = read()
domain Row = 0..n
domain Col = 0..n
domain Tick = 1..=100
type Cell = (Row, Col)
let start: Cell = read()
let popcorn: [0..coins](Row, Col, Tick) = read()
let boulders: [0..bombs](Row, Col, Tick) = read()

var gain: [Tick, Row, Col]Int = 0
var crush: [Tick, Row, Col]Bool = false
for (r, c, t) in popcorn: gain[t, r, c] += 1
for (r, c, t) in boulders: crush[t, r, c] = true

var held: [Row, Col]?Int = None
held[start] = Some(0)
for t in Tick:
    let moved = [Row, Col] p => max(h for q in p.adj4 + [p] if let h = held[q])
    held = [Row, Col] p => moved[p].map(
        h => if crush[t, p.0, p.1]: h / 2 else: h + gain[t, p.0, p.1]
    )
print([held[r, c] for r in Row for c in Col].present().max()!)
