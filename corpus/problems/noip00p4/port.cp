// Source: NOIP 2000 Senior P4 (Square Number Picking), https://dmoj.ca/problem/noip00p4
//   (not public on DMOJ; statement from Luogu P1004)
// Problem: an N x N grid (1 <= N <= 9) with numbers 1..=30 in some cells (any of them), given
//   as lines x y v (1 <= x, y <= N) up to a line holding a single 0. Walk twice from the
//   top-left to the bottom-right corner moving only down or right; a cell's number is taken
//   the first time it is passed. Print the largest total taken.
// Solution: walk both paths in lockstep. After k steps each walker is on anti-diagonal k, so
//   the rows (a, b) fix both cells, and the walkers share a cell exactly when a == b, taking
//   its number once. best[a, b] over diagonal k is the best of the (up to four) reached
//   predecessor row pairs on diagonal k - 1 plus the cells' numbers. O(N^3).
// original: corpus/submissions/5067920.cpp
let n: 1..=9 = read()
domain Row = 1..=n
var grid: [Row, Row]0..=30 = 0
while true:
    let x: 0..=n = read()
    if x == 0: break
    let y: 1..=n = read()
    let v: 1..=30 = read()
    grid[Row(x), Row(y)] = v

// best[a, b]: the most taken with the walkers on rows a and b of the current diagonal
var best: [Row, Row]?Int = None
best[Row.first!, Row.first!] = grid[Row.first!, Row.first!]
for k in 1..=2 * n - 2:
    var next: [Row, Row]?Int = None
    let rows = Row.at(max(1, k - n + 2))!..=Row.at(min(n, k + 1))!
    for a in rows:
        for b in rows:
            var before: ?Int = None
            for pa in [a.prev, Some(a)]:
                for pb in [b.prev, Some(b)]:
                    if let x = pa and let y = pb: before max= best[x, y]
            let col = (r: Row) => Row(k + 2 - Int(r))
            let here = grid[a, col(a)] + (if a == b: 0 else: grid[b, col(b)])
            if let got = before: next[a, b] = got + here
    best = next
print(best[Row.last!, Row.last!]!)
