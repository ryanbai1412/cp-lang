// Source: Codeforces 2165A (Cyclic Merging), https://codeforces.com/contest/2165/problem/A
// limits: 2000 ms, 256 MB
// Problem: n <= 2e5 values in 0..1e9 on a ring. Merging two adjacent x, y leaves max(x, y) and
//   costs max(x, y); print the minimum total cost of n - 1 merges. Multitest, sum n <= 2e5.
// Solution: each of the n ring edges (x, y) costs at least max(x, y) once, except that one edge
//   at the global maximum is never paid; merging the smallest element into its smaller
//   neighbour first achieves it. Answer: sum of max over cyclic adjacent pairs minus max(a), O(n).

@!multitest
let n: 2..=200_000 = read()
let a: [0..n]0..=10 ** 9 = read()
print(sum(max(x, y) for (x, y) in cyclic_pairs(a)) - a.max()!)
