// Sorted, every group is a contiguous run, and the first group takes the
// prefix 0..st, each of whose elements lowers its cost. Past it the cost
// (P_i - P_j)(i - j) is Monge, so f(k) is convex in k: Aliens over a Monge
// shortest path. Scaling each edge by w = n + 1 and adding 1 keeps it Monge
// and breaks ties toward fewer groups, which `aliens` needs.
use std.dp.{monge_paths}
@!multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let a = read<[n]-1e9..=1e9>().sorted()
let pref = prefix(a)
let st = (0..n).find(c => pref.prefix(c) + a[c] * c > 0) ?? n
fn cost(j: Int, i: Int) -> I128 = I128(pref.fold(j..i)) * (i - j)
let rest = n - k + 1
if rest <= st: return print(cost(0, rest) + pref.fold(rest..n))

// The cheapest path through the cuts 0, st, ..., n with each group charged
// pen, and how many groups it takes.
let w = I128(n + 1)
fn penalized(pen: I128) -> (val: I128, count: Int):
    fn cut(p: Int) -> Int = if p == 0: 0 else: st + p - 1
    let path = monge_paths([k for k in 0..n - st + 2], (p, q) => (cost(cut(p), cut(q)) + pen) * w + 1)
    var (at, groups) = (path.val.len - 1, 0)
    while at > 0: (at, groups) = (path.prev[at], groups + 1)
    (val: (path.val.last! - groups) / w, count: groups)

let fewest = cost(0, st) + pref.fold(st..n) - cost(0, n)
print(aliens(k, penalized, lambdas: I128(0)..=1 - fewest))
