// NOI 2009 Plants vs Zombies (DMOJ: noi09p4): grid of n rows by m columns.
// Cell (r, c) has score s (may be negative) and protects k other cells.
// Zombies enter from the right (column m - 1), so (r, c + 1) must be eaten
// before (r, c); likewise every protector u must be eaten before each cell v
// it protects (`protects[u]` contains `v`).
//
// 1) Peel `protects` from in-degree 0 (`peel`): any cell on or behind a
//    directed cycle can never be reached and is discarded.
// 2) Among the remaining eatable cells, eating a protected cell `v` forces
//    eating its protector `u` (`needs` edge `(u: v, v: u)`), so the answer is
//    `max_closure(profit, needs).value`.
use std.flow.{max_closure}

let (n, m): (1..=20, 1..=30) = read()
domain Row = 0..n
domain Col = 0..m
domain Cell = (Row, Col)

var score: [Cell]Int = 0
var protects: [Cell]List<Cell> = []
for r in Row:
    for c in Col:
        let u = Cell(r, c)
        if let right = c.next: protects[Cell(r, right)].push(u)
        let (value, targets): (Int, List<Cell>) = read()
        score[u] = value
        protects[u].extend(targets)

var eatable: [Cell]Bool = false
for u in peel<Cell>(v => protects[v]): eatable[u] = true

let profit = [Cell] u => if eatable[u]: score[u] else: 0
let needs = [(u: v, v: u) for u in Cell if eatable[u] for v in protects[u] if eatable[v]]
print(max_closure(profit, needs).value)
