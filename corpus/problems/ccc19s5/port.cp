// Source: CCC 2019 S5 (Triangle: The Data Structure), https://dmoj.ca/problem/ccc19s5
// Problem: a triangle of numbers with n <= 3000 rows (row i has i entries, 0 <= a <= 1e9) and
//   1 <= k <= n. Print the sum, over every sub-triangle of side k, of its largest entry.
// Solution: best[i, j] is the max of the side-s triangle with apex (i, j). The side-s triangles
//   at (i, j), (i + d, j) and (i + d, j + d), d = s' - s, cover the side-s' one for any
//   s' <= (3s + 1) / 2, so s grows by half each round, updated in place: O(n^2 log k).
//   Then sum best over the apexes of the side-k triangles.
let n: 1..=3000 = read()
let k: 1..=n = read()

var best: [0..n, 0..n]Int = 0
for i in 0..n:
    for j in 0..=i: best[i, j] = read()

var side = 1
while side < k:
    let grown = min(k, (side * 3 + 1) / 2)
    let step = grown - side
    for i in 0..=n - grown:
        for j in 0..=i: best[i, j] = max(best[i, j], best[i + step, j], best[i + step, j + step])
    side = grown
print(sum(best[i, j] for i in 0..=n - k for j in 0..=i))
