// expect: ok, output "4 2\n0 1 3\n2\n2"
// `ParityDsu.components` counts sets like `Dsu`'s and `groups()` lists them; a contradiction
// and an implied constraint merge nothing, and `undo_to` gives a merge's set back.
domain V = 0..4
var d = ParityDsu<V>()
let before = d.components
d.unite(V(0), V(1))
d.unite(V(1), V(3))
d.unite(V(0), V(3))
let m = d.mark()
d.unite(V(3), V(0), differ: false)
print(before, d.components)
print(d.groups())
d.unite(V(2), V(0))
d.undo_to(m)
d.unite(V(2), V(2), differ: false)
print(d.components)
