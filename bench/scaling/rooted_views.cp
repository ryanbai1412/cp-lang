// n: 2048
use std.rand.{Rng}
let n: Int = read()
domain V = 0..n
var rng = Rng(7)
let rpar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(rng.int(0, v.ord - 1)))
let t = Rooted.from_parents(rpar)
let ly = t.layers()
let ev = t.events()
let lp = t.long_paths()
let d = t.diameter()
var s = 0
for i in 0..64 * n:
    let a = V(1 + (i * 7919) % (n - 1))
    let b = V((i * 104729 + 7) % n)
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
