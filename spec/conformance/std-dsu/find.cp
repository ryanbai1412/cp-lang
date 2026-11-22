// expect: ok, output "YES"
domain V = 0..4
var d = Dsu<V>()
d.unite(V(2), V(3))
print(d.find(V(3)) == V(2) and d.find(V(0)) == V(0))
