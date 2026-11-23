// Source: CCC 2018 S2 (Sunflowers), https://dmoj.ca/problem/ccc18s2
// Problem: an n x n grid, 2 <= n <= 100, of positive values <= 1e9 is a rotation by a multiple of
//   90 degrees of a table whose rows increase left to right and columns top to bottom. Print the
//   original table.
// Solution: only the original orientation increases both ways: try up to four clockwise rotations,
//   checking every row and column, and print the first that passes. O(n^2).
// original: corpus/submissions/4329456.cpp
let n: 2..=100 = read()
domain R = 0..n
domain C = 0..n

var g: [R, C]Int = 0
for r in 0..n:
    for c in 0..n:
        let v: Int = read()
        g[R(r), C(c)] = v

fn is_valid() -> Bool:
    for r in 0..n:
        for c in 0..n - 1:
            if g[R(r), C(c)] >= g[R(r), C(c + 1)]: return false
    for r in 0..n - 1:
        for c in 0..n:
            if g[R(r), C(c)] >= g[R(r + 1), C(c)]: return false
    true

fn rotate_cw():
    var nxt: [R, C]Int = 0
    for r in 0..n:
        for c in 0..n:
            nxt[R(c), C(n - 1 - r)] = g[R(r), C(c)]
    for r in 0..n:
        for c in 0..n:
            g[R(r), C(c)] = nxt[R(r), C(c)]

for _ in 0..4:
    if is_valid():
        for r in 0..n:
            var row: List<Int> = [g[R(r), C(c)] for c in 0..n]
            print(row)
        break
    rotate_cw()
