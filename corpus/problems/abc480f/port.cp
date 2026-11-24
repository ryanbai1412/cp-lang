// Source: AtCoder ABC 480 F (Cake Cutting), https://atcoder.jp/contests/abc480/tasks/abc480_f
// Problem: N <= 20 cakes of sizes A_i <= 1e9, M <= 100 people, sum S divisible by M. A cut splits
//   one piece in two (any real sizes). Print the fewest cuts after which every person can get
//   exactly S / M.
// Solution: lay the cakes in some order and cut at every multiple of T = S / M: that costs M
//   minus the number of prefixes whose sum is a multiple of T, and is optimal. dp[s] is the most
//   such prefixes over orders of the cakes in s (rem[s] = sum mod T); answer M - dp[full].
//   O(2^N N).

let (n, m): (1..=20, 1..=100) = read()
domain I = 0..n
let a: [I]1..=1e9 = read()
let t = a.sum() / m  // each person's share

// the sum of the cakes in s, mod t: a prefix that hits 0 saves a cut
var rem: [Subset<I>]Int = 0
for s in Subset<I>:
    if let i = s.first():
        rem[s] = (rem[s.without(i)] + a[i]) % t

// the most saved cuts over orders of the cakes in s
var saved: [Subset<I>]Int = 0
for s in Subset<I> if not s.is_empty:
    saved[s] = max(saved[s.without(i)] for i in s)! + Int(rem[s] == 0)

print(m - saved[Subset<I>.full])
