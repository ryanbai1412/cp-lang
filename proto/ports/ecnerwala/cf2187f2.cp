// Codeforces 2187 F2: Al Fine (counting version).
// Same permutation tree as F1: an increasing node multiplies its children's
// counts; a maximal chain of decreasing nodes multiplies the counts of its
// non-leaf right children by the ways to place its leaf pattern.
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
