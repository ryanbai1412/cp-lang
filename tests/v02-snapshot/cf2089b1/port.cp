// Source: Codeforces 2089B1 (Canteen, Easy), https://codeforces.com/contest/2089/problem/B1
// Problem: circular arrays a, b of n <= 2e5 values (1..=1e9), sum a <= sum b, k = 0. Each round
//   every i drops t = min(a[i], b[i]) from both, then a rotates one step forward. Print the
//   rounds until all of a is 0. Multitest, sum of n <= 2e5.
// Solution: cut the circle right after the minimum prefix sum of a[i] - b[i]: then no supply
//   needs to wrap past the cut. In that order each demand b[i] drains the nearest earlier
//   unspent supply a[pos] (a stack of (qty, pos)), which arrives after i + 1 - pos rounds; the
//   answer is the largest such value. O(n) amortized.

struct Supply(qty: Int, pos: Int)

@!multitest
let (n, _k): (1..=200_000, 0..=10 ** 14) = read()
let a: [0..n]0..=10 ** 9 = read()
let b: [0..n]0..=10 ** 9 = read()

var pref = 0
var min_val = 0
var start = 0
for i in 0..n:
    pref += a[i] - b[i]
    if pref < min_val:
        min_val = pref
        start = i + 1

var stk: List<Supply> = []
var ans = 0
for i in 0..n:
    let idx = (start + i) % n
    stk.push(Supply(a[idx], i))
    var rem = b[idx]
    while let top = stk.last and rem > 0:
        let take = min(top.qty, rem)
        ans max= i + 1 - top.pos
        stk.pop()
        if top.qty > take: stk.push(Supply(top.qty - take, top.pos))
        rem -= take
print(ans)
