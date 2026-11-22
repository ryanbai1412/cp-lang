let (n, m) = read<(1..=1e5, 1..=2e5)>()
domain V = 1..=n
var d = Dsu<V>()
var largest = 1
for _ in 0..m:
    let (a, b) = read<(V, V)>()
    d.unite(a, b)
    largest max= d.size(a)
    print(d.components, largest)
