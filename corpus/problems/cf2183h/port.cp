// Source: Codeforces 2183H, Minimise Cost, https://codeforces.com/contest/2183/problem/H
// Problem: multitest, k <= n <= 2e5 (sum bounded), |a_i| <= 1e9. Split a into exactly k
//   non-empty subsequences; a subsequence of length m and sum s costs m * s. Print the
//   minimum total cost.
// Solution: sorted, every group is a contiguous run, and the first takes the prefix 0..st whose
//   elements each lower its cost. Past it the cost (P_i - P_j)(i - j) is Monge, so f(k) is
//   convex: Aliens over a Monge shortest path (edges scaled by n + 1 plus 1, which breaks
//   ties toward fewer groups). O(n log C) Monge path computations.
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
    let path = monge_paths(
        (0..n - st + 2).collect(),
        (p, q) => (cost(cut(p), cut(q)) + pen) * w + 1,
    )
    var (at, groups) = (path.val.len - 1, 0)
    while at > 0: (at, groups) = (path.prev[at], groups + 1)
    (val: (path.val.last! - groups) / w, count: groups)

let fewest = cost(0, st) + pref.fold(st..n) - cost(0, n)
print(aliens(k, penalized, lambdas: I128(0)..=1 - fewest))
