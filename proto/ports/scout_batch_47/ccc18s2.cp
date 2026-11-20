// CCC '18 S2 — Sunflowers (DMOJ: ccc18s2)
// original: corpus/submissions/4329456.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given an n x n table rotated by a multiple of 90 degrees, find the unique
// orientation whose rows and columns are all strictly increasing.
// Note: using one square domain N = 0..n lets `g.transposed` keep type `[N, N]Int`
// (distinct Row/Col domains would reject assigning `g.transposed` back to `g`).
let n: 2..=100 = read()
domain N = 0..n
var g: [N, N]Int = read()

fn valid(a: [N, N]Int) -> Bool:
    let rows_ok = all(all(x < y for (x, y) in a.row(r).adjacent) for r in N)
    let cols_ok = all(all(x < y for (x, y) in a.col(c).adjacent) for c in N)
    rows_ok and cols_ok

for _ in 0..4:
    if valid(g):
        for r in N:
            print(" ".join("{g[r, c]}" for c in N))
        return
    g = g.transposed.flipped
