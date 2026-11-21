// Codeforces 2187 F1: Al Fine (dream). The answer is the depth of the
// permutation tree of b read backwards in a's coordinates, where a decreasing
// node whose right child is a leaf adds one level.
@!multitest
let n: 2..=1e6 = read()
domain V = 1..=n
let (a, b): (Perm<V>, Perm<V>) = read()
let p = b.rev.map(a.inverse())                 // Perm<Pos>, no hand-built inverse
let t = PermTree.of(p)                         // stdlib, linear; typed nodes t.Node

fn depth(v: t.Node) -> Int = match t[v]:
    leaf => 1
    decr(l, r: leaf) => 1 + depth(l)
    decr(l, r) | incr(l, r) => max(depth(l), depth(r))
print(depth(t.root))
