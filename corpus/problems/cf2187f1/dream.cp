// Every subtree is an interval of both DFS orders, so read b backwards in a's
// positions: the common trees are the cuts of that sequence's permutation tree.
// A decreasing node whose right child is one vertex hangs its left part one
// level deeper; any other node is as deep as its deeper child.
use std.seq.{Perm}
use std.tree.{PermKind, PermTree}

@!multitest
let n: 2..=1e6 = read()
domain V = 1..=n
let (a, b) = read<(Perm<V>, Perm<V>)>()
let t = PermTree(b.rev.map(a.inverse))

fn depth(v: t.Node) -> Int = match t.node(v):
    Leaf(_) => 1
    Inner(l, r, Decr) if t.kind[r] == PermKind.Leaf => depth(l) + 1
    Inner(l, r, _) => max(depth(l), depth(r))
print(depth(t.root))
