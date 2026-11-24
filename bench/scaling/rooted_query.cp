// n: 2048
use std.rand.{Rng}
let n: Int = read()
domain V = 0..n
var rng = Rng(7)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let t = Rooted.from_parents(rpar)
let gt = Rooted(Graph.undirected([0..n - 1] e => (u: V(e.ord + 1), v: rpar[V(e.ord + 1)]!)), root: V(0))
let jw_f: V -> Bool = v => v.ord > n / 2 // within: tree.Rooted.jump_while
var s = 0
for i in 0..64 * n:
    let a = V(1 + (i * 7919) % (n - 1))
    let b = V((i * 104729 + 7) % n)
    let c = V((i * 31 + 3) % n)
    s += t.pos(a).ord // timed: tree.Rooted.pos
    s += (t.parent(a) ?? a).ord // timed: tree.Rooted.parent
    s += if gt.up_arc(a) == None: 0 else: 1 // timed: tree.Rooted.up_arc
    s += t.depth(a) // timed: tree.Rooted.depth
    s += t.size(a) // timed: tree.Rooted.size
    s += t.sub(a).count() // timed: tree.Rooted.sub
    s += t.below(a).count() // timed: tree.Rooted.below
    if t.is_anc(a, b): s += 1 // timed: tree.Rooted.is_anc
    s += t.lower(a, t.parent(a) ?? b).ord // timed: tree.Rooted.lower
    s += t.orient(a, t.parent(a) ?? a).anc.ord // timed: tree.Rooted.orient
    s += t.root_of(a).ord // timed: tree.Rooted.root_of
    s += t.lca(a, b).ord // timed: tree.Rooted.lca
    s += t.dist(a, b) // timed: tree.Rooted.dist
    s += (t.ancestor(a, 2) ?? a).ord // timed: tree.Rooted.ancestor
    s += (t.jump(a, b, 3) ?? a).ord // timed: tree.Rooted.jump
    s += (t.jump_while(a, jw_f) ?? a).ord // timed: tree.Rooted.jump_while
    if t.on_path(c, a, b): s += 1 // timed: tree.Rooted.on_path
    s += t.meet(a, b, c).ord // timed: tree.Rooted.meet
    s %= 1000000007
print(s)
