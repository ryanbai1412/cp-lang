let (n, m): (1..=100000, 0..=200000) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V) = read()
let g = Graph.directed(edges)
print(yes(find_cycle(g) != None))
