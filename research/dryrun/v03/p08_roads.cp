// CSES 1676 Road Construction: after each road, the component count and the largest size.
let (n, m): (1..=1e5, 1..=2e5) = read()
domain V = 1..=n
var dsu = Dsu<V>.new()
var largest = 1
for _ in 0..m:
    let (a, b) = read<(V, V)>()
    if dsu.unite(a, b):
        largest max= dsu.size(a)
    print(dsu.components, largest)
