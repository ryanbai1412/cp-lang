// Source: COCI 2006/2007 Contest 1 #6 (Debug), https://dmoj.ca/problem/coci06c1p6
// Problem: an R x C grid of 0/1, R, C <= 300. A square killer is a square submatrix of size >= 2
//   equal to its own 180-degree rotation. Print the size of the largest killer, or -1.
// Solution: 2-D polynomial hash: cell (r, c) weighs (digit + 1) * 131^r * 137^c mod 1e9+7, with
//   prefix sums over the grid and over the grid rotated by 180 degrees. A square is a killer iff
//   its hash equals that of the matching square of the rotated grid, both shifted to the same
//   powers. Killers are not monotone in size, so test every size and corner: O(R C min(R, C)).
// Agrees with 4183051.cpp on 500 random grids fed by a script (the typed generator
//   cannot make this input).
const P = 1000000007
let rows: 1..=300 = read()
let cols: 1..=300 = read()
domain Row = 1..=rows
domain Col = 1..=cols
let g: [Row, Col]Char = read()

var row_pow: [Row]Int = 1
var col_pow: [Col]Int = 1
for r in Row:
    if let up = Row.at(r - 1): row_pow[r] = row_pow[up] * 131 % P
for c in Col:
    if let left = Col.at(c - 1): col_pow[c] = col_pow[left] * 137 % P
fn flip<D>(i: D) -> D = D.nth(D.count - 1 - i.ord)
fn weight(r: Row, c: Col, ch: Char) -> Int: (ch - '0' + 1) * row_pow[r] % P * col_pow[c] % P
let seen = prefix([Row, Col] (r, c) => weight(r, c, g[r, c]))
let turned = prefix([Row, Col] (r, c) => weight(r, c, g[flip(r), flip(c)]))

fn killer(r: Row, c: Col, size: Int) -> Bool:
    if let r2 = Row.at(r + size - 1) and let c2 = Col.at(c + size - 1):
        let here = seen.fold(span(r, r2), span(c, c2)) % P
        let there = turned.fold(span(flip(r2), flip(r)), span(flip(c2), flip(c))) % P
        return here * row_pow[flip(r2)] % P * col_pow[flip(c2)] % P == there * row_pow[r] % P * col_pow[c] % P
    false
let best = [size for size in 2..=min(rows, cols) if any(killer(r, c, size) for r in Row for c in Col)].last
print(best ?? -1)
