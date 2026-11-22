domain V = 0..5
var d = Dsu<V>()
let snap = d.mark()
fn try():
    defer d.undo_to(snap)
    d.unite(V(0), V(1))
try()
print(d.same(V(0), V(1)))
