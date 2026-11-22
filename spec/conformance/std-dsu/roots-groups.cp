// expect: ok, output "0 2 3\n0\n1 3\n2"
domain V = 0..4
var d = Dsu<V>()
d.unite(V(3), V(1))
print([r for r in d.roots()])
print(d.groups())
