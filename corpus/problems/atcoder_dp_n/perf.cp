// Tuned: plain [0..n, 0..n] cost table filled by explicit loops, so each range's best split
// is a running min instead of a list of split costs.
//
// Source: AtCoder Educational DP Contest N (Slimes), https://atcoder.jp/contests/dp/tasks/dp_n
// limits: 2000 ms, 1024 MB
// Problem: n <= 400 slimes in a row with sizes a_i <= 1e9. Merging two adjacent slimes costs
//   the sum of their sizes and leaves one slime of that size. Print the least total cost to
//   merge everything into one slime.
// Solution: the last merge on a range r joins its two halves at some split, so
//   cost[r] = min over splits (x, y) of cost[x] + cost[y], plus the total size of r.
//   `I.ranges` goes by length, so the halves are done first: O(n^3).
// Bounds: n: 1..=400; a: 1..=1e9
let n: 1..=400 = read()
let a = read<[n]1..=1e9>()
var cost: [0..n, 0..n]Int = 0
for i in (0..n).rev:
    var sum = a[i]
    for j in i + 1..n:
        sum += a[j]
        var best = cost[i, i] + cost[i + 1, j]
        for k in i + 1..j:
            best min= cost[i, k] + cost[k + 1, j]
        cost[i, j] = best + sum
print(cost[0, n - 1])
