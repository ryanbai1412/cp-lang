// expect: R0401 (checked)
// rule: std.dsu.Dsu.undo_to
domain V = 0..4
var d = Dsu<V>(rollback: true)
let a = d.mark()
d.unite(V(0), V(1))
let b = d.mark()
d.undo_to(a)
d.undo_to(b) // <- here
print(d.components)
