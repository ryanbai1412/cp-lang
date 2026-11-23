// Source: COCI '20 Contest 6 #1 (Bold), https://dmoj.ca/problem/coci20c6p1
// Problem: an n x m picture of '.' and '#' (2 <= n, m <= 100; no '#' in the last row or
//   column). Bolding replaces every '#' by a 2 x 2 square of '#' reaching right and down;
//   print the bolded picture.
// Solution: copy the picture and, for each '#' of the original, also mark the cells to
//   its right, below and below-right (bounds checked with at). O(nm).
// Runnable on proto/cpc today.
// original: corpus/submissions/4188008.cpp
let n: 1..=100 = read()
let m: 1..=100 = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()
var out: [Row, Col]Char = g.copy()
for r in Row:
    for c in Col:
        if g[r, c] != '#': continue
        if let c2 = Col.at(c + 1): out[r, c2] = '#'
        if let r2 = Row.at(r + 1):
            out[r2, c] = '#'
            if let c2 = Col.at(c + 1): out[r2, c2] = '#'
for r in Row: print(out[r, ..])
