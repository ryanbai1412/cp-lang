// Source: IOI 2006 (Pyramid), https://dmoj.ca/problem/ioi06p2
// limits: 600 ms, 64 MB
// Problem: an m x n grid (m, n <= 1000) of heights <= 100. Place an a x b pyramid base and,
//   inside it with a wall at least 1 thick, a c x d chamber, maximising the base's sum minus
//   the chamber's. Print the column and row of each one's upper-left corner.
// Solution: 2D prefix sums give every block sum. For a base, the chamber's corner lies in a
//   fixed window, so the best chamber is a sliding-window minimum of chamber sums, first along
//   rows and then down columns, each a `window_fold(k, min)`: O(mn).
// original: corpus/submissions/5034372.cpp
// Bounds: w, h: 3..=1000; bw: 3..=w; bh: 3..=h; cw: 1..=bw - 2; ch: 1..=bh - 2; height: 1..=100
let w: 3..=1000 = read()
let h: 3..=1000 = read()
let bw: 3..=w = read()
let bh: 3..=h = read()
let cw: 1..=bw - 2 = read()
let ch: 1..=bh - 2 = read()
domain Row = 1..=h
domain Col = 1..=w
let height: [Row, Col]1..=100 = read()
let pre = prefix(height)

// The sum of the rows x cols block whose upper-left cell is (r + 1, c + 1).
fn block(r: Int, c: Int, rows: Int, cols: Int) -> Int:
    pre.fold(Row(r + 1)..=Row(r + rows), Col(c + 1)..=Col(c + cols))

let chamber = [[block(r, c, ch, cw) for c in 0..=w - cw] for r in 0..=h - ch]
// down[j][i]: the least chamber with its corner in the base's window starting at (i, j).
let along = [row.window_fold(bw - cw - 1, min).collect() for row in chamber]
let down = [[row[j] for row in along].window_fold(bh - ch - 1, min).collect() for j in 0..along[0].len]

let (_, r, c) = [(block(r, c, bh, bw) - down[c + 1][r + 1], r, c) for r in 0..=h - bh for c in 0..=w - bw].max()!
let (_, i, j) = [(chamber[i][j], i, j) for i in r + 1..r + bh - ch for j in c + 1..c + bw - cw].min()!
print(c + 1, r + 1)
print(j + 1, i + 1)
