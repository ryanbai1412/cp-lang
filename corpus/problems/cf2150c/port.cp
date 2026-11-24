// Source: Codeforces 2150 C (Limited Edition Shop), https://codeforces.com/contest/2150/problem/C
// limits: 2000 ms, 256 MB
// Problem: n objects with values in [-1e9, 1e9]; Alice and Bob each have a preference order. In
//   any order of visits, the visitor buys their favourite remaining object. Over all sets Alice
//   can end with, print the maximum total value. Multitest, sum of n <= 2e5.
// Solution: walk Alice's order keeping dp[j], the best value so far when Bob has taken his first
//   j objects, which is non-decreasing in j. Alice taking an object at Bob position pos adds
//   its value to dp[0..pos]; then prefix-max restores monotonicity. dp is kept as a base plus
//   a map of positive increments, a negative increment cancelling the next positive ones;
//   the answer is dp[n]. O(n log n) amortised.

@!multitest
let n: 1..=200_000 = read()
domain I = 0..n
domain V = 1..=n
let vals: [V]Int = read()
let a: [I]V = read()
let b: [I]V = read()
let inv_b = b.inverse()

var tot = 0
var deltas: Map<Int, Int> = Map()
for z in I:
    let cur = a[z]
    let v = vals[cur]
    let pos = inv_b[cur].ord
    tot += v
    deltas[pos] -= v
    while let cur_val = deltas.get(pos) and cur_val < 0:
        if let (nxt_k, nxt_v) = deltas.first_gt(pos):
            let d = min(-cur_val, nxt_v)
            if nxt_v == d: deltas.remove(nxt_k) else: deltas[nxt_k] -= d
            if cur_val + d == 0: deltas.remove(pos) else: deltas[pos] += d
        else:
            deltas.remove(pos)

print(tot + sum(v for (_, v) in deltas))
