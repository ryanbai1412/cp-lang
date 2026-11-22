// Codeforces 2089 B1 — Canteen (Easy Version)
//
// Each round subtracts min(a[i], b[i]) at every index and rotates `a` forward
// by 1 until all `a[i]` reach 0 (with sum a <= sum b).
//
// Because sum(a[i] - b[i]) <= 0, rotating the circle to start right after the
// minimum prefix sum of `a[i] - b[i]` ensures every suffix/prefix surplus from
// `a` is absorbed before wrapping back past the cut. In that linearized order,
// every supply `a[pos]` is consumed by the first demand `b[i]` that reaches it
// (LIFO stack), taking `i + 1 - pos` rounds; the answer is the maximum over
// all matched (supply, demand) pairs.

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
