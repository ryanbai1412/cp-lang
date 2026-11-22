let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
var d = Dsu<V>()
for _ in 0..m:
    let (u, v): (V, V) = read()
    d.unite(u, v)
print(d.components, d.size(V(1)))
print(if d.same(V(1), V(3)): "YES" else: "NO")
