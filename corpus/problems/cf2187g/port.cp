// Source: Codeforces 2187G (Many Cartesian Trees), https://codeforces.com/contest/2187/problem/G
// limits: 2000 ms, 256 MB
// Problem: for a permutation q (n <= 3000), p_1 = q and each p_i adds n to the minimum of
//   p_(i-1). Given the n x n 0/1 matrix of S, the union of the (child, parent) edges of the
//   Cartesian trees of p_1..p_n, print any q that gives S. Multitest, sum of n^2 <= 9e6.
// Solution: S only sees the cyclic order of the values, so put value 1 at position 0 and
//   hand out values 2, 3, ... by walking: from the position cur just given a value, go to
//   p0, its leftmost parent in S, when cur is p0's rightmost right child in S, else to its
//   rightmost parent. Each step scans a row: O(n^2).

@!multitest
let n: 1..=3_000 = read()
let s: [0..n, 0..n]Char = read()
let ch1 = [0..n] i => first(j for j in (i + 1..n).rev if s[j, i] == '1')

var ans: [0..n]Int = 0
var cur = 0
for v in 1..=n:
    ans[cur] = v
    let pa0 = first(j for j in 0..cur if s[cur, j] == '1')
    let pa1 = first(j for j in (cur + 1..n).rev if s[cur, j] == '1')
    if let p0 = pa0 and ch1[p0] == cur: cur = p0
    elif let p1 = pa1: cur = p1
print(ans)
