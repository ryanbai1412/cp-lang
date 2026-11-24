// Sorted, every group is a contiguous run, and the first group takes the
// prefix 0..st, each of whose elements lowers its cost. Past it the cost
// (P_i - P_j)(i - j) is Monge, so f(k) is convex in k: Aliens over an O(n)
// Monge shortest path.
use std.dp.{monge_path}

@!multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let a = read<[n]-1e9..=1e9>().sorted()
let pref = a.scan(+)
let st = (0..=n).find(c => c == n or pref.prefix(c) + a[c] * c > 0)!
fn cost(j: Int, i: Int) -> I128 = I128(pref.fold(j..i)) * (i - j)
let rest = n - k + 1
if rest <= st: return print(cost(0, rest) + pref.fold(rest..n))
print(aliens(k, solve: pen => monge_path([0] + (st..=n).collect(), cost: (j, i) => cost(j, i) + pen)))
