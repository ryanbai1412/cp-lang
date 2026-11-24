// expect: ok, output "NO"
domain V = 0..3
var d = Dsu<V>()
let m = d.mark()
d.unite(V(0), V(2))
d.undo_to(m)
d.undo_to(m)
print(yes(d.same(V(0), V(2))))
