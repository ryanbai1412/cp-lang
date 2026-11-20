// AtCoder DP G Longest Path: the longest path in a DAG, in edges.

let (n, m): (2..=1e5, 1..=1e5) = read()
domain V = 1..=n
domain E = 0..m
let dag = Graph<V, E>.directed(read())

var longest: [V]Int = 0
for u in topo(dag)!.rev:                // every edge leads to an already finished vertex
    for (v, _) in dag.out(u): longest[u] max= longest[v] + 1
print(max(longest))
