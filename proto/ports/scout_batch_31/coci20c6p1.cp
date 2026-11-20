// COCI '20 Contest 6 P1: every '#' on the picture is the top-left corner of
// a 2 x 2 stamp. Print the stamped picture.
// original: corpus/submissions/4188008.cpp
let (n, m): (1..=100, 1..=100) = read()
domain Row = 0..n
domain Col = 0..m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()
var out = g
for (r, c) in Cell:
    if g[r, c] == '#':
        for r2 in (r..=r + 1).clamp(Row):
            for c2 in (c..=c + 1).clamp(Col): out[r2, c2] = '#'
for r in Row: print(out.row(r))
