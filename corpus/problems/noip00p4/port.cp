// Source: NOIP 2000 Senior P4 (Square Number Picking), https://dmoj.ca/problem/noip00p4
//   (not public on DMOJ; statement from Luogu P1004)
// Problem: an N x N grid (1 <= N <= 9) with numbers 1..=30 in some cells (any of them), given
//   as lines x y v (1 <= x, y <= N) up to a line holding a single 0. Walk twice from the
//   top-left to the bottom-right corner moving only down or right; a cell's number is taken
//   the first time it is passed. Print the largest total taken.
// Solution: walk both paths in lockstep. After k steps each walker is on anti-diagonal k, so
//   the rows (a, b) fix both cells, and the walkers share a cell exactly when a == b, taking
//   its number once. best[a, b] over diagonal k is the best of the four predecessor row pairs
//   on diagonal k - 1 plus the cells' numbers; row 0 is a border no walk reaches. O(N^3).
// original: corpus/submissions/5067920.cpp
let n: 1..=9 = read()
var grid: [1..=n, 1..=n]0..=30 = 0
while true:
    let x: 0..=n = read()
    if x == 0: break
    let y: 1..=n = read()
    let v: 1..=30 = read()
    grid[x, y] = v

const UNREACHED = -1
var best: [0..=n, 0..=n]Int = UNREACHED
best[1, 1] = grid[1, 1]
for k in 1..=2 * n - 2:
    var next: [0..=n, 0..=n]Int = UNREACHED
    let rows = max(1, k - n + 2)..=min(n, k + 1)
    for a in rows:
        for b in rows:
            let stay = max(best[a, b], best[a - 1, b - 1])
            let before = max(stay, max(best[a - 1, b], best[a, b - 1]))
            let here = grid[a, k + 2 - a] + (if a == b: 0 else: grid[b, k + 2 - b])
            next[a, b] = before + here
    best = next
print(best[n, n])
