// expect: ok, output "0 0 2 0\n0 0 2 0"
// rule: std.dsu DsuOf prints each vertex's root in V order, without compressing paths
domain V = 0..4
var d = DsuOf<V, Int>(init: v => v.ord, join: (x, y) => x + y)
let _ = d.unite(V(0), V(1))
let _ = d.unite(V(1), V(3))
print(d)
print(d)
