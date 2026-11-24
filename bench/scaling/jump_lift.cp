// n: 2048
use std.rand.{Rng}
use std.tree.{JumpTree, Lift}
let n: Int = read()
domain V = 0..n
var rng = Rng(5)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let wpar: [V]?(V, Int) = [V] v => if let p = rpar[v]: Some((p, v.ord % 7)) else: None
let next_f: V -> ?V = v => rpar[v] // within: tree.Lift.of
var s = 0
domain W = 0..32 * n
for _ in 0..100:
    s += JumpTree<W>().depth(W(0)) // timed: tree.JumpTree.new
for _ in 0..40:
    var j0 = JumpTree<V>()
    for v in V:
        if let p = rpar[v]: j0.add(v, p) // timed: tree.JumpTree.add
    s += j0.depth(V(n - 1))
for _ in 0..100:
    s += Lift(wpar, op: (x, y) => x + y, identity: 0).depth(V(n - 1)) // timed: tree.Lift.new
    s += Lift<V, ()>.of(next_f).depth(V(n - 1)) // timed: tree.Lift.of
var j = JumpTree<V>()
for v in V:
    if let p = rpar[v]: j.add(v, p)
let l = Lift(wpar, op: (x, y) => x + y, identity: 0)
let ok_f: V -> Bool = v => v.ord > n / 3 // within: tree.JumpTree.jump_while
let cl_f: V -> Bool = v => v.ord > n / 3 // within: tree.Lift.climb
let sum_f: Int -> Bool = x => x <= 20 // within: tree.Lift.climb_while
for i in 0..64 * n:
    let a = V((i * 7919) % n)
    let b = V((i * 104729 + 7) % n)
    s += (j.parent(a) ?? a).ord // timed: tree.JumpTree.parent
    s += j.depth(a) // timed: tree.JumpTree.depth
    s += j.root_of(a).ord // timed: tree.JumpTree.root_of
    s += (j.ancestor(a, 3) ?? a).ord // timed: tree.JumpTree.ancestor
    s += (j.lca(a, b) ?? a).ord // timed: tree.JumpTree.lca
    s += (j.jump_while(a, ok_f) ?? a).ord // timed: tree.JumpTree.jump_while
    s += j.dist(a, b) ?? 0 // timed: tree.JumpTree.dist
    s += (j.jump(a, b, 2) ?? a).ord // timed: tree.JumpTree.jump
    s += l.climb(a, cl_f).steps // timed: tree.Lift.climb
    s += (l.parent(a) ?? a).ord // timed: tree.Lift.parent
    s += l.depth(a) // timed: tree.Lift.depth
    s += (l.ancestor(a, 3) ?? a).ord // timed: tree.Lift.ancestor
    let c = l.lca(a, b) ?? a // timed: tree.Lift.lca
    s += l.fold(a, up_to: c) // timed: tree.Lift.fold
    s += l.climb_while(a, sum_f).ord // timed: tree.Lift.climb_while
    s %= 1000000007
print(s)
