// Every subtree is an interval of both DFS orders, so read b backwards in a's
// positions: the common trees are the cuts of that sequence's permutation tree.
// Increasing and prime nodes multiply their children. Down a decreasing chain,
// ends(v)[k] counts v's ways with k single-vertex right children still free to
// climb; each such child adds a slot and lets every open one move up.
use std.tree.{PermKind, PermTree}

type Z = Mod<998_244_353>
@!multitest
let n: 2..=5000 = read()
domain V = 1..=n
let (a, b) = read<(Perm<V>, Perm<V>)>()
let pos_a = a.inverse()
let t = PermTree([V] i => pos_a[b[V.nth(n - 1 - i.ord)]])

fn running(e: List<Z>) -> List<Z>:
    let pre = e.scan(+)
    [pre.prefix(i + 1) for i in e.keys]

fn ends<Node>(v: Node) -> List<Z> = match t.node(v):
    Leaf(_) => [1]
    Inner(l, r, Decr) if t.kind[r] == PermKind.Leaf => running(ends(l) + [0])
    Inner(l, r, Decr) => (ends(l) + [0]).map(e => e * ways(r))
    Inner(l, r, _) => [ways(l) * ways(r)]
fn ways<Node>(v: Node) -> Z = ends(v).sum()
print(ways(t.root))
