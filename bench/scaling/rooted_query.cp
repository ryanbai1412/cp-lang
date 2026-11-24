// n: 2048
use std.rand.{Rng}
use std.segtree.{SegTree}
use std.algebra.{add}
let n: Int = read()
domain V = 0..n
var rng = Rng(7)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let t = Rooted.from_parents(rpar)
let gt = Rooted(Graph.undirected([0..n - 1] e => (u: V(e.ord + 1), v: rpar[V(e.ord + 1)]!)), root: V(0))
let jw_f: V -> Bool = v => v.ord > n / 2 // within: tree.Rooted.jump_while
var s = 0
let seg = SegTree(+, t.reorder([V] v => v.ord))
var lazy = SegTree(+, t.reorder([V] v => 0), act: add)
let ly = t.layers()
let ev = t.events()
let lp = t.long_paths()
let d = t.diameter()
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
    s += t.root_of(a).ord // timed: tree.Rooted.root_of
    s += t.lca(a, b).ord // timed: tree.Rooted.lca
    s += t.dist(a, b) // timed: tree.Rooted.dist
    s += (t.ancestor(a, 2) ?? a).ord // timed: tree.Rooted.ancestor
    s += (t.jump(a, b, 3) ?? a).ord // timed: tree.Rooted.jump
    s += (t.jump_while(a, jw_f) ?? a).ord // timed: tree.Rooted.jump_while
    for ch in t.chains(a, b): s += ch.range.count() // timed: tree.Rooted.chains
    s += t.fold_path(a, b, seg) // timed: tree.Rooted.fold_path
    t.apply_path(a, b, mut lazy, 1) // timed: tree.Rooted.apply_path
    s += t.w(a) // timed: tree.Rooted.w
    s += t.top(a).ord // timed: tree.Rooted.top
    s += t.chain(a).count() // timed: tree.Rooted.chain
    for r in t.sub_at(a, b): s += r.count() // timed: tree.Rooted.sub_at
    if t.on_path(c, a, b): s += 1 // timed: tree.Rooted.on_path
    s += t.meet(a, b, c).ord // timed: tree.Rooted.meet
    s += (t.farthest(Some((len: 0, u: a, v: a)), Some(d)) ?? d).len // timed: tree.Rooted.farthest
    s += ly.pos(a).ord // timed: tree.Layers.pos
    s += ly.level(t.depth(a)).count() // timed: tree.Layers.level
    s += ly.below(a, t.depth(a) + 1).count() // timed: tree.Layers.below
    s += ev.enter(a).ord // timed: tree.Events.enter
    s += ev.exit(a).ord // timed: tree.Events.exit
    s += ev.sub(a).count() // timed: tree.Events.sub
    s += ev.path(a, b).range.count() // timed: tree.Events.path
    s += lp.slots(a).count() // timed: tree.LongPaths.slots
    s += lp.slot(a, 0).ord // timed: tree.LongPaths.slot
    s %= 1000000007
print(s)
