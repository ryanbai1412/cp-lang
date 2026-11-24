// Source: NOI 2018 Day 1 P2 (Inverse), https://dmoj.ca/problem/noi18p2
// Problem: p is good if bubble sort makes exactly 1/2 sum |i - p_i| swaps. Given a permutation q
//   of 1..n, count the good permutations of length n lexicographically greater than q, mod
//   998244353. T = 5 tests, n <= 6e5, sum n <= 2e6.
// Solution: p is good iff it has no decreasing subsequence of length 3: each element is a new
//   prefix maximum or the smallest unused value. Such suffixes are ballot paths, counted after
//   i placed values with maximum j by C(2n-i-j, n-i) - C(2n-i-j, n-j-1). Walk q: where p first
//   exceeds q at i, p_i is a new maximum >= max(mx, q_i) + 1, and those completions sum to one
//   ballot term; stop once q's own prefix is not good. O(n) per test after factorials.
// original: corpus/submissions/6512891.cpp
// Bounds: n: 1..=6e5; q: 1..=n

type Z = Mod<998_244_353>
let comb = Comb<Z>(2 * 600_000 + 5)

/// Good completions after i placed values with maximum `low - 1`.
fn ballot(n: Int, i: Int, low: Int) -> Z:
    if low > n: return 0
    let steps = 2 * n - i - low
    comb.choose(steps, n - i) - comb.choose(steps, n - low - 2)

@multitest
let n: 1..=6e5 = read()
let q: [1..=n]1..=n = read()
var used: [1..=n + 1]Bool = false
var (smallest, top, greater) = (1, 0, Z(0))
for i in 1..=n:
    greater += ballot(n, i, max(top, q[i]) + 1)
    if q[i] > top: top = q[i]
    elif q[i] != smallest: break
    used[q[i]] = true
    smallest = (smallest..=n + 1).find(v => not used[v])!
print(greater)
