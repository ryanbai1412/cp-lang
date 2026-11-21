// Every subtree is an interval of both DFS orders, so read b backwards in a's
// positions: the common trees are the cuts of that sequence's permutation tree.
// Increasing and prime nodes multiply their children. Down a decreasing chain,
// ends(v)[k] counts v's ways with k single-vertex right children still free to
// climb; each such child adds a slot and lets every open one move up.
use std.seq.{inverse, scan_in_place}
use std.tree.{PermTree, PermKind}

type Z = Mod<998_244_353>

@!multitest
let n: 2..=5000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()
let pos_a = a.inverse()
let t = PermTree.of([I] i => pos_a[b[I(n - 1 - i.ord)]])

fn count<Node>(v: Node) -> Z:
    if t.kind[v] == PermKind.Leaf: return 1
    if t.kind[v] != PermKind.Decr: return count(t.left(v)) * count(t.right(v))
    var leafs = List<Bool>()
    var ways: Z = 1
    var cur = v
    while t.kind[cur] == PermKind.Decr:
        let r = t.right(cur)
        leafs.push(t.kind[r] == PermKind.Leaf)
        if t.kind[r] != PermKind.Leaf: ways *= count(r)
        cur = t.left(cur)
    ways *= count(cur)
    var ends: List<Z> = [1]
    for leaf in leafs.rev:
        ends.push(0)
        if leaf:
            ends.scan_in_place((x, y) => x + y)
    ways * ends.sum()
print(count(t.root))
