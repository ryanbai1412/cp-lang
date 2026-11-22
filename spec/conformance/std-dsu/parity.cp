// expect: ok, output "YES\nYES\nYES\nYES"
domain V = 0..3
var d = ParityDsu<V>()
print(d.consistent and d.diff(V(0), V(1)) == None)
d.unite(V(0), V(1))
d.unite(V(1), V(2))
let m = d.mark()
print(not d.unite(V(0), V(2)) and not d.consistent)
d.undo_to(m)
print(d.consistent and d.unite(V(0), V(2), differ: false))
print(d.find(V(1)) == (root: V(0), flip: true))
