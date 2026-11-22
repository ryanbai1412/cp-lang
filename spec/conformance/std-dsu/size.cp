// expect: ok, output "YES"
domain V = 0..3
var d = Dsu<V>()
d.unite(V(0), V(2))
print(yes(d.size(V(2)) == 2 and d.size(V(1)) == 1 and d.same(V(2), V(0)) and not d.same(V(0), V(1))))
