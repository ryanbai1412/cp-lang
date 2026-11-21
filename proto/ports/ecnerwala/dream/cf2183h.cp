// Sorted, every group is a contiguous run, and the first group takes the
// prefix 0..st, each of whose elements lowers its cost. Past it the cost
// (P_i - P_j)(i - j) is Monge, so f(k) is convex in k: Aliens over an O(n)
// Monge shortest path.
use std.dp.{aliens, monge_path}

@!multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let a = read<[n]-1e9..=1e9>().sorted()
let pref = a.prefix_sums()                                       // [a.Cut]Int
let st = a.cuts.find(c => c == n or pref[c] + a[c] * c > 0)!
fn cost(j: a.Cut, i: a.Cut) -> I128 = (pref[i] - pref[j]) * (i - j)
let rest = n - k + 1
if rest <= st: return print(cost(0, rest) + pref[n] - pref[rest])
let nodes = [a.Cut(0)] + (st..=n)
let fewest = cost(0, st) + pref[n] - pref[st] - cost(0, n)
fn penalized(pen: I128) -> (val: I128, count: Int):
    let path = monge_path(nodes.len, w: (p, q) => cost(nodes[p], nodes[q]) + pen)
    (val: path.val.last!, count: path.edges.last!)
print(aliens(k, lambdas: I128(0)..=1 - fewest, solve: penalized))
