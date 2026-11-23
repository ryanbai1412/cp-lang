// Codeforces 2103C Median Splits: https://codeforces.com/problemset/problem/2103/C
// A part has median <= k iff it has at least as many values <= k as > k: its sum of +1/-1 is >= 0.
// Two of the three parts must be good. Parts 1 and 3: the shortest good prefix and the shortest
// good suffix must leave a middle (two pointers from the ends). Parts 1 and 2 (or 2 and 3): at
// the second cut r, some earlier cut l has 0 <= pre[l] <= pre[r] (or pre[l] <= pre[r] <= total).
@!multitest
let (n, k): (3..=2e5, 1..=1e9) = read()
domain I = 1..=n
let a = read<[I]1..=1e9>()
let pre = a.map(x => if x <= k: 1 else: -1).scan(+)
let total = pre[I.cuts.last!]
let inner = I.cuts.filter(c => 1 <= Int(c) <= n - 1)
var ok = false
if let l = inner.find(c => pre[c] >= 0), let r = inner.rfind(c => total - pre[c] >= 0):
    ok = Int(l) < Int(r)
var low_good: ?Int = None
var low: ?Int = None
for r in inner if Int(r) >= 2:
    let p = pre[r.prev!]
    if p >= 0: low_good min= p
    low min= p
    if low_good.is_some_and(x => x <= pre[r]) or low.is_some_and(x => x <= pre[r] <= total):
        ok = true
print(yes(ok))
