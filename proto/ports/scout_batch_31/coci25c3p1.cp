// COCI '25 Contest 3 P1: a brush starts at the centre of an n x n canvas.
// L R U D move it one cell (it stays put at an edge); any other letter
// paints, in that colour, every cell at Manhattan distance below k.
// original: corpus/submissions/7730233.cpp
let (n, k): (1..=100, 1..=200) = read()
let moves: Str = read()
domain Row = 0..n
domain Col = 0..n
type Cell = (Row, Col)
var g: [Row, Col]Char = '.'
var at: Cell = (Row(n / 2), Col(n / 2))
for ch in moves:
    if let d = Dir4.from("UDLR", ch):
        at = at.step(d) ?? at
    else:
        for p in Cell:
            if abs(p.0 - at.0) + abs(p.1 - at.1) < k: g[p] = ch
for r in Row: print(g.row(r))
