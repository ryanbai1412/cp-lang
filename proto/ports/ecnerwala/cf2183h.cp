// Codeforces 2183 H.
// Sorted, an optimal partition is contiguous. The first group always takes
// the prefix `0..st` (each of those elements lowers its cost), and past it the
// segment cost (P_i - P_j)(i - j) is Monge, so f(k) is convex: Aliens trick.
// A group costs an extra 1/2 - lam (doubled to stay integral), which makes the
// optimal group count unique; binary search the least lam reaching k groups.
// Each penalized DP is a shortest path through 0, st, ..., n under a Monge
// cost: `monge_path`.
use std.dp.{monge_path}
type Best = (cost: I128, groups: Int)
@!multitest
let n: 1..=200_000 = read()
let k: 1..=n = read()
var a: [0..n]-1_000_000_000..=1_000_000_000 = read()
a.sort()
var pref: [0..=n]Int = 0
for i in 0..n: pref[i + 1] = pref[i] + a[i]
var st = 0
while st < n and pref[st] + a[st] * st <= 0: st += 1

fn solve(lam: I128) -> Best:
    fn cost(p: Int, q: Int) -> I128:
        let (j, i) = (if p == 0: 0 else: st + p - 1, st + q - 1)
        2 * I128(pref[i] - pref[j]) * (i - j) - (2 * lam - 1)
    let path = monge_path(n - st + 2, cost)
    var (at, groups) = (path.val.len - 1, 0)
    while at > 0:
        (at, groups) = (path.prev[at], groups + 1)
    (cost: path.val.last!, groups: groups)

let one_group = I128(pref[n]) * n
let most_groups = n - st + 1
if k == 1:
    print(one_group)
elif k >= most_groups:
    let rest = n - k + 1
    print(I128(pref[rest]) * rest + (pref[n] - pref[rest]))
else:
    let fewest = I128(pref[st]) * st + (pref[n] - pref[st]) - one_group
    var (lo, lam) = (fewest, I128(0))
    while lo < lam:
        let mid = lo + (lam - lo) / 2
        if solve(mid).groups >= k: lam = mid else: lo = mid + 1
    let best = solve(lam)
    let f = (best.cost + (2 * lam - 1) * best.groups) / 2
    print(f - (lam - 1) * (best.groups - k))
