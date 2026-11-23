let (n, m): (1..=200000, 0..=200000) = read()
domain V = 1..=n
domain E = 0..m
let es: [E](u: V, v: V) = read()
let g = Graph.directed(es)
print(g.deg(1))
