// CF 1176E Cover it!: the smaller BFS parity class from vertex 1 dominates the graph.
@!multitest
let n: 2..=2e5 = read()
let m: 1..=2e5 = read()
domain V = 1..=n
let es = read<[m](u: V, v: V)>()
let d = bfs(Graph.undirected(es), from: [V(1)])
let even = [v for v in V if d[v]! % 2 == 0]
let odd = [v for v in V if d[v]! % 2 == 1]
let pick = if even.len <= odd.len: even else: odd
print(pick.len)
print(pick)
