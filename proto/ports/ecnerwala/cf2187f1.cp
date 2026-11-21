// Codeforces 2187 F1: Al Fine (maximizing version).
// The common trees are the decreasing nodes of the permutation tree of b read
// backwards in a's coordinates; the deepest one follows a decreasing node's
// chain while its right child is a leaf.
use std.seq.{inverse}
use std.tree.{PermTree, PermKind}

@!multitest
let n: 2..=1_000_000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()
let pos_a = a.inverse()
let t = PermTree.of([I] i => pos_a[b[I(n - 1 - i.ord)]])

fn depth<Node>(v: Node) -> Int:
    if t.kind[v] == PermKind.Leaf: return 1
    if t.kind[v] == PermKind.Decr and t.kind[t.right(v)] == PermKind.Leaf: return 1 + depth(t.left(v))
    max(depth(t.left(v)), depth(t.right(v)))
print(depth(t.root))
