// expect: redundant_prelude_import
let n: 2..=100 = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V(1))
print(t.size(V(1)))
