// Source: UCRPC F21 G (Snack Attack), https://dmoj.ca/problem/ucrpc21g
// Problem: an n x n grid (n <= 100), a start cell, p <= 1000 popcorn and b <= 1000 boulder drops
//   (cell, time 1..=100). Each second stay or move to a side neighbour; being on a cell when
//   popcorn drops there gains one each, a boulder leaves floor(h/2) of the h held. Print the
//   most popcorn held at the end.
// Solution: simulate tick by tick the set of reachable (cell, held) states: apply the cell's
//   event at that tick, then step to the cell or a neighbour. A per-state stamp of the last
//   tick it was queued dedups the next set without clearing a table: O(ticks * n^2 * p) in
//   the worst case, far less in practice.
// Runnable on proto/cpc today.
// original: corpus/submissions/3944717.cpp
// Bounds: n: 1..=100; coins, bombs: 0..=1000
let n: 1..=100 = read()
let coins: 0..=1000 = read()
let bombs: 0..=1000 = read()
domain Row = 0..n
domain Col = 0..n
domain Tick = 0..105
type Cell = (Row, Col)
let start: Cell = read()
let coin_at: [0..coins](Row, Col, Tick) = read()
let bomb_at: [0..bombs](Row, Col, Tick) = read()

// Coins appearing per (tick, cell); -1 marks a bomb.
var event: [Tick, Row, Col]Int = 0
for (r, c, t) in coin_at: event[t, r, c] += 1
for (r, c, t) in bomb_at: event[t, r, c] = -1

// The reachable (cell, coins held) states, tick by tick. `stamp` holds the
// last tick a state was queued for, which saves clearing a table every tick.
var stamp: [Row, Col, 0..=coins]Int = -1
var cur: List<(Cell, Int)> = [(start, 0)]
for t in Tick:
    var next: List<(Cell, Int)> = []
    for (p, held) in cur:
        let e = event[t, p.0, p.1]
        let have = if e == -1: held / 2 else: held + e
        var targets = p.adj4
        targets.push(p)
        for q in targets:
            if stamp[q.0, q.1, have] != t.idx:
                stamp[q.0, q.1, have] = t.idx
                next.push((q, have))
    cur = next
var best = 0
for (p, held) in cur: best = max(best, held)
print(best)
