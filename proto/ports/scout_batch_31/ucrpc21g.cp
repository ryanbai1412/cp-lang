// UCRPC 2021 G: walk an n x n board for 105 ticks, each tick staying or
// stepping to a side neighbour. Standing on a cell at the tick a coin appears
// there collects it; standing on a bomb's cell at its tick halves the coins
// held (a bomb beats coins on the same cell and tick). Most coins at the end.
// original: corpus/submissions/3944717.cpp
let (n, coins, bombs): (1..=100, 0..=1000, 0..=1000) = read()
domain Row = 0..n
domain Col = 0..n
domain Tick = 0..105
type Cell = (Row, Col)
let start: Cell = read()
let coin_at: [0..coins](at: Cell, t: Tick) = read()
let bomb_at: [0..bombs](at: Cell, t: Tick) = read()

var gain: [Tick, Cell]Int = 0
var bomb: [Tick, Cell]Bool = false
for e in coin_at: gain[e.t, e.at] += 1
for e in bomb_at: bomb[e.t, e.at] = true

// The reachable (cell, coins held) states, tick by tick.
var seen: [Cell, 0..=coins]Bool = false
var cur: List<(Cell, Int)> = [(start, 0)]
for t in Tick:
    seen.fill(false)                          // FRICTION F1: O(table) per tick; see backlog
    var next: List<(Cell, Int)> = []
    for (p, held) in cur:
        let have = if bomb[t, p]: held / 2 else: held + gain[t, p]
        for q in chain([p], p.adj4):
            if not seen[q, have]:
                seen[q, have] = true
                next.push((q, have))
    cur = next
print(max(held for (_, held) in cur))
