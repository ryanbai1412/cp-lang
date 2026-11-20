// cp: 0.1 bea787c+dirty
domain V = 1..=5
var d = Dsu<V>()
print(d.unite(V(1), V(2)), d.unite(V(1), V(2)), d.find(V(2)) == d.find(V(1)), d.same(V(1), V(3)), d.size(V(1)))
