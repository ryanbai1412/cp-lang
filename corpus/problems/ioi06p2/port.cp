// Source: IOI 2006 (Pyramid), https://dmoj.ca/problem/ioi06p2
// Problem: an m x n grid (m, n <= 1000) of heights <= 100. Place an a x b pyramid base and,
//   inside it with a wall at least 1 thick, a c x d chamber, maximising the base's sum minus
//   the chamber's. Print the column and row of each one's upper-left corner.
// Solution: 2D prefix sums give every block sum. For a base, the chamber's corner lies in a
//   fixed window, so the best chamber is a sliding-window minimum of chamber sums, first along
//   rows and then down columns, each with a monotone deque: O(mn).
// Note: as it builds today, blocks are four plain numbers and the sliding minimum is a
//   hand-kept deque over plain lists (K14).
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

fn block(r: Int, c: Int, rows: Int, cols: Int) -> Int:
    pre.fold(Row(r + 1)..=Row(r + rows), Col(c + 1)..=Col(c + cols))

fn window_mins(xs: List<Int>, k: Int) -> List<Int>:
    var out: List<Int> = []
    var keep: List<Int> = []
    var head = 0
    for i in 0..xs.len:
        while keep.len > head and xs[keep.last!] >= xs[i]: keep.pop()
        keep.push(i)
        if keep[head] <= i - k: head += 1
        if i >= k - 1: out.push(xs[keep[head]])
    out

let chamber = [[block(r, c, ch, cw) for c in 0..=w - cw] for r in 0..=h - ch]
let along = [window_mins(row, bw - cw - 1) for row in chamber]
let down = [window_mins([along[i][j] for i in 0..along.len], bh - ch - 1) for j in 0..along[0].len]

let (_, r, c) = [(block(r, c, bh, bw) - down[c + 1][r + 1], r, c) for r in 0..=h - bh for c in 0..=w - bw].max()!
let (_, i, j) = [(chamber[i][j], i, j) for i in r + 1..r + bh - ch for j in c + 1..c + bw - cw].min()!
print(c + 1, r + 1)
print(j + 1, i + 1)
