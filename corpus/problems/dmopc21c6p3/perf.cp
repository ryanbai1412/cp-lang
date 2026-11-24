// Tuned: the grid prints row by row, so it stays in 32 bits; a grid printed whole is
// stored in 64 (sent to Perf).
//
// Source: DMOPC '21 Contest 6 P3, An Art Problem, https://dmoj.ca/problem/dmopc21c6p3
// Problem: an N x M grid (N, M, K <= 1500) of colours c <= 1e9, 0 meaning blank. Every blank
//   pixel within K four-neighbour steps of a coloured one takes the colour of the closest
//   coloured pixel, the lowest colour on a tie. Print the grid after this expansion.
// Solution: one multi-source BFS from every coloured cell, stopping K steps out. The sources
//   enter in colour order, so at every distance the lowest colour claims a cell first and the
//   cell keeps the colour that claimed it. O(NM log NM) for the sort, O(NM) for the BFS.
// original: corpus/submissions/4342005.cpp
// Bounds: n, m: 1..=1500; k: 0..=3000; paint: 0..=1e9

let n: 1..=1500 = read()
let m: 1..=1500 = read()
let k: 0..=3000 = read()
domain Row = 1..=n
domain Col = 1..=m
domain Cell = (Row, Col)
var paint: [Row, Col]0..=1e9 = read()

// Breadth first by rounds from the painted cells in colour order: an unpainted cell takes
// the colour of the first cell to reach it, so its colour also marks it seen.
var front = [p for p in Cell if paint[p] != 0].sorted(by: p => paint[p])
repeat k:
    var next: List<Cell> = []
    for u in front:
        for v in u.adj4 if paint[v] == 0:
            paint[v] = paint[u]
            next.push(v)
    front = next
for r in Row: print([paint[r, c] for c in Col])
