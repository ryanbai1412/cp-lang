// expect: ok, output "YES"
domain V = 0..3
var d = Dsu<V>()
d.unite(V(0), V(1))
let m = d.mark()
d.unite(V(1), V(2))
d.undo_to(m)
print(d.same(V(0), V(1)) and not d.same(V(0), V(2)) and d.components == 2)
