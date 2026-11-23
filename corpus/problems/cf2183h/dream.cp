// Sorted, every group is a contiguous run, and the first group takes the
// prefix 0..st, each of whose elements lowers its cost. Past it the cost
// (P_i - P_j)(i - j) is Monge, so f(k) is convex in k: Aliens over an O(n)
// Monge shortest path.
use std.dp.{monge_path}

@!multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let a = read<[n]-1e9..=1e9>().sorted()
let pref = a.prefix_sums()                                       // [a.Cut]Int
let st = a.cuts.find(c => c == n or pref[c] + a[c] * c > 0)!
fn cost(j: a.Cut, i: a.Cut) -> I128 = (pref[i] - pref[j]) * (i - j)
let rest = n - k + 1
if rest <= st: return print(cost(0, rest) + pref[n] - pref[rest])
print(aliens(k, solve: pen => monge_path([0] + (st..=n), cost: (j, i) => cost(j, i) + pen)))
