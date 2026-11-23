// COCI 2010 contest 7 #4, Poštar: a postman starts at the post office (P)
// and must reach every house (K) on a hilly square town, moving to any of
// the eight cells around him. Print the smallest gap between the highest
// and lowest ground he has to stand on.
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
