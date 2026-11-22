// CSES 1676 Road Construction: after each road, the component count and the largest size.
let (n, m): (1..=1e5, 1..=2e5) = read()
domain V = 1..=n
var dsu = Dsu<V>()
var largest = 1
repeat m:
    let (a, b): (V, V) = read()
    if dsu.unite(a, b):
        largest max= dsu.size(a)
    print(dsu.components, largest)
