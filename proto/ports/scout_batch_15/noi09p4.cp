// NOI 2009 Plants vs Zombies: peel the protection graph, then take a
// max-weight closure of what is left with a min cut.
// original: corpus/submissions/5144326.cpp
use std.flow.{max_closure}

let (n, m): (Int, Int) = read()
domain Cell = 0..n * m
fn cell(r: Int, c: Int) -> Cell = Cell(r * m + c)

var score: [Cell]Int = 0
var protects: [Cell]List<Cell> = []
for r in 0..n:
    for c in 0..m:
        let u = cell(r, c)
        if c + 1 < m: protects[cell(r, c + 1)].push(u)
        let (value, k): (Int, Int) = read()
        score[u] = value
        for _ in 0..k:
            let (x, y): (Int, Int) = read()
            protects[u].push(cell(x, y))

var eatable: [Cell]Bool = false
for u in peel(v => protects[v]): eatable[u] = true

let profit = [Cell] u => if eatable[u]: score[u] else: 0
let needs = [(u: u, v: v) for u in Cell if eatable[u] for v in protects[u] if eatable[v]]
print(max_closure(profit, needs).value)
