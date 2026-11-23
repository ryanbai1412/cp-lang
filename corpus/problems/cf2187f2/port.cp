// Source: Codeforces 2187F2 (Al Fine, counting), https://codeforces.com/contest/2187/problem/F2
// Problem: two DFS orders a, b of vertices 1..n (n <= 5000, sum of n^2 <= 2.5e7, multitest) of
//   trees rooted at 0. Count the rooted trees (children unordered) that have both as DFS orders,
//   mod 998244353.
// Solution: every subtree is an interval of both orders, so read b backwards in a's positions:
//   the common trees are the cuts of that permutation's PermTree. Increasing and prime nodes
//   multiply their children's ways. Down a decreasing chain, ends(v)[k] counts v's ways with k
//   single-vertex right children still free to climb; each such leaf adds a slot and lets every
//   open one move up (a prefix sum). O(n^2).
use std.tree.{PermTree, PermKind}

type Z = Mod<998_244_353>

@!multitest
let n: 2..=5000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()
let pos_a = a.inverse()
let t = PermTree([I] i => pos_a[b[I.at(n - 1 - i.ord)!]])

fn ends<Node>(v: Node) -> List<Z>:
    if t.kind[v] == PermKind.Leaf: return [Z(1)]
    let (l, r) = (t.left(v), t.right(v))
    if t.kind[v] != PermKind.Decr: return [ways(l) * ways(r)]
    var res = ends(l)
    res.push(0)
    if t.kind[r] == PermKind.Leaf:
        let pre = res.scan(+)
        [pre.prefix(i + 1) for i in res.keys]
    else:
        let wr = ways(r)
        res.map(e => e * wr)

fn ways<Node>(v: Node) -> Z = ends(v).sum()
print(ways(t.root))
