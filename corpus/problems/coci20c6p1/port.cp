// COCI '20 Contest 6 P1: every '#' on the picture is the top-left corner of
// a 2 x 2 stamp. Print the stamped picture.
// Runnable on proto/cpc today.
// original: corpus/submissions/4188008.cpp
let n: 1..=100 = read()
let m: 1..=100 = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()
var out: [Row, Col]Char = g
for r in Row:
    for c in Col:
        if g[r, c] != '#': continue
        if let c2 = c + 1: out[r, c2] = '#'
        if let r2 = r + 1:
            out[r2, c] = '#'
            if let c2 = c + 1: out[r2, c2] = '#'
for r in Row: print(out[r, ..])
