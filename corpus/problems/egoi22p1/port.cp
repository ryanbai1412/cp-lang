// Source: EGOI '22 P1 (SubsetMex), https://dmoj.ca/problem/egoi22p1
// Problem: a multiset S given as counts f_0..f_{n-1} (n <= 50, f_i <= 1e16, t <= 200 tests),
//   with n not in S. An operation removes a set T of distinct elements of S and inserts
//   mex(T). Print the fewest operations that put n into S.
// Solution: one new copy of i takes one operation that uses a copy of every j < i. Go down from
//   need[n] = 1: the shortfall need[i] - f_i, if positive, costs that many operations and adds
//   it to need[j] for every j < i. The answer is the total shortfall, O(n^2).
// Bounds: n: 1..=50; have: 0..=1e16
// original: corpus/submissions/5198974.cpp

@!multitest
let n: 1..=50 = read()
let have: [0..n]0..=1e16 = read()
var need: [0..=n]Int = 0  // copies of each value still to make
need[n] = 1
var ops = 0
for i in (0..=n).rev:
    let short = need[i] - (if i < n: have[i] else: 0)
    if short > 0:
        ops += short
        for j in 0..i: need[j] += short
print(ops)
