// Source: AtCoder Educational DP Contest V (Subtree), https://atcoder.jp/contests/dp/tasks/dp_v
// Problem: a tree of N <= 1e5 vertices. For every vertex v, count the ways to paint vertices
//   black so the black ones are connected and include v, modulo M (M <= 1e9, not prime).
// Solution: rooted at v, a connected set containing v picks, for each neighbour c, either
//   nothing of c's side or a connected set of it containing c: ways[v] = product over
//   neighbours c of (1 + ways[c] rooted at v). `reroot` computes that for every root at once
//   from prefix/suffix folds of the siblings, so no division is needed (M may not be prime).
//   O(N).
// Bounds: n: 1..=1e5; m: 2..=1e9
let (n, m): (1..=1e5, 2..=1e9) = read()
domain V = 1..=n
let tree = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first!)

let ways = tree.reroot(Monoid(1, (a, b) => a * b % m), (x, _, _) => (x + 1) % m, (u, _) => u)
for v in V: print(ways[v])
