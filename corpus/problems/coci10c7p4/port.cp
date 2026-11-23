// Source: COCI 2010/2011 Contest 7 #4, Postar, https://dmoj.ca/problem/coci10c7p4
// Problem: an N x N town (N <= 50) of cells P (one post office), K (houses, at least one) and
//   '.', each with a height < 1e6. Starting from P and moving to any of the 8 neighbours,
//   visit every K and return. Print the smallest possible (highest - lowest) height visited.
// Solution: Two pointers over the sorted heights: raising the floor never lowers the needed
//   ceiling. For each floor, raise the ceiling until a flood fill from P through cells with
//   heights inside [floor, ceiling] reaches every K. O(N^2) window steps, each an O(N^2)
//   fill: O(N^4).
// original: corpus/submissions/5245379.cpp
let n: 2..=50 = read()
domain Row = 1..=n
domain Col = 1..=n
type Cell = (Row, Col)
let town: [Row, Col]Char = read()
let high: [Row, Col]1..=1e6 = read()

fn cells() -> List<Cell>: [(r, c) for r in Row for c in Col]  // WISH L5: `for p in Cell`

fn around(p: Cell) -> List<Cell>:  // WISH L1: p.around() over Dir8
    var out: List<Cell> = []
    for dr in -1..=1:
        for dc in -1..=1:
            if let r = Row.at(p.0 + dr) and let c = Col.at(p.1 + dc): out.push((r, c))
    out

// Can he deliver everything standing only on ground between lo and hi?
fn delivers(lo: Int, hi: Int) -> Bool:
    var seen: [Row, Col]Bool = false  // WISH L7: let seen = reach(adj, from: town.where('P'))
    var todo: List<Cell> = []
    for p in cells():
        if town[p] == 'P' and lo <= high[p] and high[p] <= hi:
            seen[p] = true
            todo.push(p)
    while let p = todo.pop():
        for q in around(p):
            if not seen[q] and lo <= high[q] and high[q] <= hi:
                seen[q] = true
                todo.push(q)
    all(seen[p] for p in cells() if town[p] == 'K')

// Raising the floor never lowers the ceiling it needs, so both only move up.
let levels = sorted([high[p] for p in cells()])
var best = levels.last! - levels[0]
var top = 0
for floor in 0..levels.len:  // WISH L8: for (lo, hi) in levels.stretches(delivers)
    top max= floor           // with no house every stretch delivers, the empty ones too
    while top < levels.len and not delivers(levels[floor], levels[top]): top += 1
    if top == levels.len: break
    best min= levels[top] - levels[floor]
print(best)
