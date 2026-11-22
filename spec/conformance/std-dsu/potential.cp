// expect: ok, output "YES\nYES\nYES\nYES"
domain V = 0..3
var d = PotentialDsu<V>()
print(yes(d.components == 3 and d.consistent))
d.unite(V(0), V(1), 5)
d.unite(V(1), V(2), 3)
print(yes(d.diff(V(0), V(2)) == Some(8) and d.diff(V(2), V(0)) == Some(-8)))
let m = d.mark()
print(yes(not d.unite(V(0), V(2), 7) and not d.consistent and d.components == 1))
d.undo_to(m)
print(yes(d.consistent and d.unite(V(0), V(2), 8)))
