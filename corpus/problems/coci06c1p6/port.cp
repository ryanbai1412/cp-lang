// Source: COCI 2006/2007 Contest 1 #6 (Debug), https://dmoj.ca/problem/coci06c1p6
// Problem: an R x C grid of 0/1, R, C <= 300. A square killer is a square submatrix of size >= 2
//   equal to its own 180-degree rotation. Print the size of the largest killer, or -1.
// Solution: a 2-D rolling hash of the grid and one of the grid turned 180 degrees, with the same
//   bases. A square is a killer iff its hash equals that of the matching square of the turned
//   grid. Killers are not monotone in size, so test every size and corner: O(R C min(R, C)).
// Agrees with 4183051.cpp on 500 random grids fed by a script (the typed generator
//   cannot make this input).
// Bounds: rows, cols: 1..=300
use std.strings.{Hash2D}

let rows: 1..=300 = read()
let cols: 1..=300 = read()
domain Row = 1..=rows
domain Col = 1..=cols
let g: [Row, Col]Char = read()

fn flip<D>(i: D) -> D = D.nth(D.count - 1 - i.ord)
let bases = (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
let seen = Hash2D<Row, Col>((r, c) => g[r, c].ord, bases:)
let turned = Hash2D<Row, Col>((r, c) => g[flip(r), flip(c)].ord, bases:)

fn killer(r: Row, c: Col, size: Int) -> Bool:
    let (Some(r2), Some(c2)) = (Row.at(r + size - 1), Col.at(c + size - 1)) else: return false
    seen.rect(r..=r2, c..=c2) == turned.rect(flip(r2)..=flip(r), flip(c2)..=flip(c))

let best = (2..=min(rows, cols)).rev.find(size => Row.any(r => Col.any(c => killer(r, c, size))))
print(best ?? -1)
