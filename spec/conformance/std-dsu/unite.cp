// expect: ok, output "YES"
domain V = 0..3
var d = Dsu<V>()
print(d.unite(V(0), V(1)) and not d.unite(V(1), V(0)))
