// Source: IOI 1994 P1 (The Triangle), https://dmoj.ca/problem/ioi94p1
// Problem: a number triangle of n <= 100 rows, values 0..99. Print the largest sum on a path
//   from the top to the base, each step going down-left or down-right.
// Solution: row by row, best[c] = x + max(best of the two cells above), missing ones counting
//   as 0; print the max of the last row. O(n^2).
let n: 1..=100 = read()
var best: List<Int> = []  // best path sum ending at each cell of the row above
for r in 1..=n:
    var row: List<Int> = []
    for c in 0..r:
        let x: 0..=99 = read()
        let left = if c > 0: best[c - 1] else: 0
        let right = if c < r - 1: best[c] else: 0
        row.push(x + max(left, right))
    best = row
print(best.collect().max()!)
