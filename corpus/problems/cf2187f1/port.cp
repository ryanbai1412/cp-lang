// Every subtree is an interval of both DFS orders, so read b backwards in a's
// positions: the common trees are the cuts of that sequence's permutation tree.
// A decreasing node whose right child is one vertex hangs its left part one
// level deeper; any other node is as deep as its deeper child.
use std.tree.{PermTree, PermKind}

@!multitest
let n: 2..=1_000_000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()
let pos_a = a.inverse()
let t = PermTree.new([I] i => pos_a[b[I(n - 1 - i.ord)]])

fn depth<Node>(v: Node) -> Int:
    if t.kind[v] == PermKind.Leaf: return 1
    if t.kind[v] == PermKind.Decr and t.kind[t.right(v)] == PermKind.Leaf: return 1 + depth(t.left(v))
    max(depth(t.left(v)), depth(t.right(v)))
print(depth(t.root))
