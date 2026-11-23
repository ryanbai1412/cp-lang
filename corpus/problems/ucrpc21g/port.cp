// UCRPC 2021 G: walk an n x n board for 105 ticks, each tick staying or
// stepping to a side neighbour. Standing on a cell at the tick a coin appears
// there collects it; standing on a bomb's cell at its tick halves the coins
// held (a bomb beats coins on the same cell and tick). Most coins at the end.
// Runnable on proto/cpc today.
// original: corpus/submissions/3944717.cpp
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

fn adj4(p: Cell) -> List<Cell>:  // v2: p.adj4
    var out: List<Cell> = []
    if let r = p.0 - 1: out.push((r, p.1))
    if let r = p.0 + 1: out.push((r, p.1))
    if let c = p.1 - 1: out.push((p.0, c))
    if let c = p.1 + 1: out.push((p.0, c))
    out

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
        var targets = adj4(p)
        targets.push(p)
        for q in targets:
            if stamp[q.0, q.1, have] != t.idx:
                stamp[q.0, q.1, have] = t.idx
                next.push((q, have))
    cur = next
var best = 0
for (p, held) in cur: best = max(best, held)
print(best)
