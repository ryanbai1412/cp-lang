use std.persistent.{PArena}
use std.dsu.{RollbackArray, DsuNode, PersistentDsu}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
let ord: V -> Int = x => Int(x)
let add: (Int, Int) -> Int = (x, y) => x + y
let init_new: V -> Int = x => Int(x) // within: dsu.DsuOf.new
let join_unite: (Int, Int) -> Int = (x, y) => x + y // within: dsu.DsuOf.unite
let join_merge: (Int, Int) -> Int = (x, y) => x + y // within: dsu.DsuOf.merge
let init_array: V -> Int = x => Int(x) // within: dsu.RollbackArray.new
var s = 0
for _ in 0..2000:
    s += Dsu<V>().components // timed: dsu.Dsu.new
var d = Dsu<V>()
for i in 0..6 * n:
    if d.unite(V((i * 7919) % n), V((i * 104729 + 1) % n)): s += 1 // timed: dsu.Dsu.unite
    if d.merge(V((i * 31) % n), V((i * 17 + 5) % n)) != None: s += 1 // timed: dsu.Dsu.merge
for i in 0..reps:
    s += d.find(V((i * 13) % n)).ord // timed: dsu.Dsu.find
    if d.same(V((i) % n), V((i * 7 + 3) % n)): s += 1 // timed: dsu.Dsu.same
    s += d.size(V((i * 11) % n)) // timed: dsu.Dsu.size
    s += d.components // timed: dsu.Dsu.components
for _ in 0..1500:
    for r in d.roots(): s += r.ord // timed: dsu.Dsu.roots
for _ in 0..60:
    s += d.groups().len // timed: dsu.Dsu.groups
var r = Dsu<V>(rollback: true)
for i in 0..reps:
    let m = r.mark() // timed: dsu.Dsu.mark
    r.unite(V((i) % n), V((i * 7 + 1) % n))
    r.undo_to(m) // timed: dsu.Dsu.undo_to

for _ in 0..60:
    s += DsuOf<V, Int>(init: init_new, join: add).components // timed: dsu.DsuOf.new
var o = DsuOf<V, Int>(init: ord, join: join_unite)
var om = DsuOf<V, Int>(init: ord, join: join_merge)
for i in 0..6 * n:
    if o.unite(V((i * 7919) % n), V((i * 104729 + 1) % n)): s += 1 // timed: dsu.DsuOf.unite
    if om.merge(V((i * 31) % n), V((i * 17 + 5) % n)) != None: s += 1 // timed: dsu.DsuOf.merge
for i in 0..reps:
    s += o.find(V((i * 13) % n)).ord // timed: dsu.DsuOf.find
    s += o.get(V((i * 5) % n)) // timed: dsu.DsuOf.get
    o.set(V((i * 3) % n), i) // timed: dsu.DsuOf.set
    if o.same(V((i) % n), V((i * 7 + 3) % n)): s += 1 // timed: dsu.DsuOf.same
    s += o.size(V((i * 11) % n)) // timed: dsu.DsuOf.size
    s += o.components // timed: dsu.DsuOf.components
for _ in 0..1500:
    for x in o.roots(): s += x.ord // timed: dsu.DsuOf.roots
    for x in o.values: s += x // timed: dsu.DsuOf.values
for _ in 0..60:
    s += o.groups().len // timed: dsu.DsuOf.groups
var ou = DsuOf<V, Int>(init: ord, join: add)
for i in 0..reps:
    let m = ou.mark() // timed: dsu.DsuOf.mark
    ou.unite(V((i) % n), V((i * 7 + 1) % n))
    ou.undo_to(m) // timed: dsu.DsuOf.undo_to

for _ in 0..300:
    if ParityDsu<V>().consistent: s += 1 // timed: dsu.ParityDsu.new
var p = ParityDsu<V>()
for i in 0..6 * n:
    if p.unite(V((i * 7919) % n), V((i * 104729 + 1) % n)): s += 1 // timed: dsu.ParityDsu.unite
for i in 0..reps:
    s += p.find(V((i * 13) % n)).root.ord // timed: dsu.ParityDsu.find
    if p.diff(V((i) % n), V((i * 3 + 1) % n)) == Some(true): s += 1 // timed: dsu.ParityDsu.diff
    if p.same(V((i) % n), V((i * 7 + 3) % n)): s += 1 // timed: dsu.ParityDsu.same
    s += p.size(V((i * 11) % n)) // timed: dsu.ParityDsu.size
    if p.consistent: s += 1 // timed: dsu.ParityDsu.consistent
for i in 0..reps:
    let m = p.mark() // timed: dsu.ParityDsu.mark
    p.unite(V((i) % n), V((i * 7 + 1) % n))
    p.undo_to(m) // timed: dsu.ParityDsu.undo_to

for _ in 0..250:
    s += PotentialDsu<V>().components // timed: dsu.PotentialDsu.new
var q = PotentialDsu<V>()
for i in 0..6 * n:
    if q.unite(V((i * 7919) % n), V((i * 104729 + 1) % n), i % 7): s += 1 // timed: dsu.PotentialDsu.unite
for i in 0..reps:
    s += q.find(V((i * 13) % n)).offset // timed: dsu.PotentialDsu.find
    s += q.diff(V((i) % n), V((i * 3 + 1) % n)) ?? 0 // timed: dsu.PotentialDsu.diff
    if q.same(V((i) % n), V((i * 7 + 3) % n)): s += 1 // timed: dsu.PotentialDsu.same
    s += q.size(V((i * 11) % n)) // timed: dsu.PotentialDsu.size
    s += q.components // timed: dsu.PotentialDsu.components
    if q.consistent: s += 1 // timed: dsu.PotentialDsu.consistent
for i in 0..reps:
    let m = q.mark() // timed: dsu.PotentialDsu.mark
    q.unite(V((i) % n), V((i * 7 + 1) % n), 1)
    q.undo_to(m) // timed: dsu.PotentialDsu.undo_to

for _ in 0..60:
    s += RollbackArray<V, Int>(init: init_array).get(V(1)) // timed: dsu.RollbackArray.new
var a = RollbackArray<V, Int>(init: ord)
for i in 0..reps:
    s += a.get(V((i * 7) % n)) // timed: dsu.RollbackArray.get
    let m = a.mark() // timed: dsu.RollbackArray.mark
    a.set(V((i) % n), i) // timed: dsu.RollbackArray.set
    a.undo_to(m) // timed: dsu.RollbackArray.undo_to

var ar = PArena<DsuNode<V>>()
for _ in 0..6:
    let e = PersistentDsu<V>.new(mut ar) // timed: dsu.PersistentDsu.new
var e = PersistentDsu<V>.new(mut ar)
for i in 0..n:
    e = e.unite(mut ar, V((i * 7919) % n), V((i * 104729 + 1) % n)) // timed: dsu.PersistentDsu.unite
for i in 0..reps / 8:
    s += e.find(ar, V((i * 13) % n)).ord // timed: dsu.PersistentDsu.find
    if e.same(ar, V((i) % n), V((i * 7 + 3) % n)): s += 1 // timed: dsu.PersistentDsu.same
    s += e.size(ar, V((i * 11) % n)) // timed: dsu.PersistentDsu.size
print(s, r.components, ar.len)
