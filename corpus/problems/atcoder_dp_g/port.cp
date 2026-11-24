// Source: AtCoder Educational DP G (Longest Path), https://atcoder.jp/contests/dp/tasks/dp_g
// Problem: a DAG with 2 <= n <= 1e5 vertices and m <= 1e5 distinct edges. Print the number of
//   edges on its longest directed path.
// Solution: longest[u] = max(longest[v] + 1) over edges u -> v, filled in reverse topological
//   order so every successor is done first; print the max. O(n + m).
// Bounds: n: 2..=1e5; m: 1..=1e5

let (n, m): (2..=1e5, 1..=1e5) = read()
domain V = 1..=n
domain E = 0..m
let arcs: [E](V, V) = read()
let dag = Graph<V, E>.directed(arcs)

var longest: [V]Int = 0
for u in topo(dag)!.rev:  // every edge leads to an already finished vertex
    for v in dag.out(u): longest[u] max= longest[v] + 1
print(longest.max()!)
