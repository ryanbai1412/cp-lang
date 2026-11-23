let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V) = read()
let (domain C, comp) = scc(Graph.directed(es))
print(C.len)
