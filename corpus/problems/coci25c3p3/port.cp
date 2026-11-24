// Source: COCI 2025/2026 Contest 3 P3 (Domjenak), https://dmoj.ca/problem/coci25c3p3
// limits: 1000 ms, 512 MB
// Problem: 2N people (N <= 5e5), M <= 1e6 friendships, bipartite, with a unique perfect
//   matching (the pairs who came together). Told a story, a person passes it to their pair if
//   the pair hasn't heard it, else to one friend who hasn't. Print the longest chain of
//   tellers and one such sequence: an alternating path that starts with a pair edge.
// Solution: with a unique perfect matching some vertex has degree 1; two-colour the graph,
//   then peel degree-1 vertices on side 0: the lone unmatched neighbour is its pair, and
//   removing the pair's other edges creates new degree-1 vertices. The peel order is a DAG
//   of non-pair edges, so dp[v] (the longest chain of pairs ending at v) relaxes along it,
//   with a parent pointer to rebuild the path. O(N + M).
// original: corpus/submissions/7730378.cpp
// Bounds: n: 1..=5e5; m: 1..=1e6; edges.u, edges.v: 1..=2 * n

let (n, m): (1..=5e5, 1..=1e6) = read()
domain V = 1..=2 * n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)
let side = two_color(g)!

var deg: [V]Int = [V] u => g.deg(u)
var mate: [V]?V = None
var chain: [V]Int = 0  // pairs on the longest chain that ends at v's pair
var from: [V]?V = None
var todo = Deque<V>()
for u in V if not side[u] and deg[u] == 1: todo.push(u)
var peeled: List<V> = []
while let u = todo.pop_front():
    peeled.push(u)
    for a in g.out(u) if mate[a.to] == None:
        mate[a.to] = u
        mate[u] = a.to
    for a in g.out(mate[u]!) if mate[a.to] == None:
        let v = a.to
        if chain[v] < chain[u] + 1:
            chain[v] = chain[u] + 1
            from[v] = u
        deg[v] -= 1
        if deg[v] == 1: todo.push(v)

var tellers: List<V> = []
var at: ?V = peeled[peeled.argmax(by: u => chain[u])!]
while let u = at:
    tellers.push(mate[u]!)
    tellers.push(u)
    at = from[u]
print(tellers.len)
print(tellers)
