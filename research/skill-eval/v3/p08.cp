let (n, m) = read<(1..=1e5, 1..=2e5)>()
domain V = 1..=n
var cities = Dsu<V>()
var largest = 1
for road in read<[m](u: V, v: V)>():
    cities.unite(road.u, road.v)
    largest max= cities.size(road.u)
    print(cities.components, largest)
