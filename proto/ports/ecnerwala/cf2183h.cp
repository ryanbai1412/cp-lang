// Codeforces 2183 H.
// Sorted, an optimal partition is contiguous. The first group always takes
// the prefix `0..st` (each of those elements lowers its cost), and past it the
// segment cost (P_i - P_j)(i - j) is Monge, so f(k) is convex: Aliens trick.
// Each penalized DP is a shortest path through 0, st, ..., n under a Monge
// cost: `monge_path`. Scaling every edge by w = n + 1 and adding 1 keeps it
// Monge and breaks ties toward fewer groups, which `aliens` needs.
use std.dp.{aliens, monge_path}
@!multitest
let n: 1..=200_000 = read()
let k: 1..=n = read()
var a: [0..n]-1_000_000_000..=1_000_000_000 = read()
a.sort()
let pref = prefix_n([Int(x) for x in a])
var st = 0
while st < n and pref.prefix(st) + a[st] * st <= 0: st += 1

let w = I128(n + 1)
fn solve(pen: I128) -> (val: I128, count: Int):
    fn cost(p: Int, q: Int) -> I128:
        let (j, i) = (if p == 0: 0 else: st + p - 1, st + q - 1)
        (I128(pref.sum(j..i)) * (i - j) + pen) * w + 1
    let path = monge_path(n - st + 2, cost)
    var (at, groups) = (path.val.len - 1, 0)
    while at > 0:
        (at, groups) = (path.prev[at], groups + 1)
    (val: (path.val.last! - groups) / w, count: groups)

let one_group = I128(pref.prefix(n)) * n
let most_groups = n - st + 1
if k == 1:
    print(one_group)
elif k >= most_groups:
    let rest = n - k + 1
    print(I128(pref.prefix(rest)) * rest + pref.sum(rest..n))
else:
    let fewest = I128(pref.prefix(st)) * st + pref.sum(st..n) - one_group
    print(aliens(k, I128(0)..=1 - fewest, solve))
