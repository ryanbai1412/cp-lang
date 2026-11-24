// Source: IOI 1994 P1 (The Triangle), https://dmoj.ca/problem/ioi94p1
// limits: 600 ms, 16 MB
// Problem: a number triangle of n <= 100 rows, values 0..99. Print the largest sum on a path
//   from the top to the base, each step going down-left or down-right.
// Solution: row by row, each cell adds its value to the better of the (at most two) cells
//   above it; print the best of the last row. O(n^2).
// original: corpus/submissions/4810274.cpp
// Bounds: n: 1..=100
let n: 1..=100 = read()
// best[c]: the best path sum ending at cell c of the row so far
var best: List<Int> = []
for r in 1..=n:
    let row: [0..r]0..=99 = read()
    best = [x + max(best.get(c - 1) ?? 0, best.get(c) ?? 0) for (c, x) in row.items]
print(best.max()!)
