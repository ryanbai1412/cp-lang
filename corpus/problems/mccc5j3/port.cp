// Source: Mock CCC '23 Contest 1 J3 (Pairing Gifts), https://dmoj.ca/problem/mccc5j3
// Problem: n <= 2e5 distinct prices a_i, n distinct prices b_j, all in 1..=M (M <= 1e18), and
//   0 <= K < n. Print the most disjoint pairs (i, j) with a_i + b_j = M and |i - j| >= K.
// Solution: the a_i are distinct, so b_j has at most one partner a_i = M - b_j, and the b_j are
//   distinct, so the good pairs are already disjoint: count them. A Map takes each value of a
//   to its position; for each j look up M - b_j and check |i - j| >= K. O(n).
// Bounds: n: 1..=2e5; k: 0..=199_999; m: 1..=1e18; a, b: 1..=m
// original: corpus/submissions/5333116.cpp
let (n, k, m): (1..=200_000, 0..=199_999, 1..=1e18) = read()
assert k < n
domain I = 0..n
let a: [I]1..=m = read()
let b: [I]1..=m = read()

var at = Map<Int, I>()
for i in I: at[a[i]] = i
fn good(j: I) -> Bool =
    if let i = at.get(m - b[j]): abs(Int(i) - Int(j)) >= k else: false
print(I.count(good))
