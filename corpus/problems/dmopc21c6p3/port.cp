// DMOPC '21 contest 6 P3, An Art Problem: every painted cell spreads its
// colour to cells at most k steps away. A blank cell takes the colour that
// reaches it first, and the smaller colour on a tie. Print the grid.
// original: corpus/submissions/4342005.cpp
let n: 1..=1500 = read()
let m: 1..=1500 = read()
let k: 0..=3000 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
var paint: [Row, Col]0..=1e9 = read()

const DIRS = [(-1, 0), (1, 0), (0, -1), (0, 1)]
fn step(p: Cell, d: (Int, Int)) -> ?Cell:  // WISH L1: p.step(d) over Dir4
    if let r = p.0 + d.0 and let c = p.1 + d.1:
        return (r, c)
    return none

// Smaller colours start first, so they win every tie.
var wave: List<(Int, Cell)> = sorted(
    [(paint[r, c], (r, c)) for r in Row for c in Col if paint[r, c] != 0]
)
for round in 0..k:  // WISH L7: bfs from many sources, in a given order
    if wave.len == 0: break
    var grown: List<(Int, Cell)> = []
    for (colour, p) in wave:
        for d in DIRS:
            if let q = step(p, d):
                if paint[q] == 0:
                    paint[q] = colour
                    grown.push((colour, q))
    wave = grown
print(paint)
