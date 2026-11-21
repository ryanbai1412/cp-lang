// Every subtree is an interval of both DFS orders, so read b backwards in a's
// positions: the common trees are the cuts of that sequence's permutation tree.
// Increasing and prime nodes multiply their children. Down a decreasing chain,
// ends(v)[k] counts v's ways with k single-vertex right children still free to
// climb; each such child adds a slot and lets every open one move up.
use std.mod.{Mod}
use std.seq.{Perm}
use std.tree.{PermTree}

type Z = Mod<998_244_353>
@!multitest
let n: 2..=5000 = read()
domain V = 1..=n
let (a, b) = read<(Perm<V>, Perm<V>)>()
let t = PermTree(b.rev.map(a.inverse))

fn ends(v: t.Node) -> List<Z> = match t[v]:
    leaf => [1]
    decr(l, r: leaf) => (ends(l) + [0]).prefix_sums()
    decr(l, r) => (ends(l) + [0]).map(e => e * ways(r))
    node(l, r) => [ways(l) * ways(r)]
fn ways(v: t.Node) -> Z = ends(v).sum()
print(ways(t.root))
