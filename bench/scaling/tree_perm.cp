// n: 2048
// let: size = n
use std.rand.{Rng}
use std.tree.{PermTree, PermNode}
let n: Int = read()
var rng = Rng(11)
domain I = 0..n
var perm: [I]I = [I] i => i
for i in (1..n).rev:
    let j = rng.int(0, i)
    let x = perm[I(i)]
    perm[I(i)] = perm[I(j)]
    perm[I(j)] = x
var s = 0
for _ in 0..100:
    s += PermTree(perm).first.len // timed: tree.PermTree.new
let pt = PermTree(perm)
for i in 0..64 * n:
    let lf = pt.leaf(I((i * 31) % n)) // timed: tree.PermTree.leaf
    s += pt.span(lf).count() // timed: tree.PermTree.span
    s += pt.vals(lf).count() // timed: tree.PermTree.vals
    let x = pt.root
    s += pt.left(x).ord // timed: tree.PermTree.left
    s += pt.right(x).ord // timed: tree.PermTree.right
    s += if pt.node(lf) == PermNode.Leaf(I(0)): 1 else: 0 // timed: tree.PermTree.node
    s %= 1000000007
print(s)
