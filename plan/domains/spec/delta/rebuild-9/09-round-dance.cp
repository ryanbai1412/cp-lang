// CF 1833E Round Dance
// in: t; per test n, a_1..a_n (a neighbour of each person). out: min and max number of round dances.
@!multitest
let n = read<2..=2e5>()
domain V = 1..=n
let a = read<[V]V>()

var d = Dsu<V>()
for v in V: let _ = d.unite(v, a[v])
// a component is an open chain exactly when it holds one mutual pair (v and a[v] name each other)
let chains = V.count(v => a[a[v]] == v) / 2
let rings = d.components - chains
print(rings + min(chains, 1), d.components)
