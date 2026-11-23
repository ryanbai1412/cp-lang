// Source: Codeforces 2089E, Black Cat Collapse, https://codeforces.com/contest/2089/problem/E
// Problem: tree rooted at 1 whose labels 1..=n form a DFS order, n <= 80 (multitest, sum
//   n <= 80). Each day explore an uncollapsed node u, which collapses u's subtree; at the end of
//   day i node n - i + 1 also collapses. For each i <= n count the schemes of exactly i days
//   whose last exploration is node 1, mod 998244353.
// Solution: labels are preorder, so a parent is the smaller endpoint. Retire nodes n down to 2
//   and keep EGF series in the number of pending steps: sub[v] (f + integral f over v's
//   children's product), before[v] (its earlier siblings' product), dp[v] (states parked at v,
//   weighted by k!). Retiring t moves the states parked at t + 1 down to t's path and lifts
//   every state on the path to the root, taking siblings out by transposed products; the
//   root's constant term is the next answer. O(n^2) series products of length <= n.
use std.poly.{integral}

type Z = Mod<998_244_353>

fn add(a: List<Z>, b: List<Z>) -> List<Z>:
    var sum = a.resized(max(a.len, b.len), 0)
    for (i, x) in b.items: sum[i] += x
    sum

fn transposed_mul(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty: return []
    convolve(a, b.rev)[b.len - 1..].to_list()

fn plus_integral(a: List<Z>) -> List<Z> = add(a, integral(a))

@!multitest
let n: 1..=250_000 = read()
domain V = 1..=n
var parent: [V]?V = None
repeat n - 1:
    let (u, v): (V, V) = read()
    parent[max(u, v)] = min(u, v)
let tree = Rooted.from_parents(parent)

var sub: [V]List<Z> = [1]
for v in V.rev:
    sub[v] = plus_integral(sub[v])
    if let p = parent[v]: sub[p] = convolve(sub[p], sub[v])

var before: [V]List<Z> = []
var siblings: [V]List<Z> = [1]
for v in V if let p = parent[v]:
    before[v] = siblings[p]
    siblings[p] = convolve(siblings[p], sub[v])

var dp: [V]List<Z> = []
var ans: List<Z> = [1]
for t in V.rev if t != V.first:
    let next = t.next
    let (moving, stop): (List<Z>, V) = match next:
        Some(s) => (dp[s], parent[s]!)
        None => ([0, 1], V.first!)
    dp[t] = add(dp[t], moving)
    let chain = tree.path(t, stop)
    var ways: List<Z> = [1]
    for (prev, p) in chain[..chain.len - 1].with_prev:
        if let prev = prev: ways = convolve(ways, before[prev])
        dp[p] = add(dp[p], transposed_mul(moving, integral(ways)))
        ways = plus_integral(ways)
    if let s = next: dp[s] = []

    let up = tree.path(t, V.first!)
    var carry: List<Z> = []
    for (prev, p) in up.with_prev:
        if let prev = prev: carry = transposed_mul(carry, before[prev])
        let lifted: List<Z> = [0] + [x * (k + 1) for (k, x) in dp[p].items]
        (dp[p], carry) = (add(lifted, carry), add(carry, dp[p]))
    ans.push(dp[V.first!].first ?? 0)
print(ans)
