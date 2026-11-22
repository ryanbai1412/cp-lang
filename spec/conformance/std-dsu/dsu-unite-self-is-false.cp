// expect: ok, output "NO 4"
// rule: std.dsu.Dsu.unite
domain V = 0..4
var d = Dsu<V>()
print(yes(d.unite(V(2), V(2))), d.components)
